pub mod types;
pub mod interfaces;
pub mod permissions;
pub mod packet_handler;

mod listener;

pub use listener::NetworkListener;

// Re-export the network_capture function
pub fn network_capture() {
    crate::system::netlisten::listener::network_capture()
}
