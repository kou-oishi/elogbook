mod api;
mod app;
mod components;
mod config;
mod js_bridge;
mod models;
mod render;

pub fn run() {
    yew::Renderer::<app::Model>::new().render();
}
