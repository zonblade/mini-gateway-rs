use actix_web::{web, get};
use serde::Deserialize;

use super::logs_broadcast::{LogsBroadcaster, SubscriptionFilter};

#[derive(Deserialize)]
pub struct EventsParams {
    #[serde(rename = "type")]
    pub stats_type: String,  // default, bytes, status
    pub target: Option<String>,  // domain, proxy (defaults to domain)
    pub status: Option<i32>,     // only for type=status
}

// SSE endpoint for statistics events with parameters
#[get("/events")]
pub async fn logs_stream(
    query: web::Query<EventsParams>, 
    broadcaster: web::Data<LogsBroadcaster>
) -> actix_web::Result<actix_web_lab::sse::Sse<actix_web_lab::util::InfallibleStream<tokio_stream::wrappers::ReceiverStream<actix_web_lab::sse::Event>>>> {
    // Validate required parameters
    match query.stats_type.as_str() {
        "default" | "bytes" => {
            // No additional validation needed
        },
        "status" => {
            if query.status.is_none() {
                return Err(actix_web::error::ErrorBadRequest(
                    "status parameter required when type=status"
                ));
            }
        },
        _ => {
            return Err(actix_web::error::ErrorBadRequest(
                "type must be one of: default, bytes, status"
            ));
        }
    }

    // Create subscription filter from query parameters
    let filter = SubscriptionFilter {
        stats_type: query.stats_type.clone(),
        target: query.target.clone().unwrap_or_else(|| "domain".to_string()),
        status: query.status,
    };

    // Create new SSE client with subscription filter
    Ok(broadcaster.new_client(filter).await)
}