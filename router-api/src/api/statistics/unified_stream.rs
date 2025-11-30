//! # Unified Statistics Stream
//!
//! SSE endpoint that streams combined gateway and proxy statistics every 1 second.

use super::unified_stats::{TargetStats, UnifiedStats};
use crate::module::temporary_log::{tlog_gateway, tlog_proxy};
use actix_web::rt::time::interval;
use actix_web_lab::{
    sse::{self, Sse},
    util::InfallibleStream,
};
use chrono::{Duration as ChronoDuration, Utc};
use futures_util::future;
use parking_lot::Mutex;
use std::{sync::Arc, time::Duration};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

/// Client connection for unified stats SSE
struct UnifiedClient {
    sender: mpsc::Sender<sse::Event>,
}

/// Broadcaster for unified statistics SSE
pub struct UnifiedStatsBroadcaster {
    inner: Mutex<Vec<UnifiedClient>>,
}

impl UnifiedStatsBroadcaster {
    /// Creates new broadcaster and spawns broadcast loop
    pub fn create() -> Arc<Self> {
        let this = Arc::new(Self {
            inner: Mutex::new(Vec::new()),
        });

        // Spawn ping/cleanup task
        Self::spawn_cleanup(Arc::clone(&this));
        // Spawn statistics broadcaster (1 second interval)
        Self::spawn_broadcaster(Arc::clone(&this));

        this
    }

    /// Spawn cleanup task to remove stale clients
    fn spawn_cleanup(this: Arc<Self>) {
        actix_web::rt::spawn(async move {
            let mut tick = interval(Duration::from_secs(5));
            loop {
                tick.tick().await;
                this.remove_stale_clients().await;
            }
        });
    }

    /// Remove clients that fail to receive ping
    async fn remove_stale_clients(&self) {
        let clients = self.inner.lock().iter().map(|c| c.sender.clone()).collect::<Vec<_>>();
        let mut ok_senders = Vec::new();

        for sender in clients {
            if sender.send(sse::Event::Comment("ping".into())).await.is_ok() {
                ok_senders.push(sender);
            }
        }

        let mut inner = self.inner.lock();
        inner.retain(|c| ok_senders.iter().any(|s| s.same_channel(&c.sender)));
    }

    /// Spawn broadcaster task (every 1 second)
    fn spawn_broadcaster(this: Arc<Self>) {
        actix_web::rt::spawn(async move {
            let mut tick = interval(Duration::from_secs(1));
            loop {
                tick.tick().await;
                this.broadcast_stats().await;
            }
        });
    }

    /// Broadcast current stats to all clients
    async fn broadcast_stats(&self) {
        let stats = get_current_stats();
        let json = match serde_json::to_string(&stats) {
            Ok(j) => j,
            Err(e) => {
                log::error!("Failed to serialize unified stats: {}", e);
                return;
            }
        };

        let clients = self.inner.lock().iter().map(|c| c.sender.clone()).collect::<Vec<_>>();
        let send_futures = clients.iter().map(|sender| sender.send(sse::Data::new(json.clone()).into()));
        let _ = future::join_all(send_futures).await;
    }

    /// Register new client
    pub async fn new_client(&self) -> Sse<InfallibleStream<ReceiverStream<sse::Event>>> {
        let (tx, rx) = mpsc::channel(10);

        // Send initial data
        let stats = get_current_stats();
        if let Ok(json) = serde_json::to_string(&stats) {
            let _ = tx.send(sse::Data::new(json).into()).await;
        }

        self.inner.lock().push(UnifiedClient { sender: tx });

        Sse::from_infallible_receiver(rx)
    }
}

/// Get current statistics for last 1 second window
fn get_current_stats() -> UnifiedStats {
    let now = Utc::now();
    let start = now - ChronoDuration::seconds(1);

    let gateway = aggregate_stats(true, start, now);
    let proxy = aggregate_stats(false, start, now);

    UnifiedStats {
        ts: now.to_rfc3339(),
        gateway,
        proxy,
    }
}

/// Aggregate stats for a target (gateway or proxy)
fn aggregate_stats(
    is_gateway: bool,
    start: chrono::DateTime<Utc>,
    end: chrono::DateTime<Utc>,
) -> TargetStats {
    let logs = if is_gateway {
        tlog_gateway::load_logs(start, end).unwrap_or_default()
    } else {
        tlog_proxy::load_logs(start, end).unwrap_or_default()
    };

    let mut stats = TargetStats::new();

    for log in logs {
        if log.conn_req == 1 {
            stats.req += 1;
        }
        if log.conn_res == 1 {
            stats.res += 1;
        }
        stats.bytes_in += log.bytes_in as i64;
        stats.bytes_out += log.bytes_out as i64;

        if log.status_code > 0 {
            *stats.status.entry(log.status_code.to_string()).or_insert(0) += 1;
        }
    }

    stats
}

/// SSE endpoint: GET /statistics/stream
#[actix_web::get("/stream")]
pub async fn stream(
    broadcaster: actix_web::web::Data<Arc<UnifiedStatsBroadcaster>>,
) -> Sse<InfallibleStream<ReceiverStream<sse::Event>>> {
    broadcaster.new_client().await
}
