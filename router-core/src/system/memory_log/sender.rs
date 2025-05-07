use super::{log_gateway, log_proxy};



pub fn switcher(marker: &str, level:log::Level, message: &str) {
    
    let level = match level {
        log::Level::Error => super::LEVEL_ERROR,
        log::Level::Warn => super::LEVEL_WARN,
        log::Level::Info => super::LEVEL_INFO,
        log::Level::Debug => super::LEVEL_DEBUG,
        log::Level::Trace => super::LEVEL_TRACE,
    };
    
    match marker {
        "[PXY]" => unsafe {
            let res = log_proxy(level, message);
            if let Err(e) = res {
                eprintln!("[MEMLOG::PX] {}", e);
            }
        },
        "[GWX]" => unsafe {
            let res = log_gateway(level, message);
            if let Err(e) = res {
                eprintln!("[MEMLOG::GW] {}", e);
            }
        },
        _ => (),
    }
}