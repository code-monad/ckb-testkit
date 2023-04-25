use ckb_jsonrpc_types::{Alert, BannedAddr, Block, BlockNumber, BlockTemplate, BlockView, Byte32, Capacity, CellWithStatus, ChainInfo, Consensus, EpochNumber, EpochView, EstimateCycles, HeaderView, JsonBytes, LocalNode, OutPoint, RawTxPool, RemoteNode, Script, Timestamp, Transaction, TransactionWithStatusResponse, TxPoolInfo, Uint64, Version};
use ckb_types::H256;

jsonrpc!(pub struct Inner2021 {
    pub fn get_block(&self, _hash: H256) -> Option<BlockView>;
    pub fn get_fork_block(&self, _hash: H256) -> Option<BlockView>;
    pub fn get_block_by_number(&self, _number: BlockNumber) -> Option<BlockView>;
    pub fn get_header(&self, _hash: H256) -> Option<HeaderView>;
    pub fn get_header_by_number(&self, _number: BlockNumber) -> Option<HeaderView>;
    pub fn get_transaction(&self, _hash: H256) -> Option<TransactionWithStatusResponse>;
    pub fn get_block_hash(&self, _number: BlockNumber) -> Option<H256>;
    pub fn get_tip_header(&self) -> HeaderView;
    pub fn get_live_cell(&self, _out_point: OutPoint, _with_data: bool) -> CellWithStatus;
    pub fn get_tip_block_number(&self) -> BlockNumber;
    pub fn get_current_epoch(&self) -> EpochView;
    pub fn get_epoch_by_number(&self, number: EpochNumber) -> Option<EpochView>;
    pub fn get_consensus(&self) -> Consensus;
    pub fn estimate_cycles(&self, _tx: Transaction) -> EstimateCycles;
    pub fn local_node_info(&self) -> LocalNode;
    pub fn get_peers(&self) -> Vec<RemoteNode>;
    pub fn get_banned_addresses(&self) -> Vec<BannedAddr>;
    pub fn set_ban(
        &self,
        address: String,
        command: String,
        ban_time: Option<Timestamp>,
        absolute: Option<bool>,
        reason: Option<String>
    ) -> ();

    pub fn get_block_template(
        &self,
        bytes_limit: Option<Uint64>,
        proposals_limit: Option<Uint64>,
        max_version: Option<Version>
    ) -> BlockTemplate;
    pub fn submit_block(&self, _work_id: String, _data: Block) -> H256;
    pub fn get_blockchain_info(&self) -> ChainInfo;
    pub fn get_block_median_time(&self, block_hash: H256) -> Option<Timestamp>;
    pub fn send_transaction(&self, tx: Transaction, outputs_validator: Option<String>) -> H256;
    pub fn tx_pool_info(&self) -> TxPoolInfo;

    pub fn send_alert(&self, alert: Alert) -> ();

    pub fn add_node(&self, peer_id: String, address: String) -> ();
    pub fn remove_node(&self, peer_id: String) -> ();
    pub fn process_block_without_verify(&self, _data: Block, broadcast: bool) -> Option<H256>;
    pub fn truncate(&self, target_tip_hash: H256) -> ();
    pub fn generate_block(&self, block_assembler_script: Option<Script>, block_assembler_message: Option<JsonBytes>) -> H256;
    pub fn generate_block_with_template(&self, block_template: BlockTemplate) -> H256;
    pub fn calculate_dao_field(&self, block_template: BlockTemplate) -> Byte32;
    pub fn get_raw_tx_pool(&self, verbose: Option<bool>) -> RawTxPool;

    pub fn calculate_dao_maximum_withdraw(&self, _out_point: OutPoint, _hash: H256) -> Capacity;
});
