pub mod api;
pub mod models;

#[cfg(target_arch = "wasm32")]
pub mod frontend;

#[cfg(not(target_arch = "wasm32"))]
pub mod server;
