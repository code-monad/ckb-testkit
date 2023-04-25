use std::any::Any;
use std::convert::TryInto;
use crate::Node;
use ckb_jsonrpc_types::{Status, TxStatus};
use ckb_types::core::TransactionView;

impl Node {
    pub fn is_transaction_pending(&self, transaction: &TransactionView) -> bool {
        self.rpc_client()
            .get_transaction(transaction.hash())
            .map(|txstatus| TxStatus::from(txstatus.tx_status).status == Status::Pending)
            .unwrap_or(false)
    }

    pub fn is_transaction_proposed(&self, transaction: &TransactionView) -> bool {
        self.rpc_client()
            .get_transaction(transaction.hash())
            .map(|txstatus| TxStatus::from(txstatus.tx_status).status == Status::Proposed)
            .unwrap_or(false)
    }

    pub fn is_transaction_committed(&self, transaction: &TransactionView) -> bool {
        self.rpc_client()
            .get_transaction(transaction.hash())
            .map(|txstatus| TxStatus::from(txstatus.tx_status).status == Status::Committed)
            .unwrap_or(false)
    }

    pub fn is_transaction_unknown(&self, transaction: &TransactionView) -> bool {
        self.rpc_client()
            .get_transaction(transaction.hash())
            .is_none()
    }
}
