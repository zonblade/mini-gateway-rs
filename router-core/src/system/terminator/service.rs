
use std::process::{id as pid, Command};

pub fn init(){
    let pid = pid();
    log::debug!(
        "Sample termination: sending SIGINT to process id: {}",
        pid
    );
    // Use the `kill` command to send SIGINT to self.
    let status = Command::new("kill")
        .arg("-SIGINT")
        .arg(pid.to_string())
        .status()
        .expect("Failed to execute kill command");
    log::debug!("Kill command exited with status: {}", status);
}