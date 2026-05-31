#[cfg(not(target_arch = "wasm32"))]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    elogbook::server::run().await
}

#[cfg(target_arch = "wasm32")]
fn main() {
    elogbook::frontend::run();
}
