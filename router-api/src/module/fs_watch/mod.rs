use std::thread;
use tokio::runtime::Runtime;

pub mod constants;
pub mod file_id;
pub mod utils;
pub mod db_pool;
pub mod log_watcher;


/// Start watching the log file in a separate thread
///
/// # Returns
/// 
/// A JoinHandle to the spawned thread, which can be used to wait for the
/// thread to complete or to detach it.
/// 
/// # Example
/// 
/// ```
/// let _handle = start_log_watcher();
/// 
/// // Continue with other operations, the log watcher runs in the background
/// // The handle can be ignored if you don't need to join the thread later
/// ```
pub fn start_log_watcher() -> thread::JoinHandle<()> {
    let mut watcher = log_watcher::LogWatcher::new();
    
    // Spawn a dedicated thread for the log watcher
    thread::spawn(move || {
        // Create a new runtime for this thread
        let rt = Runtime::new().expect("Failed to create Tokio runtime");
        
        loop {
            // Execute the async task to completion using block_on
            let result = rt.block_on(async {
                watcher.tail_file().await
            });
            
            match result {
                Ok(_) => println!("Log watcher stopped unexpectedly"),
                Err(e) => println!("Error in log watcher: {}", e)
            }
            
            // Wait before retrying
            thread::sleep(constants::RETRY_INTERVAL);
        }
    })
}
