use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

/// Represents the service manager type
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ServiceManagerType {
    Systemd,
    OpenRC,
    None,
}

/// Information about a service
#[derive(Debug, Clone)]
pub struct ServiceInfo {
    pub name: String,
    pub active: bool,
}

/// Checks if the program is running inside a service manager
pub fn is_running_as_service() -> bool {
    #[cfg(not(target_os = "windows"))]
    {
        // Check for systemd
        if let Ok(ppid) = env::var("PPID") {
            if let Ok(cmdline) = fs::read_to_string(format!("/proc/{}/cmdline", ppid)) {
                if cmdline.contains("systemd") {
                    return true;
                }
            }
        }

        // Check for OpenRC
        if env::var("RC_SVCNAME").is_ok() || Path::new("/run/openrc").exists() {
            return true;
        }

        // Check if parent is a service manager
        if let Ok(output) = Command::new("ps").args(["-o", "ppid=", "-p", &std::process::id().to_string()]).output() {
            if let Ok(ppid_str) = String::from_utf8(output.stdout) {
                if let Ok(ppid) = ppid_str.trim().parse::<u32>() {
                    if let Ok(output) = Command::new("ps").args(["-o", "comm=", "-p", &ppid.to_string()]).output() {
                        if let Ok(comm) = String::from_utf8(output.stdout) {
                            let comm = comm.trim();
                            if comm == "systemd" || comm == "init" || comm == "openrc-init" || comm == "openrc" {
                                return true;
                            }
                        }
                    }
                }
            }
        }

        false
    }

    #[cfg(target_os = "windows")]
    {
        false // Windows is excluded as per requirements
    }
}

/// Detects which service manager is being used
pub fn detect_service_manager() -> ServiceManagerType {
    #[cfg(not(target_os = "windows"))]
    {
        // Check for systemd
        if Path::new("/run/systemd/system").exists() || Path::new("/sys/fs/cgroup/systemd").exists() {
            return ServiceManagerType::Systemd;
        }

        // Check for OpenRC
        if env::var("RC_SVCNAME").is_ok() || Path::new("/run/openrc").exists() {
            return ServiceManagerType::OpenRC;
        }

        ServiceManagerType::None
    }

    #[cfg(target_os = "windows")]
    {
        ServiceManagerType::None
    }
}

/// Get list of gwrs-* services
pub fn get_gwrs_services() -> Vec<ServiceInfo> {
    let mut services = Vec::new();
    
    #[cfg(not(target_os = "windows"))]
    {
        match detect_service_manager() {
            ServiceManagerType::Systemd => {
                // List all services with systemctl
                if let Ok(output) = Command::new("systemctl").args(["list-units", "--type=service", "--all"]).output() {
                    if let Ok(output_str) = String::from_utf8(output.stdout) {
                        for line in output_str.lines() {
                            if line.contains("gwrs-") {
                                let parts: Vec<&str> = line.split_whitespace().collect();
                                if parts.len() >= 3 {
                                    let name = parts[0].trim_end_matches(".service").to_string();
                                    let active = parts[2] == "active";
                                    services.push(ServiceInfo { name, active });
                                }
                            }
                        }
                    }
                }
            },
            ServiceManagerType::OpenRC => {
                // List services with rc-status
                if let Ok(output) = Command::new("rc-status").args(["--all"]).output() {
                    if let Ok(output_str) = String::from_utf8(output.stdout) {
                        for line in output_str.lines() {
                            if line.contains("gwrs-") {
                                let parts: Vec<&str> = line.split_whitespace().collect();
                                if parts.len() >= 2 {
                                    let name = parts[0].trim().to_string();
                                    let status = parts[1].trim().to_lowercase();
                                    let active = status == "started" || status == "running";
                                    services.push(ServiceInfo { name, active });
                                }
                            }
                        }
                    }
                }

                // Alternative check with rc-service for OpenRC
                if let Ok(output) = Command::new("rc-service").args(["-l"]).output() {
                    if let Ok(output_str) = String::from_utf8(output.stdout) {
                        for line in output_str.lines() {
                            let service_name = line.trim();
                            if service_name.starts_with("gwrs-") {
                                // Check if this service is already in our list
                                if !services.iter().any(|s| s.name == service_name) {
                                    // Check if service is running
                                    let active = if let Ok(status) = Command::new("rc-service")
                                        .args([service_name, "status"])
                                        .output() {
                                        String::from_utf8_lossy(&status.stdout).contains("started") ||
                                        String::from_utf8_lossy(&status.stdout).contains("running")
                                    } else {
                                        false
                                    };
                                    
                                    services.push(ServiceInfo {
                                        name: service_name.to_string(),
                                        active,
                                    });
                                }
                            }
                        }
                    }
                }
            },
            ServiceManagerType::None => {
                // No service manager detected, return empty vector
            }
        }
    }

    services
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_manager_detection() {
        let manager = detect_service_manager();
        println!("Detected service manager: {:?}", manager);
    }

    #[test]
    fn test_running_as_service() {
        let is_service = is_running_as_service();
        println!("Running as service: {}", is_service);
    }

    #[test]
    fn test_gwrs_services() {
        let services = get_gwrs_services();
        println!("GWRS services found: {:?}", services);
    }
}
