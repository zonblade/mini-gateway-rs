#[cfg(target_os = "macos")]
pub fn get_default_log_dir() -> String {
    String::from("/tmp/gwrs/log/core.proxy.log")
}

#[cfg(target_os = "linux")]
pub fn get_default_log_dir() -> String {
    String::from("/tmp/gwrs/log/core.proxy.log")
}

#[cfg(target_os = "windows")]
pub fn get_default_log_dir() -> String {
    String::from("C:\\ProgramData\\gwrs\\core.proxy.log")
} 