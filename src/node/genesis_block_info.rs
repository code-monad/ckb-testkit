use crate::Node;
use ckb_types::packed::Byte32;

impl Node {
    pub fn genesis_cellbase_hash(&self) -> Byte32 {
        self.genesis_block().transactions()[0].hash()
    }

    pub fn dep_group_tx_hash(&self) -> Byte32 {
        self.genesis_block().transactions()[1].hash()
    }
}
