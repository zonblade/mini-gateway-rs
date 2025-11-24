use std::sync::mpsc;
use std::thread;
use std::future::Future;

pub struct BrokerClient;

impl BrokerClient {
    pub fn new() -> Self {
        BrokerClient
    }

    pub fn spawn<T, F>(&self, handler: F) -> mpsc::Sender<T>
    where
        F: Fn(T) + Send + 'static,
        T: Send + 'static,
    {
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            while let Ok(msg) = rx.recv() {
                handler(msg);
            }
        });

        tx
    }

    pub fn spawn_async<T, F, Fut>(&self, handler: F) -> tokio::sync::mpsc::UnboundedSender<T>
    where
        F: Fn(T) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
        T: Send + 'static,
    {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                handler(msg).await;
            }
        });

        tx
    }
}
