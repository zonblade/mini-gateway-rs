use std::process::Command;

pub struct PermissionChecker;

impl PermissionChecker {
    /// Check if the current process has the necessary permissions for packet capture
    pub fn has_capture_permissions() -> bool {
        // Check if running as root (effective user ID is 0)
        if unsafe { libc::geteuid() } == 0 {
            return true;
        }

        // Check if the binary has CAP_NET_RAW capability
        let output = Command::new("getcap")
            .arg(std::env::current_exe().unwrap_or_default().to_string_lossy().into_owned())
            .output();
        
        match output {
            Ok(output) => {
                let cap_output = String::from_utf8_lossy(&output.stdout);
                cap_output.contains("cap_net_raw")
            }
            Err(_) => false,
        }
    }
}
