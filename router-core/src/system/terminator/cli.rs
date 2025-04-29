//! # Terminator CLI Module
//! 
//! This module provides functionality to detect keyboard interrupt signals
//! that can terminate the application's execution. It specifically handles
//! the detection of the Ctrl+X key combination as an interrupt signal.
//!
//! The module works by temporarily enabling raw terminal mode (if not already enabled),
//! polling for keyboard events, and then restoring the terminal to its original state.
//!
//! ## Usage
//!
//! ```rust
//! use std::time::Duration;
//! use router_core::system::terminator::cli;
//!
//! // Check if Ctrl+X was pressed within 100ms
//! let should_terminate = cli::init(Duration::from_millis(100));
//! if should_terminate {
//!     println!("Application terminating due to keyboard interrupt");
//! }
//! ```

use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};

/// Detects if the Ctrl+X key combination was pressed within the specified duration.
///
/// This function temporarily switches the terminal to raw mode (if not already in it)
/// to capture raw keyboard events. It then polls for keyboard events for the specified
/// duration, checking specifically for the Ctrl+X combination which serves as the 
/// application's interrupt signal.
///
/// # Arguments
///
/// * `duration` - The maximum time to wait for a keyboard event, as a `Duration`.
///   This controls how long the function will block waiting for input.
///
/// # Returns
///
/// * `bool` - Returns `true` if the Ctrl+X combination was detected within the
///   specified duration, otherwise returns `false`.
///
/// # Implementation Details
///
/// 1. The function first checks if the terminal is already in raw mode
/// 2. If not, it temporarily enables raw mode for direct keyboard input
/// 3. It then polls for keyboard events for the specified duration
/// 4. If an event occurs and it's a Ctrl+X key combination, it returns `true`
/// 5. Before returning, the function restores the terminal to its previous mode
/// 6. The function handles errors gracefully, defaulting to `false` if events cannot be read
///
/// # Behavior Notes
///
/// * The function will not modify terminal mode permanently - if raw mode was disabled
///   before calling this function, it will be disabled again before returning
/// * Any error in polling or reading events results in a `false` return value
/// * The function logs debug information about whether Ctrl+X was detected
///
/// # Example
///
/// ```rust
/// // Check every 100ms for the Ctrl+X interrupt signal
/// if cli::init(Duration::from_millis(100)) {
///     println!("Received termination signal");
///     // Perform cleanup and exit
/// }
/// ```
pub fn init(duration: Duration) -> bool {
    // Enable raw mode temporarily if not already enabled
    let raw_mode_enabled = crossterm::terminal::is_raw_mode_enabled().unwrap_or(false);
    if !raw_mode_enabled {
        let _ = crossterm::terminal::enable_raw_mode();
    }

    let result = if event::poll(duration).unwrap_or(false) {
        if let Ok(Event::Key(KeyEvent { 
            code: KeyCode::Char('x'),  // Changed from 'c' to 'x'
            modifiers,
            ..
        })) = event::read() {
            modifiers.contains(KeyModifiers::CONTROL)  // Only check for CONTROL, not SHIFT
        } else {
            false
        }
    } else {
        false
    };

    // Disable raw mode if we enabled it
    if !raw_mode_enabled {
        let _ = crossterm::terminal::disable_raw_mode();
    }

    result
}