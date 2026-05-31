use crate::models::{Entry, EntryResponse};

use gloo_net::http::Request;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::FormData;

#[derive(Debug, Clone, PartialEq)]
pub struct ApiClient {
    base_url: &'static str,
    client_hash: String,
}

impl ApiClient {
    pub fn new(base_url: &'static str, client_hash: String) -> Self {
        Self {
            base_url,
            client_hash,
        }
    }

    pub fn entries_url(&self, limit: u32, offset: u64) -> String {
        self.url(&format!(
            "/get_entries?client={}&limit={limit}&offset={offset}",
            self.client_hash
        ))
    }

    pub fn add_entry_url(&self) -> String {
        self.url("/add_entry")
    }

    pub fn extend_download_lifetime_url(&self) -> String {
        self.url(&format!("/extend?client={}", self.client_hash))
    }

    pub fn download_url(&self, token: &str) -> String {
        self.url(&format!(
            "/download?client={}&token={token}",
            self.client_hash
        ))
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }
}

pub async fn fetch_entries(url: &str) -> Result<Vec<Entry>, String> {
    let response = Request::get(url)
        .send()
        .await
        .map_err(|error| format!("request failed: {error}"))?;

    if !response.ok() {
        return Err(format!("request failed with status {}", response.status()));
    }

    response
        .json::<Vec<EntryResponse>>()
        .await
        .map_err(|error| format!("failed to decode entries: {error}"))?
        .into_iter()
        .map(EntryResponse::into_entry)
        .collect()
}

pub async fn post_form(url: &str, form_data: FormData) -> Result<(), String> {
    let request_init = web_sys::RequestInit::new();
    request_init.set_method("POST");
    request_init.set_body(&JsValue::from(form_data));

    let request = web_sys::Request::new_with_str_and_init(url, &request_init)
        .map_err(|error| format!("failed to build request: {error:?}"))?;
    let window = web_sys::window().ok_or_else(|| "window is not available".to_string())?;
    let response = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|error| format!("request failed: {error:?}"))?;
    let response: web_sys::Response = response
        .dyn_into()
        .map_err(|_| "fetch returned a non-Response value".to_string())?;

    if response.ok() {
        Ok(())
    } else {
        Err(format!("request failed with status {}", response.status()))
    }
}
