use async_trait::async_trait;
use log::debug;

use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::select;

use pingora::apps::ServerApp;
use pingora::connectors::TransportConnector;
use pingora::protocols::Stream;
use pingora::server::ShutdownWatch;
use pingora::upstreams::peer::BasicPeer;

/// `ProxyApp` is an application that facilitates proxying data between a downstream
/// server session and an upstream client session. It uses a `TransportConnector`
/// to establish connections to the upstream peer and handles bidirectional data
/// transfer between the two streams.
pub struct ProxyApp {
    /// Connector used to establish connections to the upstream peer.
    client_connector: TransportConnector,
    /// The upstream peer to which the proxy will forward data.
    proxy_to: BasicPeer,
}

/// Events representing data read from either the downstream or upstream streams.
enum DuplexEvent {
    /// Data read from the downstream stream.
    DownstreamRead(usize),
    /// Data read from the upstream stream.
    UpstreamRead(usize),
}

impl ProxyApp {
    /// Creates a new instance of `ProxyApp` with the specified upstream peer.
    ///
    /// # Arguments
    ///
    /// * `proxy_to` - The upstream peer to which the proxy will forward data.
    ///
    /// # Returns
    ///
    /// A new `ProxyApp` instance.
    pub fn new(proxy_to: BasicPeer) -> Self {
        ProxyApp {
            client_connector: TransportConnector::new(None),
            proxy_to,
        }
    }

    /// Handles bidirectional data transfer between the downstream server session
    /// and the upstream client session.
    ///
    /// This method reads data from one stream and writes it to the other, ensuring
    /// that data flows seamlessly between the two endpoints. It terminates when
    /// either stream is closed.
    ///
    /// # Arguments
    ///
    /// * `server_session` - The downstream server session.
    /// * `client_session` - The upstream client session.
    async fn duplex(&self, mut server_session: Stream, mut client_session: Stream) {
        // Buffers for reading data from the streams.
        let mut upstream_buf = [0; 1024];
        let mut downstream_buf = [0; 1024];
        loop {
            // Concurrently read from both streams.
            let downstream_read = server_session.read(&mut upstream_buf);
            let upstream_read = client_session.read(&mut downstream_buf);
            let event: DuplexEvent;
            select! {
                n = downstream_read => event
                    = DuplexEvent::DownstreamRead(n.unwrap()),
                n = upstream_read => event
                    = DuplexEvent::UpstreamRead(n.unwrap()),
            }
            match event {
                DuplexEvent::DownstreamRead(0) => {
                    debug!("downstream session closing");
                    return;
                }
                DuplexEvent::UpstreamRead(0) => {
                    debug!("upstream session closing");
                    return;
                }
                DuplexEvent::DownstreamRead(n) => {
                    // Write data from downstream to upstream.
                    client_session.write_all(&upstream_buf[0..n]).await.unwrap();
                    client_session.flush().await.unwrap();
                }
                DuplexEvent::UpstreamRead(n) => {
                    // Write data from upstream to downstream.
                    server_session
                        .write_all(&downstream_buf[0..n])
                        .await
                        .unwrap();
                    server_session.flush().await.unwrap();
                }
            }
        }
    }
}

#[async_trait]
impl ServerApp for ProxyApp {
    /// Processes a new incoming connection from the downstream server.
    ///
    /// This method establishes a connection to the upstream peer and starts
    /// the bidirectional data transfer between the downstream and upstream
    /// streams. If the connection to the upstream peer fails, it logs the
    /// error and terminates the session.
    ///
    /// # Arguments
    ///
    /// * `io` - The downstream server stream.
    /// * `_shutdown` - A shutdown watcher to monitor for server shutdown signals.
    ///
    /// # Returns
    ///
    /// Always returns `None` as the proxy does not reuse the downstream stream.
    async fn process_new(
        self: &Arc<Self>,
        io: Stream,
        _shutdown: &ShutdownWatch,
    ) -> Option<Stream> {
        // Attempt to establish a connection to the upstream peer.
        let client_session = self.client_connector.new_stream(&self.proxy_to).await;

        match client_session {
            Ok(client_session) => {
                // Start bidirectional data transfer.
                self.duplex(io, client_session).await;
                None
            }
            Err(e) => {
                // Log the error if the connection fails.
                debug!("Failed to create client session: {}", e);
                None
            }
        }
    }
}