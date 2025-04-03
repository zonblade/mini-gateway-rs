use actix_web::{get, web, HttpResponse};
use super::super::config::{HtmlTemplate, HtmlAssets};

#[get("/css/{tail:.*}")]
pub async fn init(path: web::Path<String>) -> HttpResponse {
    let tail = path.into_inner();
    println!("Requested CSS file: {}", tail);
    
    // Get the embedded CSS assets
    if let Some(css_assets) = HtmlTemplate::GlobCSSAssets.xget::<Vec<HtmlAssets>>() {
        // Look for the requested file in our embedded assets
        for asset in css_assets {
            if asset.filename == tail {
                return HttpResponse::Ok()
                    .content_type("text/css")
                    .body(asset.content);
            }
        }
    }
    
    // File not found
    HttpResponse::NotFound().body(format!("CSS file not found: {}", tail))
}