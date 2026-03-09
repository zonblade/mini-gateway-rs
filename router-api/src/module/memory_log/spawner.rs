use super::logging::{gateway, proxy};

pub fn spawn_all() {
    log::info!("Starting memory log spawners...");

    // // Spawn gateway listener thread
    std::thread::spawn(move || {
        log::info!("Gateway listener thread started");
        tokio::runtime::Runtime::new()
            .expect("Failed to create Tokio runtime")
            .block_on(gateway::listen());
    });

    // Spawn proxy listener thread
    std::thread::spawn(move || {
        log::info!("Proxy listener thread started");
        tokio::runtime::Runtime::new()
            .expect("Failed to create Tokio runtime")
            .block_on(proxy::listen());
    });

    log::info!("All memory log spawners started and detached");
}
