use actix_web::{App, HttpServer, web};
use actix_cors::Cors;

mod config;
mod pages;
mod assets;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    config::init();

    HttpServer::new(|| {
        App::new()
            .wrap(Cors::permissive())
            .service(pages::home_handler)
            .service(pages::users_handler)
            .service(
                web::scope("/settings")
                    .service(pages::proxy_handler)
                    .service(pages::gwnode_handler)
                    .service(pages::gateway_handler)
            )
            .service(
                web::scope("/assets")
                    .service(assets::script::init)
                    .service(assets::css::init)
            )
    })
    .bind(("0.0.0.0", 24041))?
    .run()
    .await
}
