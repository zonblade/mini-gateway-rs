use actix_web::{get, web, HttpResponse, Responder};
use chrono::{Duration, Utc};
use serde::Deserialize;

use crate::module::temporary_log::{tlog_gateway, tlog_proxy};


#[derive(Deserialize)]
struct Params {
    target: Option<String>,
}

#[get("/default")]
pub async fn init(query: web::Query<Params>) -> impl Responder {
    let end = Utc::now();
    let start = end - Duration::minutes(120);

    let result = {
        match &query.target {
            Some(str) => {
                match str.as_str() {
                    "proxy" => tlog_proxy::get_data_time_frame(start, end),
                    "domain" => tlog_gateway::get_data_time_frame(start, end),
                    _ => tlog_gateway::get_data_time_frame(start, end)
                }
            }
            None => tlog_gateway::get_data_time_frame(start, end),
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
