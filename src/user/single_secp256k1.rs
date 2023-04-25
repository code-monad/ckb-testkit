use crate::{
    Node, User, GENESIS_DEP_GROUP_TRANSACTION_INDEX, GENESIS_SIGHASH_ALL_DEP_GROUP_CELL_INDEX,
    SIGHASH_ALL_DATA_HASH, SIGHASH_ALL_TYPE_HASH,
};
use ckb_crypto::secp::{Message, Pubkey, Signature};
use ckb_hash::blake2b_256;
use ckb_types::core::cell::CellMeta;
use ckb_types::core::EpochNumberWithFraction;
use ckb_types::{
    bytes::Bytes,
    core::{DepType, ScriptHashType, TransactionView},
    packed::{CellDep, OutPoint, Script, WitnessArgs},
    prelude::*,
    H160, H256,
};

impl User {
    pub fn single_secp256k1_lock_script_via_type(&self) -> Script {
        Script::new_builder()
            .hash_type(ScriptHashType::Type.into())
            .code_hash(SIGHASH_ALL_TYPE_HASH.pack())
            .args(self.single_secp256k1_address().0.pack())
            .build()
    }

    pub fn single_secp256k1_lock_script_via_data(&self) -> Script {
        Script::new_builder()
            .hash_type(ScriptHashType::Data.into())
            .code_hash(SIGHASH_ALL_DATA_HASH.pack())
            .args(self.single_secp256k1_address().0.pack())
            .build()
    }

    pub fn single_secp256k1_lock_script_via_data1(&self) -> Script {
        Script::new_builder()
            .hash_type(ScriptHashType::Data1.into())
            .code_hash(SIGHASH_ALL_DATA_HASH.pack())
            .args(self.single_secp256k1_address().0.pack())
            .build()
    }

    pub fn single_secp256k1_address(&self) -> H160 {
        let pubkey = self.single_secp256k1_pubkey();
        H160::from_slice(&blake2b_256(pubkey.serialize())[0..20]).unwrap()
    }

    pub fn single_secp256k1_out_point(&self) -> OutPoint {
        OutPoint::new_builder()
            .tx_hash(
                self.genesis_block
                    .transaction(GENESIS_DEP_GROUP_TRANSACTION_INDEX)
                    .expect("index genesis dep-group transaction")
                    .hash(),
            )
            .index(GENESIS_SIGHASH_ALL_DEP_GROUP_CELL_INDEX.pack())
            .build()
    }

    pub fn single_secp256k1_cell_dep(&self) -> CellDep {
        CellDep::new_builder()
            .out_point(self.single_secp256k1_out_point())
            .dep_type(DepType::DepGroup.into())
            .build()
    }

    pub fn single_secp256k1_pubkey(&self) -> Pubkey {
        if let Some(ref privkey) = self.single_secp256k1_privkey {
            privkey.pubkey().unwrap()
        } else {
            unreachable!("single_secp256k1 unset")
        }
    }

    pub fn single_secp256k1_signed_witness(&self, tx: &TransactionView) -> WitnessArgs {
        if let Some(ref privkey) = self.single_secp256k1_privkey {
            let tx_hash = tx.hash();
            let mut blake2b = ckb_hash::new_blake2b();
            let mut message = [0u8; 32];
            blake2b.update(&tx_hash.raw_data());
            let witness_for_digest = WitnessArgs::new_builder()
                .lock(Some(Bytes::from(vec![0u8; 65])).pack())
                .build();
            let witness_len = witness_for_digest.as_bytes().len() as u64;
            blake2b.update(&witness_len.to_le_bytes());
            blake2b.update(&witness_for_digest.as_bytes());
            blake2b.finalize(&mut message);
            let message = H256::from(message);
            let sig = privkey.sign_recoverable(&message).expect("sign");
            WitnessArgs::new_builder()
                .lock(Some(Bytes::from(sig.serialize())).pack())
                .build()
            // .as_bytes()
            // .pack()
        } else {
            unreachable!("single_secp256k1 unset")
        }
    }

    pub fn sign_recoverable(&self, message: &Message) -> Signature {
        if let Some(ref privkey) = self.single_secp256k1_privkey {
            privkey.sign_recoverable(message).expect("sign")
        } else {
            unreachable!("single_secp256k1 unset")
        }
    }

    pub fn get_spendable_single_secp256k1_cells(&self, node: &Node) -> Vec<CellMeta> {
        Vec::new()
        //let tip_number = node.get_tip_block_number();
        //let mut live_out_points = Vec::new();

        // FIXME: The Indexer has changed into a bounded one
        //live_out_points.extend(
        //    node.rpc_client()
        //        .get_live_cells_by_lock_script(&self.single_secp256k1_lock_script_via_type())
        //        .expect("indexer get_live_cells_by_lock_script"),
        //);
        //live_out_points.extend(
        //    node.indexer()
        //        .get_live_cells_by_lock_script(&self.single_secp256k1_lock_script_via_data())
        //        .expect("indexer get_live_cells_by_lock_script"),
        //);
        //live_out_points.extend(
        //    node.indexer()
        //        .get_live_cells_by_lock_script(&self.single_secp256k1_lock_script_via_data1())
        //        .expect("indexer get_live_cells_by_lock_script"),
        //);
        //
        //live_out_points
        //    .into_iter()
        //    .filter_map(|out_point| {
        //        let cell_meta = node.get_cell_meta(out_point)?;

        //        let txinfo = cell_meta
        //            .transaction_info
        //            .as_ref()
        //            .expect("committed tx has transaction_info");
        //        if txinfo.is_cellbase() {
        //            let cellbase_maturity: EpochNumberWithFraction = {
        //                EpochNumberWithFraction::from_full_value(
        //                    node.consensus().cellbase_maturity.into(),
        //                )
        //            };
        //            // We didn't fill the block_epoch inside `fn get_cell_meta`
        //            if txinfo.block_number + cellbase_maturity.number() * 1800 > tip_number {
        //                return None;
        //            }
        //        }

        //        if cell_meta.data_bytes != 0 {
        //            return None;
        //        }

        //        Some(cell_meta)
        //    })
        //    .collect::<Vec<_>>()
    }
}
