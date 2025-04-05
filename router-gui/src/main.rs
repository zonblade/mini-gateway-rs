use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use actix_cors::Cors;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};

mod config;

#[get("/{tail:.*}")]
async fn omnicontrol(
    tail: web::Path<String>,data: web::Data<Arc<RwLock<HashMap<String, Vec<u8>>>>>, // Updated type
) -> impl Responder {
    // if it's only empty assign as index.html
    let tail = if tail.is_empty() {
        "index.html".to_string()
    } else {
        tail.into_inner()
    };

    // Get the assets from the shared state
    let assets = data.read().unwrap();
    let asset = assets.get(&format!("/{}", tail));
    if asset.is_none() {
        return HttpResponse::NotFound().body(format!("File not found: {}", tail));
    }

    // serve the file to the client
    let asset = asset.unwrap();
    
    // Determine content type based on file extension
    let content_type = match tail.split('.').last() {
        Some("html") => "text/html",
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("json") => "application/json",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("ttf") => "font/ttf",
        Some("otf") => "font/otf",
        Some("eot") => "application/vnd.ms-fontobject",
        Some("txt") => "text/plain",
        Some("xml") => "application/xml",
        Some("pdf") => "application/pdf",
        _ => "application/octet-stream", // Default content type for unknown types
    };
    
    // Return the asset with the appropriate content type
    HttpResponse::Ok()
        .content_type(content_type)
        .body(asset.clone())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let assets = config::init();
    let shared_assets = Arc::new(RwLock::new(assets));

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .app_data(web::Data::new(shared_assets.clone()))
            .service(omnicontrol)
    })
    .bind(("0.0.0.0", 24041))?
    .run()
    .await
}
