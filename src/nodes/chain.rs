use crate::util::wait_until;
use crate::Nodes;
use ckb_types::{
    core::{BlockNumber, HeaderView},
    packed::Byte32,
};
use std::collections::HashSet;

impl Nodes {
    pub fn waiting_for_sync(&self) -> Result<(), Vec<(&str, BlockNumber, Byte32)>> {
        crate::trace!("Nodes::waiting_for_sync start");
        let highest_hashes: HashSet<_> = {
            let tip_blocks: HashSet<_> = self.nodes().map(|node| node.get_tip_block()).collect();
            let tip_numbers = tip_blocks.iter().map(|block| block.number());
            let highest_number = tip_numbers.max().unwrap();
            let highest_blocks = tip_blocks
                .into_iter()
                .filter(|block| block.number() == highest_number);
            highest_blocks.map(|block| block.hash()).collect()
        };

        // 60 seconds is a reasonable timeout to sync, even for poor CI server
        let synced = wait_until(60, || {
            highest_hashes.iter().all(|hash| {
                self.nodes()
                    .all(|node| node.rpc_client().get_header(hash.clone()).is_some())
            })
        });

        if !synced {
            let tips = self
                .nodes()
                .map(|node| {
                    let block = node.get_tip_block();
                    (node.node_name(), block.number(), block.hash())
                })
                .collect::<Vec<_>>();
            return Err(tips);
        }
        for node in self.nodes() {
            node.wait_for_tx_pool();
        }
        crate::trace!("Nodes::waiting_for_sync end");
        Ok(())
    }

    pub fn get_fixed_header(&self) -> HeaderView {
        let maximal_number = self
            .nodes()
            .map(|node| node.get_tip_block_number())
            .min()
            .expect("at least 1 node");
        for number in (0..=maximal_number).rev() {
            let headers = self
                .nodes()
                .map(|node| node.get_header_by_number(number))
                .collect::<HashSet<_>>();
            if headers.len() == 1 {
                return headers.into_iter().collect::<Vec<_>>()[0].to_owned();
            }
        }
        unreachable!()
    }
}
