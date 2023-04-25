use crate::Node;
use ckb_types::core::{Cycle, TransactionView};

impl Node {
    /// Get the transaction cycles via RPC `dry_run_transaction`.
    ///
    /// NOTE: Transaction runs on different VM comsumes different cycles.
    /// Therefore, if a transaction triggers a script with `ScriptHashType::Type`,
    /// its transaction cycles depend on whether the node is fork2021 activated.
    pub fn get_transaction_cycles(&self, transaction: &TransactionView) -> Cycle {
        let dry_run_result = self
            .rpc_client()
            .estimate_cycles(transaction.data().into());
        dry_run_result.cycles.value()
    }
}
