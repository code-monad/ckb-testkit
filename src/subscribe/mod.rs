/// This file is copied from https://github.com/nervosnetwork/ckb-cli/blob/271261e7bd5e54b15b2468a64505eb5183325915/ckb-sdk/src/pubsub/mod.rs
///
/// This module provides a general rpc subscription client,
/// you can use it with any connection method that implements `AsyncWrite + AsyncRead`.
/// The simplest TCP connection is as follows:
///
/// ```ignore
/// use ckb_jsonrpc_types::HeaderView;
/// use ckb_types::core::HeaderView as CoreHeaderView;
/// use tokio::net::{TcpStream, ToSocketAddrs};
///
/// pub async fn new_tcp_client(addr: impl ToSocketAddrs) -> io::Result<Client<TcpStream>> {
///     let tcp = TcpStream::connect(addr).await?;
///     Ok(Client::new(tcp))
/// }
///
/// fn main() {
///     let mut rt = tokio::runtime::Runtime::new().unwrap();
///     rt.block_on(async {
///         let c = new_tcp_client("127.0.0.1:18114").await.unwrap();
///         let mut h = c
///             .subscribe::<HeaderView>("new_tip_header")
///             .await
///             .unwrap();
///         while let Some(Ok(r)) = h.next().await {
///             let core: CoreHeaderView = r.into();
///             println!(
///                 "number: {}, difficulty: {}, epoch: {}, timestamp: {}",
///                 core.number(),
///                 core.difficulty(),
///                 core.epoch(),
///                 core.timestamp()
///             )
///         }
///     });   
/// }
/// ```
///
use std::{
    collections::{HashMap, VecDeque},
    io,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use futures::{
    sink::SinkExt,
    stream::{Stream, StreamExt},
};
use serde::{Deserialize, Serialize};
use tokio_util::codec::Framed;

use stream_codec::StreamCodec;
use tokio::net::{TcpStream, ToSocketAddrs};

mod stream_codec {
    /// copy from jsonrpc [service-util](https://github.com/paritytech/jsonrpc/blob/master/server-utils/src/stream_codec.rs)
    ///
    /// I changed the return value of decode to BytesMut. It is not a good idea to parse it into a string in the codec.
    /// It will cause one more copy, but it is necessary to check whether it is utf8 encoding
    use bytes::BytesMut;
    use std::{io, str};

    /// Separator for enveloping messages in streaming codecs
    #[derive(Debug, Clone)]
    pub enum Separator {
        /// No envelope is expected between messages. Decoder will try to figure out
        /// message boundaries by accumulating incoming bytes until valid JSON is formed.
        /// Encoder will send messages without any boundaries between requests.
        Empty,
        /// Byte is used as an sentitel between messages
        Byte(u8),
    }

    impl Default for Separator {
        fn default() -> Self {
            Separator::Byte(b'\n')
        }
    }

    /// Stream codec for streaming protocols (ipc, tcp)
    #[derive(Debug, Default)]
    pub struct StreamCodec {
        incoming_separator: Separator,
        outgoing_separator: Separator,
    }

    impl StreamCodec {
        /// Default codec with streaming input data. Input can be both enveloped and not.
        pub fn stream_incoming() -> Self {
            StreamCodec::new(Separator::Empty, Default::default())
        }

        /// New custom stream codec
        pub fn new(incoming_separator: Separator, outgoing_separator: Separator) -> Self {
            StreamCodec {
                incoming_separator,
                outgoing_separator,
            }
        }
    }

    fn is_whitespace(byte: u8) -> bool {
        matches!(byte, 0x0D | 0x0A | 0x20 | 0x09)
    }

    impl tokio_util::codec::Decoder for StreamCodec {
        type Item = BytesMut;
        type Error = io::Error;

        fn decode(&mut self, buf: &mut BytesMut) -> io::Result<Option<Self::Item>> {
            if let Separator::Byte(separator) = self.incoming_separator {
                if let Some(i) = buf.as_ref().iter().position(|&b| b == separator) {
                    let line = buf.split_to(i);
                    let _ = buf.split_to(1);

                    match str::from_utf8(&line.as_ref()) {
                        Ok(_) => Ok(Some(line)),
                        Err(_) => Err(io::Error::new(io::ErrorKind::Other, "invalid UTF-8")),
                    }
                } else {
                    Ok(None)
                }
            } else {
                let mut depth = 0;
                let mut in_str = false;
                let mut is_escaped = false;
                let mut start_idx = 0;
                let mut whitespaces = 0;

                for idx in 0..buf.as_ref().len() {
                    let byte = buf.as_ref()[idx];

                    if (byte == b'{' || byte == b'[') && !in_str {
                        if depth == 0 {
                            start_idx = idx;
                        }
                        depth += 1;
                    } else if (byte == b'}' || byte == b']') && !in_str {
                        depth -= 1;
                    } else if byte == b'"' && !is_escaped {
                        in_str = !in_str;
                    } else if is_whitespace(byte) {
                        whitespaces += 1;
                    }
                    if byte == b'\\' && !is_escaped && in_str {
                        is_escaped = true;
                    } else {
                        is_escaped = false;
                    }

                    if depth == 0 && idx != start_idx && idx - start_idx + 1 > whitespaces {
                        let bts = buf.split_to(idx + 1);
                        match str::from_utf8(bts.as_ref()) {
                            Ok(_) => return Ok(Some(bts)),
                            Err(_) => {
                                return Ok(None);
                            } // skip non-utf requests (TODO: log error?)
                        };
                    }
                }
                Ok(None)
            }
        }
    }

    impl tokio_util::codec::Encoder<String> for StreamCodec {
        type Error = io::Error;

        fn encode(&mut self, msg: String, buf: &mut BytesMut) -> io::Result<()> {
            let mut payload = msg.into_bytes();
            if let Separator::Byte(separator) = self.outgoing_separator {
                payload.push(separator);
            }
            buf.extend_from_slice(&payload);
            Ok(())
        }
    }
}

/// General rpc subscription client
pub struct Client<T> {
    inner: Framed<T, StreamCodec>,
    id: usize,
}

impl<T> Client<T>
where
    T: tokio::io::AsyncWrite + tokio::io::AsyncRead + Unpin,
{
    /// New a pubsub rpc client
    pub fn new(io: T) -> Client<T> {
        let inner = Framed::new(io, StreamCodec::stream_incoming());
        Client { inner, id: 0 }
    }

    /// Subscription a topic
    pub async fn subscribe<F: for<'de> serde::de::Deserialize<'de>>(
        mut self,
        name: &str,
    ) -> io::Result<Handle<T, F>> {
        let mut topic_list = HashMap::default();
        let mut pending_recv = VecDeque::new();

        subscribe(
            &mut self.inner,
            self.id,
            name,
            &mut topic_list,
            &mut pending_recv,
        )
        .await?;
        self.id = self.id.wrapping_add(1);

        Ok(Handle {
            inner: self.inner,
            topic_list,
            output: PhantomData::default(),
            rpc_id: self.id,
            pending_recv,
        })
    }

    /// Subscription topics
    pub async fn subscribe_list<
        F: for<'de> serde::de::Deserialize<'de>,
        I: Iterator<Item = H>,
        H: AsRef<str>,
    >(
        mut self,
        name_list: I,
    ) -> io::Result<Handle<T, F>> {
        let mut topic_list = HashMap::default();
        let mut pending_recv = VecDeque::new();

        for topic in name_list {
            subscribe(
                &mut self.inner,
                self.id,
                topic,
                &mut topic_list,
                &mut pending_recv,
            )
            .await?;
            self.id = self.id.wrapping_add(1);
        }

        Ok(Handle {
            inner: self.inner,
            topic_list,
            output: PhantomData::default(),
            rpc_id: self.id,
            pending_recv,
        })
    }
}

/// General rpc subscription topic handle
pub struct Handle<T, F> {
    inner: Framed<T, StreamCodec>,
    topic_list: HashMap<String, String>,
    output: PhantomData<F>,
    rpc_id: usize,
    pending_recv: VecDeque<bytes::BytesMut>,
}

impl<T, F> Handle<T, F>
where
    T: tokio::io::AsyncWrite + tokio::io::AsyncRead + Unpin,
{
    /// Sub ids
    pub fn ids(&self) -> impl Iterator<Item = &String> {
        self.topic_list.keys()
    }

    /// Topic names
    pub fn topics(&self) -> impl Iterator<Item = &String> {
        self.topic_list.values()
    }

    /// if topic is empty, return Ok, else Err
    pub fn try_into(self) -> Result<Client<T>, Self> {
        if self.topic_list.is_empty() {
            Ok(Client {
                inner: self.inner,
                id: self.rpc_id,
            })
        } else {
            Err(self)
        }
    }

    pub async fn subscribe(mut self, topic: &str) -> io::Result<Self> {
        if self.topic_list.iter().any(|(_, v)| *v == topic) {
            return Ok(self);
        }

        subscribe(
            &mut self.inner,
            self.rpc_id,
            topic,
            &mut self.topic_list,
            &mut self.pending_recv,
        )
        .await?;
        self.rpc_id = self.rpc_id.wrapping_add(1);

        Ok(self)
    }

    /// Unsubscribe one topic
    pub async fn unsubscribe(&mut self, topic: &str) -> io::Result<()> {
        let id = {
            let id = self
                .topic_list
                .iter()
                .find_map(|(k, v)| if v == topic { Some(k) } else { None })
                .cloned();
            if id.is_none() {
                return Ok(());
            }
            id.unwrap()
        };
        let req_json = format!(
            r#"{{"id": {}, "jsonrpc": "2.0", "method": "unsubscribe", "params": ["{}"]}}"#,
            self.rpc_id, id
        );
        self.rpc_id = self.rpc_id.wrapping_add(1);

        self.inner.send(req_json).await?;

        let output = loop {
            let resp = self.inner.next().await;

            let resp = resp.ok_or_else::<io::Error, _>(|| io::ErrorKind::BrokenPipe.into())??;

            // Since the current subscription state, the return value may be a notification,
            // we need to ensure that the unsubscribed message returns before jumping out
            match serde_json::from_slice::<jsonrpc_core::response::Output>(&resp) {
                Ok(output) => break output,
                Err(_) => self.pending_recv.push_back(resp),
            }
        };

        match output {
            jsonrpc_core::response::Output::Success(_) => {
                self.topic_list.remove(&id);
                Ok(())
            }
            jsonrpc_core::response::Output::Failure(e) => {
                Err(io::Error::new(io::ErrorKind::InvalidData, e.error))
            }
        }
    }

    /// Unsubscribe and return this Client
    pub async fn unsubscribe_all(mut self) -> io::Result<Client<T>> {
        for topic in self.topic_list.clone().values() {
            self.unsubscribe(topic).await?
        }
        Ok(Client {
            inner: self.inner,
            id: self.rpc_id,
        })
    }
}

impl<T, F> Stream for Handle<T, F>
where
    F: for<'de> serde::de::Deserialize<'de> + Unpin,
    T: tokio::io::AsyncWrite + tokio::io::AsyncRead + Unpin,
{
    type Item = io::Result<(String, F)>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let parse = |data: bytes::BytesMut,
                     topic_list: &HashMap<String, String>|
         -> io::Result<(String, F)> {
            let output = serde_json::from_slice::<jsonrpc_core::request::Notification>(&data)
                .expect("must parse to notification");
            let message = output
                .params
                .parse::<Message>()
                .expect("must parse to message");
            serde_json::from_str::<F>(&message.result)
                .map(|r| (topic_list.get(&message.subscription).cloned().unwrap(), r))
                .map_err(|_| io::ErrorKind::InvalidData.into())
        };

        if let Some(data) = self.pending_recv.pop_front() {
            return Poll::Ready(Some(parse(data, &self.topic_list)));
        }
        match self.inner.poll_next_unpin(cx) {
            Poll::Ready(Some(Ok(frame))) => Poll::Ready(Some(parse(frame, &self.topic_list))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
            Poll::Ready(Some(Err(err))) => Poll::Ready(Some(Err(err))),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct Message {
    result: String,
    subscription: String,
}

async fn subscribe<T: tokio::io::AsyncWrite + tokio::io::AsyncRead + Unpin>(
    io: &mut Framed<T, StreamCodec>,
    id: usize,
    topic: impl AsRef<str>,
    topic_list: &mut HashMap<String, String>,
    pending_recv: &mut VecDeque<bytes::BytesMut>,
) -> io::Result<()> {
    // telnet localhost 18114
    // > {"id": 2, "jsonrpc": "2.0", "method": "subscribe", "params": ["new_tip_header"]}
    // < {"jsonrpc":"2.0","result":0,"id":2}
    // < {"jsonrpc":"2.0","method":"subscribe","params":{"result":"...block header json...",
    // "subscription":0}}
    // < {"jsonrpc":"2.0","method":"subscribe","params":{"result":"...block header json...",
    // "subscription":0}}
    // < ...
    // > {"id": 2, "jsonrpc": "2.0", "method": "unsubscribe", "params": [0]}
    // < {"jsonrpc":"2.0","result":true,"id":2}
    let req_json = format!(
        r#"{{"id": {}, "jsonrpc": "2.0", "method": "subscribe", "params": ["{}"]}}"#,
        id,
        topic.as_ref()
    );

    io.send(req_json).await?;

    // loop util this subscribe success
    loop {
        let resp = io.next().await;
        let resp = resp.ok_or_else::<io::Error, _>(|| io::ErrorKind::BrokenPipe.into())??;
        match serde_json::from_slice::<jsonrpc_core::response::Output>(&resp) {
            Ok(output) => match output {
                jsonrpc_core::response::Output::Success(success) => {
                    let res = serde_json::from_value::<String>(success.result).unwrap();
                    topic_list.insert(res, topic.as_ref().to_owned());
                    break Ok(());
                }
                jsonrpc_core::response::Output::Failure(e) => {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, e.error))
                }
            },
            // must be Notification message
            Err(_) => pending_recv.push_back(resp),
        }
    }
}

pub async fn subscribe_new_tip_block<A: ToSocketAddrs>(
    addr: A,
) -> Result<Handle<TcpStream, ckb_jsonrpc_types::BlockView>, io::Error> {
    let c = Client::new(TcpStream::connect(addr).await?);
    c
        .subscribe_list::<ckb_jsonrpc_types::BlockView, _, _>(vec!["new_tip_block"].iter())
        .await
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "not a subscribe port, please set ckb `tcp_listen_address` to use subscribe rpc feature"))
}

pub async fn subscribe_new_tip_header<A: ToSocketAddrs>(
    addr: A,
) -> Result<Handle<TcpStream, ckb_jsonrpc_types::HeaderView>, io::Error> {
    let c = Client::new(TcpStream::connect(addr).await?);
    c
        .subscribe_list::<ckb_jsonrpc_types::HeaderView, _, _>(vec!["new_tip_header"].iter())
        .await
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "not a subscribe port, please set ckb `tcp_listen_address` to use subscribe rpc feature"))
}

pub async fn subscribe_new_transaction<A: ToSocketAddrs>(
    addr: A,
) -> Result<Handle<TcpStream, ckb_jsonrpc_types::PoolTransactionEntry>, io::Error> {
    let c = Client::new(TcpStream::connect(addr).await?);
    c
        .subscribe_list::<ckb_jsonrpc_types::PoolTransactionEntry, _, _>(vec!["new_transaction"].iter())
        .await
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "not a subscribe port, please set ckb `tcp_listen_address` to use subscribe rpc feature"))
}

pub async fn subscribe_proposed_transaction<A: ToSocketAddrs>(
    addr: A,
) -> Result<Handle<TcpStream, ckb_jsonrpc_types::PoolTransactionEntry>, io::Error> {
    let c = Client::new(TcpStream::connect(addr).await?);
    c
        .subscribe_list::<ckb_jsonrpc_types::PoolTransactionEntry, _, _>(vec!["proposed_transaction"].iter())
        .await
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "not a subscribe port, please set ckb `tcp_listen_address` to use subscribe rpc feature"))
}

pub async fn subscribe_rejected_transaction<A: ToSocketAddrs>(
    addr: A,
) -> Result<
    Handle<
        TcpStream,
        (
            ckb_jsonrpc_types::PoolTransactionEntry,
            ckb_jsonrpc_types::PoolTransactionReject,
        ),
    >,
    io::Error,
> {
    let c = Client::new(TcpStream::connect(addr).await?);
    c
        .subscribe_list::<(ckb_jsonrpc_types::PoolTransactionEntry,ckb_jsonrpc_types::PoolTransactionReject), _, _>(vec!["rejected_transaction"].iter())
        .await
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "not a subscribe port, please set ckb `tcp_listen_address` to use subscribe rpc feature"))
}
