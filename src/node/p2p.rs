use crate::util::wait_until;
use crate::Node;

impl Node {
    pub fn is_p2p_connected(&self, other: &Node) -> bool {
        self.rpc_client()
            .get_peers()
            .iter()
            .any(|peer| &peer.node_id == other.node_id())
    }

    pub fn p2p_connect(&self, other: &Node) {
        crate::trace!(
            "Node::p2p_connect(\"{}\", \"{}\") start",
            self.node_name(),
            other.node_name()
        );
        let other_node_id = other.node_id().to_string();
        let other_p2p_address = other.p2p_address();

        self.rpc_client().add_node(other_node_id, other_p2p_address);
        let connected = wait_until(20, || {
            self.rpc_client()
                .get_peers()
                .iter()
                .any(|remote_node| remote_node.node_id == other.node_id())
        });
        if !connected {
            panic!(
                "timeout to connect outbound peer, \
                self node name: {}, self p2p address: {}, other node name: {}, other p2p address: {}",
                self.node_name(),
                self.p2p_address(),
                other.node_name(),
                other.p2p_address(),
            );
        }
        crate::trace!("Node::p2p_connect end");
    }

    pub fn p2p_connect_uncheck(&self, other: &Node) {
        let other_node_id = other.node_id().to_string();
        let other_p2p_address = other.p2p_address();

        self.rpc_client().add_node(other_node_id, other_p2p_address);
    }

    pub fn p2p_disconnect(&self, other: &Node) {
        let other_node_id = other.node_id().to_string();

        self.rpc_client().remove_node(other_node_id);
        let disconnected = wait_until(5, || {
            self.rpc_client()
                .get_peers()
                .iter()
                .all(|remote_node| remote_node.node_id != other.node_id())
                && other
                    .rpc_client()
                    .get_peers()
                    .iter()
                    .all(|remote_node| remote_node.node_id != self.node_id())
        });
        if !disconnected {
            panic!(
                "timeout to disconnect peer, \
                self node name: {}, self node id: {}, other node name: {}, other node id: {}",
                self.node_name(),
                self.node_id(),
                other.node_name(),
                other.node_id(),
            );
        }
    }

    // TODO so confusing
    // workaround for banned address checking (because we are using loopback address)
    // 1. checking banned addresses is empty
    // 2. connecting outbound peer and checking banned addresses is not empty
    // 3. clear banned addresses
    pub fn p2p_connect_and_wait_ban(&self, other: &Node) {
        let other_node_id = other.node_id().to_string();
        let other_p2p_address = other.p2p_address();
        let rpc_client = self.rpc_client();
        assert!(
            rpc_client.get_banned_addresses().is_empty(),
            "banned addresses should be empty"
        );
        rpc_client.add_node(other_node_id, other_p2p_address);
        let banned = wait_until(10, || {
            let banned_addresses = rpc_client.get_banned_addresses();
            !banned_addresses.is_empty()
        });

        // Clear
        if banned {
            let banned_addresses = rpc_client.get_banned_addresses();
            for banned_address in banned_addresses {
                rpc_client.set_ban(
                    banned_address.address,
                    "delete".to_owned(),
                    None,
                    None,
                    None,
                )
            }
        } else {
            panic!(
                "timeout to connect_and_wait_ban peer, \
                self node name: {}, self node id: {}, other node name: {}, other node id: {}",
                self.node_name(),
                self.node_id(),
                other.node_name(),
                other.node_id(),
            );
        }
    }
}
