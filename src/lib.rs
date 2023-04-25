pub mod connector;
pub mod logger;
mod node;
mod nodes;
mod rpc;
#[cfg(feature = "with_subscribe")]
mod subscribe;
mod user;
pub mod util;

pub use connector::{compress, decompress, Connector, ConnectorBuilder, SupportProtocols};
pub use logger::LOG_TARGET;
pub use node::{BuildInstruction, Node, NodeOptions};
pub use nodes::Nodes;
pub use user::User;

pub use ckb_crypto;
pub use ckb_jsonrpc_types;
pub use ckb_types;
pub use p2p;

use ckb_types::{h256, H256};

pub const SYSTEM_CELL_ALWAYS_SUCCESS_INDEX: u32 = 5;
pub const GENESIS_DEP_GROUP_TRANSACTION_INDEX: usize = 1;
pub const GENESIS_SIGHASH_ALL_DEP_GROUP_CELL_INDEX: usize = 0;
pub const SIGHASH_ALL_TYPE_HASH: H256 =
    h256!("0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8");
pub const SIGHASH_ALL_DATA_HASH: H256 =
    h256!("0x709f3fda12f561cfacf92273c57a98fede188a3f1a59b1f888d113f9cce08649");
