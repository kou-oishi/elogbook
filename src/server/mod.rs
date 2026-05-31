pub mod attachments;
pub mod config;
pub mod download;
pub mod error;
pub mod handlers;
pub mod models;
pub mod state;

pub use handlers::configure_routes;

use actix_cors::Cors;
use actix_files::Files;
use actix_web::{web, App, HttpServer};
use config::AppConfig;
use state::AppState;
use std::io;

pub async fn run() -> io::Result<()> {
    let config = AppConfig::from_env().map_err(to_io_error)?;
    let state = web::Data::new(AppState::connect(&config).await.map_err(to_io_error)?);
    let server_addr = config.server_addr.clone();
    let web_dir = config.web_dir.clone();

    println!("elogbook listening on http://{server_addr}");

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
                    .max_age(3600),
            )
            .app_data(state.clone())
            .configure(configure_routes)
            .service(Files::new("/", web_dir.clone()).index_file("index.html"))
    })
    .bind(server_addr)?
    .run()
    .await
}

fn to_io_error(error: impl std::error::Error + Send + Sync + 'static) -> io::Error {
    io::Error::other(error)
}
