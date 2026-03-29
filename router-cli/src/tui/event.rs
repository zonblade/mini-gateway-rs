use crate::tui::app::UnifiedStats;
use crossterm::event::{Event as CrosstermEvent, EventStream, KeyEvent};
use futures_util::{StreamExt, TryStreamExt};
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};

pub enum Event {
    Key(KeyEvent),
    Tick,
    SseData(Box<UnifiedStats>),
    SseBatch(Vec<UnifiedStats>),
    SseConnected,
    SseDisconnected(String),
}

pub struct EventHandler {
    rx: mpsc::UnboundedReceiver<Event>,
}

impl EventHandler {
    pub fn new(sse_url: String, auth_token: String) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        // Keyboard events
        let tx_key = tx.clone();
        tokio::spawn(async move {
            let mut reader = EventStream::new();
            loop {
                match reader.next().await {
                    Some(Ok(CrosstermEvent::Key(key))) => {
                        if tx_key.send(Event::Key(key)).is_err() {
                            break;
                        }
                    }
                    Some(Ok(_)) => {} // Ignore resize, mouse, etc.
                    Some(Err(_)) => break,
                    None => break,
                }
            }
        });

        // Tick timer
        let tx_tick = tx.clone();
        tokio::spawn(async move {
            let mut tick = interval(Duration::from_millis(250));
            loop {
                tick.tick().await;
                if tx_tick.send(Event::Tick).is_err() {
                    break;
                }
            }
        });

        // SSE reader
        let tx_sse = tx.clone();
        tokio::spawn(async move {
            loop {
                match connect_sse(&sse_url, &auth_token, &tx_sse).await {
                    Ok(()) => {} // Stream ended normally
                    Err(e) => {
                        let _ = tx_sse.send(Event::SseDisconnected(e.to_string()));
                    }
                }
                // Reconnect after 5 seconds
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        });

        Self { rx }
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.rx.recv().await
    }
}

async fn connect_sse(
    url: &str,
    token: &str,
    tx: &mpsc::UnboundedSender<Event>,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let resp = client
        .get(url)
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(format!("SSE connection failed: {}", resp.status()).into());
    }

    let _ = tx.send(Event::SseConnected);

    let mut stream = resp.bytes_stream();
    let mut buffer = String::new();

    while let Some(chunk) = stream.try_next().await? {
        let text = String::from_utf8_lossy(&chunk);
        buffer.push_str(&text);

        // Process complete SSE messages
        while let Some(pos) = buffer.find("\n\n") {
            let message = buffer[..pos].to_string();
            buffer = buffer[pos + 2..].to_string();

            if let Some(data) = extract_sse_data(&message) {
                // Try parsing as array first (history batch)
                if let Ok(batch) = serde_json::from_str::<Vec<UnifiedStats>>(data) {
                    let _ = tx.send(Event::SseBatch(batch));
                } else if let Ok(stats) = serde_json::from_str::<UnifiedStats>(data) {
                    let _ = tx.send(Event::SseData(Box::new(stats)));
                }
            }
        }
    }

    Ok(())
}

fn extract_sse_data(message: &str) -> Option<&str> {
    for line in message.lines() {
        if let Some(data) = line.strip_prefix("data:") {
            return Some(data.trim());
        }
    }
    None
}
