use actix_web::{get, web, HttpResponse, Responder};
use serde::Deserialize;

use crate::module::temporary_log::{tlog_gateway, tlog_proxy, BytesMetric};

#[derive(Deserialize)]
struct Params {
    target: Option<String>,
}

#[get("/bytes")]
pub async fn init(query: web::Query<Params>) -> impl Responder {
    let end = chrono::Utc::now();
    // start 30 minutes before
    let start = end - chrono::Duration::minutes(120);

    let result = {
        match &query.target {
            Some(str) => {
                match str.as_str() {
                    "proxy" => tlog_proxy::get_bytes_io_frame(start, end, BytesMetric::BytesTotal),
                    "domain" => tlog_gateway::get_bytes_io_frame(start, end, BytesMetric::BytesTotal),
                    _ => tlog_gateway::get_bytes_io_frame(start, end, BytesMetric::BytesTotal)
                }
            }
            None => tlog_gateway::get_bytes_io_frame(start, end, BytesMetric::BytesTotal),
        }
    };

    let result = match result {
        Ok(data) => data,
        Err(e) => {
            log::error!("Error fetching domains for proxy {}", e);
            vec![]
        }
    };

    HttpResponse::Ok().json(result)
}
