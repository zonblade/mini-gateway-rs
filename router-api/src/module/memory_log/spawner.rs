use super::logging::{gateway, proxy};

pub fn spawn_all() {
    log::info!("Starting memory log spawners...");
    
    // Spawn gateway listener thread
    std::thread::spawn(move || {
        log::info!("Gateway listener thread started");
        gateway::listen();
    });

    // Spawn proxy listener thread
    std::thread::spawn(move || {
        log::info!("Proxy listener thread started");
        proxy::listen();
    });
    
    
    log::info!("All memory log spawners started and detached");
}