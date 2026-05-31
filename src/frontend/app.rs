use crate::{
    api::{FORM_CONTENT_FIELD, FORM_FILE_FIELD, FORM_NAME_FIELD},
    frontend::{
        api::{fetch_entries, post_form, ApiClient},
        components::{EditorHost, EntryList, Header},
        config::{API_BASE, DEFAULT_LIMIT},
        js_bridge::{log_error, make_client_hash, register_entry_callback},
        models::Entry,
    },
};

use gloo_net::http::Request;
use gloo_timers::callback::{Interval, Timeout};
use wasm_bindgen::{prelude::Closure, JsCast};
use wasm_bindgen_futures::spawn_local;
use web_sys::{FormData, HtmlElement};
use yew::prelude::*;

pub enum Msg {
    AddEntry(String, Vec<web_sys::File>),
    GetEntries {
        limit: u32,
        offset: u64,
        latest_only: bool,
    },
    LoadMoreEntries,
    ReceiveEntries(Result<Vec<Entry>, String>),
    ReceiveLatestEntry(Result<Entry, String>),
    ExtendDownloadLifetime,
}

pub struct Model {
    api: ApiClient,
    submitter_name: String,
    entries: Vec<Entry>,
    limit: u32,
    offset: u64,
    loading: bool,
    content_ref: NodeRef,
    _download_lifetime_interval: Interval,
}

impl Model {
    fn initial(ctx: &Context<Self>) -> Self {
        let api = ApiClient::new(API_BASE, make_client_hash());
        let link = ctx.link().clone();
        link.send_message(Msg::GetEntries {
            limit: DEFAULT_LIMIT,
            offset: 0,
            latest_only: false,
        });

        register_entry_callback(ctx.link().clone());

        let interval_callback = link.callback(|_| Msg::ExtendDownloadLifetime);
        let _download_lifetime_interval = Interval::new(120_000, move || {
            interval_callback.emit(());
        });

        Self {
            api,
            submitter_name: "Kou".to_string(),
            entries: Vec::new(),
            limit: DEFAULT_LIMIT,
            offset: 0,
            loading: false,
            content_ref: NodeRef::default(),
            _download_lifetime_interval,
        }
    }

    fn submit_entry(&self, ctx: &Context<Self>, content: String, attachments: Vec<web_sys::File>) {
        let form_data = FormData::new().expect("browser must support FormData");
        form_data
            .append_with_str(FORM_NAME_FIELD, &self.submitter_name)
            .expect("append name to FormData");
        form_data
            .append_with_str(FORM_CONTENT_FIELD, &content)
            .expect("append content to FormData");

        for file in attachments {
            form_data
                .append_with_blob_and_filename(FORM_FILE_FIELD, &file, &file.name())
                .expect("append file to FormData");
        }

        let link = ctx.link().clone();
        let add_url = self.api.add_entry_url();
        spawn_local(async move {
            match post_form(&add_url, form_data).await {
                Ok(()) => link.send_message(Msg::GetEntries {
                    limit: 1,
                    offset: 0,
                    latest_only: true,
                }),
                Err(error) => link.send_message(Msg::ReceiveLatestEntry(Err(error))),
            }
        });
    }

    fn fetch_entries(&mut self, ctx: &Context<Self>, limit: u32, offset: u64, latest_only: bool) {
        self.loading = true;

        let link = ctx.link().clone();
        let request_url = self.api.entries_url(limit, offset);

        spawn_local(async move {
            let entries = fetch_entries(&request_url).await;
            if latest_only {
                link.send_message(Msg::ReceiveLatestEntry(entries.and_then(|mut entries| {
                    entries
                        .pop()
                        .ok_or_else(|| "new entry was not returned by server".to_string())
                })));
            } else {
                link.send_message(Msg::ReceiveEntries(entries));
            }
        });
    }

    fn add_older_entries(&mut self, entries: Vec<Entry>) -> bool {
        if entries.is_empty() {
            return false;
        }

        self.offset += entries.len() as u64;
        for entry in entries {
            if !self.entries.iter().any(|existing| existing.id == entry.id) {
                self.entries.insert(0, entry);
            }
        }
        true
    }

    fn add_latest_entry(&mut self, entry: Entry) -> bool {
        if self.entries.iter().any(|existing| existing.id == entry.id) {
            return false;
        }

        self.entries.push(entry);
        self.offset += 1;
        true
    }

    fn scroll_to_position(&self, offset: i32, from_bottom: bool, waiting_time: u32) {
        let content_ref = self.content_ref.clone();

        Timeout::new(waiting_time, move || {
            if let Some(content) = content_ref.cast::<HtmlElement>() {
                if from_bottom {
                    content.set_scroll_top(content.scroll_height() - offset);
                } else {
                    content.set_scroll_top(offset);
                }
            }
        })
        .forget();
    }

    fn register_scroll_loader(&self, ctx: &Context<Self>) {
        let link = ctx.link().clone();
        let content_ref = self.content_ref.clone();
        let callback = Closure::<dyn Fn()>::new(move || {
            if let Some(content) = content_ref.cast::<HtmlElement>() {
                if content.scroll_top() == 0 {
                    link.send_message(Msg::LoadMoreEntries);
                }
            }
        });

        if let Some(content) = self.content_ref.cast::<HtmlElement>() {
            content
                .add_event_listener_with_callback("scroll", callback.as_ref().unchecked_ref())
                .expect("register content scroll listener");
        }
        callback.forget();
    }
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Self::initial(ctx)
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::AddEntry(content, attachments) => {
                if content.trim().is_empty() && attachments.is_empty() {
                    log_error("entry must contain text or at least one attachment");
                    return false;
                }

                self.submit_entry(ctx, content, attachments);
                true
            }
            Msg::GetEntries {
                limit,
                offset,
                latest_only,
            } => {
                self.fetch_entries(ctx, limit, offset, latest_only);
                false
            }
            Msg::LoadMoreEntries => {
                if !self.loading {
                    ctx.link().send_message(Msg::GetEntries {
                        limit: self.limit,
                        offset: self.offset,
                        latest_only: false,
                    });
                }
                false
            }
            Msg::ReceiveEntries(response) => {
                self.loading = false;
                match response {
                    Ok(entries) => {
                        let changed = self.add_older_entries(entries);
                        self.scroll_to_position(10, false, 50);
                        changed
                    }
                    Err(error) => {
                        log_error(&error);
                        false
                    }
                }
            }
            Msg::ReceiveLatestEntry(response) => {
                self.loading = false;
                match response {
                    Ok(entry) => {
                        let changed = self.add_latest_entry(entry);
                        self.scroll_to_position(0, true, 50);
                        changed
                    }
                    Err(error) => {
                        log_error(&error);
                        false
                    }
                }
            }
            Msg::ExtendDownloadLifetime => {
                let request_url = self.api.extend_download_lifetime_url();
                spawn_local(async move {
                    let _ = Request::post(&request_url).send().await;
                });
                false
            }
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if !first_render {
            return;
        }

        self.scroll_to_position(0, true, 300);
        self.register_scroll_loader(ctx);
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div class="container">
                <Header />
                <EntryList
                    entries={self.entries.clone()}
                    api={self.api.clone()}
                    content_ref={self.content_ref.clone()}
                />
                <EditorHost />
            </div>
        }
    }
}
