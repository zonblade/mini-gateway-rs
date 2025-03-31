use crate::config::DEFAULT_PORT;

use super::p_base::run_error_page_server;

pub fn init() {
    run_error_page_server(
        DEFAULT_PORT.tls_honeypot,
        403,
        "Forbidden",
        "Default TLS Forbidden page"
    );
}