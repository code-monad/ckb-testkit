use crate::Node;
use ckb_jsonrpc_types::TxPoolInfo;
use ckb_types::{
    core::{BlockNumber, BlockView, HeaderView, TransactionView},
    packed::Byte32,
};
use std::thread::sleep;
use std::time::{Duration, Instant};

impl Node {
    pub fn submit_block(&self, block: &BlockView) -> Byte32 {
        let hash = self
            .rpc_client()
            .submit_block("".to_owned(), block.data().into())
            .unwrap();
        self.wait_for_tx_pool();
        hash
    }

    pub fn submit_transaction(&self, transaction: &TransactionView) -> Byte32 {
        self.rpc_client()
            .send_transaction(transaction.data().into())
    }

    pub fn get_tip_block(&self) -> BlockView {
        let rpc_client = self.rpc_client();
        let tip_number = rpc_client.get_tip_block_number();
        let block = rpc_client
            .get_block_by_number(tip_number)
            .expect("tip block exists");
        crate::trace!(
            "[Node {}] Node::get_tip_block(), block: {:?}",
            self.node_name(),
            block
        );
        block.into()
    }

    pub fn get_tip_block_number(&self) -> BlockNumber {
        let block_number = self.rpc_client().get_tip_block_number();
        crate::trace!(
            "[Node {}] Node::get_tip_block_number(), block_number: {}",
            self.node_name(),
            block_number
        );
        block_number
    }

    pub fn get_block(&self, hash: Byte32) -> BlockView {
        self.rpc_client()
            .get_block(hash)
            .expect("block exists")
            .into()
    }

    pub fn get_block_by_number(&self, number: BlockNumber) -> BlockView {
        self.rpc_client()
            .get_block_by_number(number)
            .expect("block exists")
            .into()
    }

    pub fn get_header_by_number(&self, number: BlockNumber) -> HeaderView {
        self.rpc_client()
            .get_header_by_number(number)
            .expect("header exists")
            .into()
    }

    /// The states of chain and txpool are updated asynchronously. Which means that the chain has
    /// updated to the newest tip but txpool not.
    /// get_tip_tx_pool_info wait to ensure the txpool update to the newest tip as well.
    pub fn get_tip_tx_pool_info(&self) -> TxPoolInfo {
        let tip_header = self.rpc_client().get_tip_header();
        let tip_hash = &tip_header.hash;
        let instant = Instant::now();
        let mut recent = TxPoolInfo::default();
        while instant.elapsed() < Duration::from_secs(10) {
            let tx_pool_info = self.rpc_client().tx_pool_info();
            if &tx_pool_info.tip_hash == tip_hash {
                return tx_pool_info;
            }
            recent = tx_pool_info;
        }
        panic!(
            "timeout to get_tip_tx_pool_info, tip_header={:?}, tx_pool_info: {:?}",
            tip_header, recent
        );
    }

    pub fn wait_for_tx_pool(&self) {
        let rpc_client = self.rpc_client();
        let mut chain_tip = rpc_client.get_tip_header();
        let mut tx_pool_tip = rpc_client.tx_pool_info();
        if chain_tip.hash == tx_pool_tip.tip_hash {
            return;
        }
        let mut instant = Instant::now();
        while instant.elapsed() < Duration::from_secs(10) {
            sleep(std::time::Duration::from_secs(1));
            chain_tip = rpc_client.get_tip_header();
            let prev_tx_pool_tip = tx_pool_tip;
            tx_pool_tip = rpc_client.tx_pool_info();
            if chain_tip.hash == tx_pool_tip.tip_hash {
                return;
            } else if prev_tx_pool_tip.tip_hash != tx_pool_tip.tip_hash
                && tx_pool_tip.tip_number.value() < chain_tip.inner.number.value()
            {
                instant = Instant::now();
            }
        }
        panic!(
            "timeout to wait for tx pool,\n\tchain   tip: {:?}, {:#x},\n\ttx-pool tip: {}, {:#x}",
            chain_tip.inner.number.value(),
            chain_tip.hash,
            tx_pool_tip.tip_number.value(),
            tx_pool_tip.tip_hash,
        );
    }
}
