use crate::app::{Model, Msg};

use wasm_bindgen::{prelude::Closure, JsCast, JsValue};

pub fn register_entry_callback(link: yew::html::Scope<Model>) {
    let callback = Closure::wrap(Box::new(move |content: JsValue, array: JsValue| {
        let content = content.as_string().unwrap_or_default();
        let files = js_sys::Array::from(&array);
        let attachments = files
            .iter()
            .filter_map(|file| file.dyn_into::<web_sys::File>().ok())
            .collect();

        link.send_message(Msg::AddEntry(content, attachments));
    }) as Box<dyn Fn(JsValue, JsValue)>);

    js_sys::Reflect::set(
        &js_sys::global(),
        &JsValue::from_str("send_add_entry"),
        callback.as_ref().unchecked_ref(),
    )
    .expect("register send_add_entry callback");

    callback.forget();
}

pub fn make_client_hash() -> String {
    let now = js_sys::Date::now() as u64;
    let random = (js_sys::Math::random() * u64::MAX as f64) as u64;
    format!("{now:x}{random:x}")
}

pub fn log_error(message: &str) {
    web_sys::console::error_1(&message.into());
}
