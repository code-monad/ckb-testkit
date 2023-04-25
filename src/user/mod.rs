pub mod single_secp256k1;

use ckb_crypto::secp::Privkey;
use ckb_types::core::BlockView;

#[derive(Clone)]
pub struct User {
    // a workaround to get out-point of system script cells
    genesis_block: BlockView,
    single_secp256k1_privkey: Option<Privkey>,
}

impl User {
    pub fn new(genesis_block: BlockView, single_secp256k1_privkey: Option<Privkey>) -> Self {
        Self {
            genesis_block,
            single_secp256k1_privkey,
        }
    }

    pub fn single_secp256k1_privkey(&self) -> Option<&Privkey> {
        self.single_secp256k1_privkey.as_ref()
    }
}
