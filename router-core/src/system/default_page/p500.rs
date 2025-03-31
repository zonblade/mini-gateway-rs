use crate::config::DEFAULT_PORT;
use super::p_base::run_error_page_server;

pub fn init() {
    run_error_page_server(
        DEFAULT_PORT.p500,
        500,
        "Internal Server Error",
        "Default 500 page"
    );
}