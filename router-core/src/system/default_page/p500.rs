use super::p_base::run_error_page_server;
use crate::config::DEFAULT_PORT;

pub fn init() {
    run_error_page_server(
        DEFAULT_PORT.p500,
        500,
        "Internal Server Error",
        "Default 500 page",
    );
}
