use std::sync::{Arc, Mutex, Once};
use crossbeam_channel::Receiver;
use crate::module::udp_log_fetcher::{MultiPortUdpLogFetcher, LogMessage};

/// Singleton instance of the multi-port UDP log fetcher
static mut MULTI_PORT_FETCHER: Option<Arc<Mutex<MultiPortUdpLogFetcher>>> = None;
static INIT: Once = Once::new();

/// Port configuration for different log channels
pub struct LogPorts {
    pub proxy_port: u16,
    pub gateway_port: u16,
    pub normal_port: u16,
}

impl Default for LogPorts {
    fn default() -> Self {
        LogPorts {
            proxy_port: 24401,
            gateway_port: 24402,
            normal_port: 24403,
        }
    }
}

/// Initialize the multi-port UDP log fetcher as a singleton
#[allow(static_mut_refs)]
pub fn initialize_udp_logger(ip: &str, ports: LogPorts) -> Result<(), String> {
    INIT.call_once(|| {
        let mut fetcher = MultiPortUdpLogFetcher::new();
        
        // Add the three main ports with ample queue size
        let _ = fetcher.add_port(ports.proxy_port, 1_000_000);
        let _ = fetcher.add_port(ports.gateway_port, 1_000_000);
        let _ = fetcher.add_port(ports.normal_port, 1_000_000);
        
        // Store the fetcher in the global static
        unsafe {
            MULTI_PORT_FETCHER = Some(Arc::new(Mutex::new(fetcher)));
        }
    });
    
    // Start the fetcher if it hasn't been started yet
    unsafe {
        if let Some(fetcher_arc) = &MULTI_PORT_FETCHER {
            let mut fetcher = fetcher_arc.lock().unwrap();
            return fetcher.start_all(ip);
        }
    }
    
    Err("Failed to initialize UDP logger".to_string())
}

/// Get a consumer for the proxy log channel
#[allow(static_mut_refs)]
pub fn get_proxy_log_consumer() -> Option<Receiver<LogMessage>> {
    unsafe {
        if let Some(fetcher_arc) = &MULTI_PORT_FETCHER {
            let fetcher = fetcher_arc.lock().unwrap();
            let ports = LogPorts::default();
            return fetcher.get_consumer_for_port(ports.proxy_port);
        }
    }
    None
}

/// Get a consumer for the gateway log channel
#[allow(static_mut_refs)]
pub fn get_gateway_log_consumer() -> Option<Receiver<LogMessage>> {
    unsafe {
        if let Some(fetcher_arc) = &MULTI_PORT_FETCHER {
            let fetcher = fetcher_arc.lock().unwrap();
            let ports = LogPorts::default();
            return fetcher.get_consumer_for_port(ports.gateway_port);
        }
    }
    None
}

/// Get a consumer for the normal log channel
#[allow(static_mut_refs)]
pub fn get_normal_log_consumer() -> Option<Receiver<LogMessage>> {
    unsafe {
        if let Some(fetcher_arc) = &MULTI_PORT_FETCHER {
            let fetcher = fetcher_arc.lock().unwrap();
            let ports = LogPorts::default();
            return fetcher.get_consumer_for_port(ports.normal_port);
        }
    }
    None
}

/// Shutdown all UDP listeners
#[allow(static_mut_refs)]
pub fn shutdown_udp_logger() {
    unsafe {
        if let Some(fetcher_arc) = &MULTI_PORT_FETCHER {
            let fetcher = fetcher_arc.lock().unwrap();
            fetcher.stop_all();
        }
    }
}