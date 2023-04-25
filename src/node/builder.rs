use crate::{Node, NodeOptions};
use ckb_jsonrpc_types::TransactionTemplate;
use ckb_types::{
    core::{BlockNumber, TransactionView},
    packed::{self, ProposalShortId},
    prelude::*,
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum BuildInstruction {
    SendTransaction {
        template_number: BlockNumber,
        transaction: TransactionView,
    },
    Propose {
        template_number: BlockNumber,
        proposal_short_id: ProposalShortId,
    },
    Commit {
        template_number: BlockNumber,
        transaction: TransactionView,
    },
    ProcessWithoutVerify {
        template_number: BlockNumber,
    },
    HeaderTimestamp {
        template_number: BlockNumber,
        timestamp: u64,
    },
}

impl BuildInstruction {
    pub fn template_number(&self) -> BlockNumber {
        match self {
            BuildInstruction::SendTransaction {
                template_number, ..
            } => *template_number,
            BuildInstruction::Propose {
                template_number, ..
            } => *template_number,
            BuildInstruction::Commit {
                template_number, ..
            } => *template_number,
            BuildInstruction::ProcessWithoutVerify { template_number } => *template_number,
            BuildInstruction::HeaderTimestamp {
                template_number, ..
            } => *template_number,
        }
    }
}

impl Node {
    pub fn build_according_to_instructions(
        &self,
        target_height: BlockNumber,
        instructions: Vec<BuildInstruction>,
    ) -> Result<(), String> {
        assert!(self.consensus().permanent_difficulty_in_dummy);
        let initial_tip_number = self.get_tip_block_number();
        let mut instructions_map: HashMap<BlockNumber, Vec<BuildInstruction>> = HashMap::new();
        for instruction in instructions {
            assert!(
                initial_tip_number < instruction.template_number(),
                "initial_tip_number: {}, instruction.template_number: {}",
                initial_tip_number,
                instruction.template_number()
            );
            assert!(
                target_height >= instruction.template_number(),
                "target_height: {}, instruction.template_number: {}",
                target_height,
                instruction.template_number(),
            );
            instructions_map
                .entry(instruction.template_number())
                .or_default()
                .push(instruction);
        }

        // build chain according to instructions
        let mut next_template_number = self.get_tip_block_number() + 1;
        loop {
            let mut template = self.rpc_client().get_block_template(None, None, None);
            let number = template.number.value();
            if number > target_height {
                break;
            }
            if number != next_template_number {
                // avoid issues cause by tx-pool async update
                continue;
            } else {
                next_template_number += 1;
            }

            if let Some(instructions) = instructions_map.remove(&number) {
                let mut process_without_verify = false;
                for instruction in instructions {
                    match &instruction {
                        BuildInstruction::SendTransaction { transaction, .. } => {
                            self.rpc_client()
                                .send_transaction_result(transaction.data().into())
                                .map_err(|err| {
                                    format!("failed to execute {:?}, error: {}", instruction, err)
                                })?;
                        }
                        BuildInstruction::Propose {
                            proposal_short_id, ..
                        } => {
                            let proposal_short_id = proposal_short_id.to_owned().into();
                            if !template.proposals.contains(&proposal_short_id) {
                                template.proposals.push(proposal_short_id);
                            }
                        }
                        BuildInstruction::Commit { transaction, .. } => {
                            let transaction_template = TransactionTemplate {
                                hash: transaction.hash().unpack(),
                                data: transaction.data().into(),
                                ..Default::default()
                            };
                            if !template
                                .transactions
                                .iter()
                                .any(|tx| tx.hash.as_bytes() == transaction.hash().as_bytes())
                            {
                                template.transactions.push(transaction_template);
                            }
                        }
                        BuildInstruction::ProcessWithoutVerify { .. } => {
                            process_without_verify = true;
                        }
                        BuildInstruction::HeaderTimestamp { timestamp, .. } => {
                            template.current_time = ckb_jsonrpc_types::Timestamp::from(*timestamp);
                        }
                    }
                }
                let updated_block: packed::Block = {
                    let dao_field = self
                        .rpc_client()
                        .calculate_dao_field(template.clone())
                        .map_err(|err| {
                            format!(
                                "failed to calculate dao field, block number: {}, error: {}",
                                number, err
                            )
                        })?;
                    template.dao = dao_field.into();
                    template.into()
                };
                if process_without_verify {
                    self.rpc_client()
                        .process_block_without_verify(updated_block.into(), true);
                } else {
                    self.rpc_client()
                        .submit_block("".to_string(), updated_block.into())
                        .map_err(|err| {
                            format!("failed to send block {}, error: {}", number, err)
                        })?;
                }
            } else {
                let block: packed::Block = template.into();
                self.rpc_client()
                    .submit_block("".to_string(), block.into())
                    .map_err(|err| format!("failed to send block {}, error: {}", number, err))?;
            }
        }
        Ok(())
    }

    /// Return the cloned node with `node_name`.
    pub fn clone_node(&self, node_name: &str) -> Node {
        let mut target_node = {
            let node_options = NodeOptions {
                node_name: String::from(node_name),
                ..self.node_options().clone()
            };
            let is_ckb2021 = self.rpc_client().ckb2021;
            Node::init("cloned_node", node_options, is_ckb2021)
        };
        target_node.start();

        target_node
            .pull_node(self)
            .expect("cloned node pull from source node should be ok");

        target_node
    }

    pub fn pull_node(&self, source_node: &Node) -> Result<(), String> {
        assert!(self.get_tip_block_number() <= source_node.get_tip_block_number());
        let min_tip_number = self.get_tip_block_number();
        let max_tip_number = source_node.get_tip_block_number();
        let mut fixed_number = min_tip_number;

        for number in (0..=min_tip_number).rev() {
            if self.rpc_client().get_block_hash(number)
                == source_node.rpc_client().get_block_hash(number)
            {
                fixed_number = number;
                break;
            }
        }

        for number in fixed_number + 1..=max_tip_number {
            let block = source_node.get_block_by_number(number);
            self.rpc_client()
                .submit_block(block.number().to_string(), block.data().into())
                .map_err(|err| err.to_string())?;
        }
        Ok(())
    }
}
