use actix_web::{get, web, HttpResponse};
use super::super::config::{HtmlTemplate, HtmlAssets};

#[get("/js/{tail:.*}")]
pub async fn init(path: web::Path<String>) -> HttpResponse {
    let tail = path.into_inner();
    println!("Requested JavaScript file: {}", tail);
    
    // Get the embedded JS assets
    if let Some(js_assets) = HtmlTemplate::GlobJSAssets.xget::<Vec<HtmlAssets>>() {
        // Look for the requested file in our embedded assets
        for asset in js_assets {
            if asset.filename == tail {
                return HttpResponse::Ok()
                    .content_type("application/javascript")
                    .body(asset.content);
            }
        }
    }
    
    // File not found
    HttpResponse::NotFound().body(format!("JavaScript file not found: {}", tail))
}