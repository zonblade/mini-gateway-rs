use std::{sync::Arc, time::Duration};

use actix_web::rt::time::interval;
use actix_web_lab::{
    sse::{self, Sse},
    util::InfallibleStream,
};
use futures_util::future;
use parking_lot::Mutex;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use serde::Serialize;

use crate::module::temporary_log::{LogCaptureTimeframe, tlog_gateway, tlog_proxy, BytesMetric};
use chrono::{DateTime, Utc, Duration as ChronoDuration};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SubscriptionFilter {
    pub stats_type: String,  // default, bytes, status
    pub target: String,      // domain, proxy  
    pub status: Option<i32>, // only for type=status
}

#[derive(Debug, Clone)]
struct ClientSubscription {
    sender: mpsc::Sender<sse::Event>,
    filter: SubscriptionFilter,
}

#[derive(Serialize)]
struct StatisticsEvent {
    #[serde(rename = "type")]
    event_type: String,
    data: LogCaptureTimeframe,
}

pub struct LogsBroadcaster {
    inner: Mutex<BroadcasterInner>,
}

#[derive(Debug, Clone, Default)]
struct BroadcasterInner {
    clients: Vec<ClientSubscription>,
}

impl LogsBroadcaster {
    /// Constructs new broadcaster and spawns ping loop and statistics broadcasting.
    pub fn create() -> Arc<Self> {
        let this = Arc::new(LogsBroadcaster {
            inner: Mutex::new(BroadcasterInner::default()),
        });

        LogsBroadcaster::spawn_proccess(Arc::clone(&this));
        LogsBroadcaster::spawn_statistics_broadcaster(Arc::clone(&this));

        this
    }

    /// Pings clients every 10 seconds to see if they are alive and remove them from the broadcast
    /// list if not.
    fn spawn_proccess(this: Arc<Self>) {
        actix_web::rt::spawn(async move {
            let mut interval = interval(Duration::from_secs(5));

            loop {



                interval.tick().await;
                this.remove_stale_clients().await;
            }
        });
    }

    /// Removes all non-responsive clients from broadcast list.
    async fn remove_stale_clients(&self) {
        let clients = self.inner.lock().clients.clone();

        let mut ok_clients = Vec::new();

        for client in clients {
            if client.sender
                .send(sse::Event::Comment("ping".into()))
                .await
                .is_ok()
            {
                ok_clients.push(client.clone());
            }
        }

        self.inner.lock().clients = ok_clients;
    }

    /// Registers client with broadcaster, returning an SSE response body.
    pub async fn new_client(&self, filter: SubscriptionFilter) -> Sse<InfallibleStream<ReceiverStream<sse::Event>>> {
        let (tx, rx) = mpsc::channel(10);

        tx.send(sse::Data::new("connected").into()).await.unwrap();

        self.inner.lock().clients.push(ClientSubscription {
            sender: tx,
            filter,
        });

        Sse::from_infallible_receiver(rx)
    }

    /// Broadcasts `msg` to all clients.
    #[allow(dead_code)]
    pub async fn broadcast(&self, msg: &str) {
        let clients = self.inner.lock().clients.clone();

        let send_futures = clients
            .iter()
            .map(|client| client.sender.send(sse::Data::new(msg.to_string()).into()));

        // try to send to all clients, ignoring failures
        // disconnected clients will get swept up by `remove_stale_clients`
        let _ = future::join_all(send_futures).await;
    }

    /// Broadcasts statistics data to clients matching the filter.
    pub async fn broadcast_statistics(&self, data: LogCaptureTimeframe, filter: &SubscriptionFilter) {
        let clients = self.inner.lock().clients.clone();

        let event = StatisticsEvent {
            event_type: "statistics_update".to_string(),
            data,
        };

        // Serialize to JSON
        let json_data = match serde_json::to_string(&event) {
            Ok(json) => json,
            Err(e) => {
                log::error!("Failed to serialize statistics event: {}", e);
                return;
            }
        };

        let matching_clients: Vec<_> = clients
            .iter()
            .filter(|client| client.filter == *filter)
            .collect();

        let send_futures = matching_clients
            .iter()
            .map(|client| client.sender.send(sse::Data::new(json_data.clone()).into()));

        // try to send to all matching clients, ignoring failures
        // disconnected clients will get swept up by `remove_stale_clients`
        let _ = future::join_all(send_futures).await;
    }

    /// Spawns background task to broadcast statistics every 15 seconds
    fn spawn_statistics_broadcaster(this: Arc<Self>) {
        actix_web::rt::spawn(async move {
            let mut interval = interval(Duration::from_secs(15));
            let mut last_broadcast_time: Option<DateTime<Utc>> = None;

            loop {
                interval.tick().await;
                
                let now = Utc::now();
                
                // Calculate the most recent completed 15-second interval
                let current_interval_start = DateTime::from_timestamp(
                    (now.timestamp() / 15) * 15, 0
                ).unwrap_or(now);
                
                // Only broadcast if this is a new interval
                if last_broadcast_time.map_or(true, |last| current_interval_start > last) {
                    this.broadcast_current_intervals(current_interval_start).await;
                    last_broadcast_time = Some(current_interval_start);
                }
            }
        });
    }

    /// Broadcasts current statistics intervals to subscribed clients
    async fn broadcast_current_intervals(&self, interval_start: DateTime<Utc>) {
        let clients = self.inner.lock().clients.clone();
        
        // Get unique subscription filters to avoid duplicate work
        let mut unique_filters = HashSet::new();
        for client in &clients {
            unique_filters.insert(client.filter.clone());
        }

        let interval_end = interval_start + ChronoDuration::seconds(15);

        // Broadcast for each unique filter
        for filter in unique_filters {
            if let Some(latest_data) = self.get_latest_statistics_data(&filter, interval_start, interval_end).await {
                self.broadcast_statistics(latest_data, &filter).await;
            }
        }
    }

    /// Gets the latest statistics data for a specific filter and time interval
    async fn get_latest_statistics_data(
        &self, 
        filter: &SubscriptionFilter, 
        start: DateTime<Utc>, 
        end: DateTime<Utc>
    ) -> Option<LogCaptureTimeframe> {
        let result = match filter.stats_type.as_str() {
            "default" => {
                match filter.target.as_str() {
                    "proxy" => tlog_proxy::get_data_time_frame(start, end),
                    _ => tlog_gateway::get_data_time_frame(start, end),
                }
            },
            "bytes" => {
                match filter.target.as_str() {
                    "proxy" => tlog_proxy::get_bytes_io_frame(start, end, BytesMetric::BytesTotal),
                    _ => tlog_gateway::get_bytes_io_frame(start, end, BytesMetric::BytesTotal),
                }
            },
            "status" => {
                if let Some(status_code) = filter.status {
                    match filter.target.as_str() {
                        "proxy" => tlog_proxy::get_data_time_frame_by_status_code(start, end, status_code),
                        _ => tlog_gateway::get_data_time_frame_by_status_code(start, end, status_code),
                    }
                } else {
                    return None;
                }
            },
            _ => return None,
        };

        match result {
            Ok(mut data) => {
                // Return the latest (most recent) interval data
                data.sort_by(|a, b| b.date_time.cmp(&a.date_time));
                data.first().cloned()
            },
            Err(e) => {
                log::warn!("Failed to get statistics data for filter {:?}: {}", filter, e);
                None
            }
        }
    }
}