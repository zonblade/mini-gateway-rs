use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};

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

    log::debug!("Ctrl+X received: {}", result);  // Updated the message too

    result
}