use crate::Nodes;

impl Nodes {
    pub fn p2p_connect(&self) {
        for node_a in self.nodes() {
            for node_b in self.nodes() {
                if node_a.p2p_address() != node_b.p2p_address() && !node_a.is_p2p_connected(node_b)
                {
                    if node_a.get_tip_block_number() < node_b.get_tip_block_number() {
                        // An ibd node will not request GetHeaders from inbound peers.
                        // https://github.com/nervosnetwork/ckb/blob/78fb281317aeaaa8b2621908cda79928ac697df4/sync/src/synchronizer/mod.rs#L543
                        node_a.p2p_connect(node_b);
                    } else {
                        node_b.p2p_connect(node_a);
                    }
                }
            }
        }
    }

    pub fn p2p_disconnect(&self) {
        for node_a in self.nodes() {
            for node_b in self.nodes() {
                if node_a.p2p_address() != node_b.p2p_address() && node_a.is_p2p_connected(node_b) {
                    node_a.p2p_disconnect(node_b);
                }
            }
        }
    }
}
