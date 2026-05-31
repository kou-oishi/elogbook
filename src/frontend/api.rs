use crate::{
    api::{
        CLIENT_QUERY, DOWNLOAD_PATH, ENTRIES_PATH, EXTEND_DOWNLOAD_LIFETIME_PATH, LIMIT_QUERY,
        OFFSET_QUERY, TOKEN_QUERY,
    },
    frontend::models::Entry,
    models::EntryResponse,
};
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
            "{}?{}={}&{}={limit}&{}={offset}",
            ENTRIES_PATH, CLIENT_QUERY, self.client_hash, LIMIT_QUERY, OFFSET_QUERY
        ))
    }

    pub fn add_entry_url(&self) -> String {
        self.url(ENTRIES_PATH)
    }

    pub fn extend_download_lifetime_url(&self) -> String {
        self.url(&format!(
            "{}?{}={}",
            EXTEND_DOWNLOAD_LIFETIME_PATH, CLIENT_QUERY, self.client_hash
        ))
    }

    pub fn download_url(&self, token: &str) -> String {
        self.url(&format!(
            "{}?{}={}&{}={token}",
            DOWNLOAD_PATH, CLIENT_QUERY, self.client_hash, TOKEN_QUERY
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
        .map(Entry::try_from)
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
