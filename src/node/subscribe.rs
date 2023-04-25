use crate::subscribe::{
    subscribe_new_tip_block, subscribe_new_tip_header, subscribe_new_transaction,
    subscribe_proposed_transaction, subscribe_rejected_transaction, Handle as SubscribeHandle,
};
use crate::Node;
use std::net::SocketAddr;
use tokio::net::ToSocketAddrs;

impl Node {
    pub async fn subscribe_new_tip_block<A: ToSocketAddrs>(&mut self, subscription_addr: A) {
        let handle = subscribe_new_tip_block(subscription_addr).await.unwrap();
        self.new_tip_block_subscriber = Some(handle);
    }

    pub fn new_tip_block_subscriber(
        &mut self,
    ) -> &mut SubscribeHandle<tokio::net::TcpStream, ckb_jsonrpc_types::BlockView> {
        self.new_tip_block_subscriber.as_mut().unwrap()
    }

    pub async fn subscribe_new_tip_header<A: ToSocketAddrs>(&mut self, subscription_addr: A) {
        let handle = subscribe_new_tip_header(subscription_addr).await.unwrap();
        self.new_tip_header_subscriber = Some(handle);
    }

    pub fn new_tip_header_subscriber(
        &mut self,
    ) -> &mut SubscribeHandle<tokio::net::TcpStream, ckb_jsonrpc_types::HeaderView> {
        self.new_tip_header_subscriber.as_mut().unwrap()
    }

    pub async fn subscribe_new_transaction<A: ToSocketAddrs>(&mut self, subscription_addr: A) {
        let handle = subscribe_new_transaction(subscription_addr).await.unwrap();
        self.new_transaction_subscriber = Some(handle);
    }

    pub fn new_transaction_subscriber(
        &mut self,
    ) -> &mut SubscribeHandle<tokio::net::TcpStream, ckb_jsonrpc_types::PoolTransactionEntry> {
        self.new_transaction_subscriber.as_mut().unwrap()
    }

    pub async fn subscribe_proposed_transaction<A: ToSocketAddrs>(&mut self, subscription_addr: A) {
        let handle = subscribe_proposed_transaction(subscription_addr)
            .await
            .unwrap();
        self.proposed_transaction_subscriber = Some(handle);
    }

    pub fn proposed_transaction_subscriber(
        &mut self,
    ) -> &mut SubscribeHandle<tokio::net::TcpStream, ckb_jsonrpc_types::PoolTransactionEntry> {
        self.proposed_transaction_subscriber.as_mut().unwrap()
    }

    pub async fn subscribe_rejected_transaction<A: ToSocketAddrs>(&mut self, subscription_addr: A) {
        let handle = subscribe_rejected_transaction(subscription_addr)
            .await
            .unwrap();
        self.rejected_transaction_subscriber = Some(handle);
    }

    pub fn rejected_transaction_subscriber(
        &mut self,
    ) -> &mut SubscribeHandle<
        tokio::net::TcpStream,
        (
            ckb_jsonrpc_types::PoolTransactionEntry,
            ckb_jsonrpc_types::PoolTransactionReject,
        ),
    > {
        self.rejected_transaction_subscriber.as_mut().unwrap()
    }
}
