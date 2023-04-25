use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct NodeOptions {
    pub node_name: String,
    pub ckb_binary: PathBuf,
    pub initial_database: &'static str,
    pub chain_spec: &'static str,
    pub app_config: &'static str,
}
