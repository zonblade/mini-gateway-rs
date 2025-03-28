use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

use tokio::{self, signal};

mod app;
mod config;
mod service;
mod system;

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    log::info!("Starting Router Core...");
    // service::registry::client();
    
    log::info!("Starting proxy server...");
    let active_state = Arc::new(AtomicBool::new(false)); // Removed extra semicolon and 'mut'

    {
        let running_clone = Arc::clone(&active_state);
        ctrlc::set_handler(move || {
            log::debug!("SIGINT received, shutting down servers...");
            running_clone.store(false, Ordering::SeqCst);
        })
        .expect("Error setting Ctrl-C handler");
    }

    loop {
        // add ctrl + e handler to terminate
        if system::terminator::cli::init(Duration::from_millis(0)) {
            log::debug!("Ctrl+X received, exiting...");
            break;
        }

        // detect if it's already on active state
        if !active_state.load(std::sync::atomic::Ordering::Relaxed) {
            // change to active state
            active_state.store(true, std::sync::atomic::Ordering::Relaxed);
            std::thread::spawn(|| {
                system::server::init();
            });
            continue;
        }

        sleep(Duration::from_millis(50));
    }
}
