use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use elogbook::{config::AppConfig, configure_routes, state::AppState};
use std::io;

#[actix_web::main]
async fn main() -> io::Result<()> {
    let config = AppConfig::from_env().map_err(to_io_error)?;
    let state = web::Data::new(AppState::connect(&config).await.map_err(to_io_error)?);
    let server_addr = config.server_addr.clone();

    println!("elogbook backend listening on http://{server_addr}");

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
    })
    .bind(server_addr)?
    .run()
    .await
}

fn to_io_error(error: impl std::error::Error + Send + Sync + 'static) -> io::Error {
    io::Error::other(error)
}
