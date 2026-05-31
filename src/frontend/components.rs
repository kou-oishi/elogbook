use crate::frontend::{api::ApiClient, models::Entry, render::markdown_to_html};

use chrono::Local;
use yew::prelude::*;

#[function_component(Header)]
pub fn header() -> Html {
    html! {
        <header class="header">
            <div class="header-flex">
                <h1>{"Elogbook Entries"}</h1>
            </div>
        </header>
    }
}

#[derive(Properties, PartialEq)]
pub struct EntryListProps {
    pub entries: Vec<Entry>,
    pub api: ApiClient,
    pub content_ref: NodeRef,
}

#[function_component(EntryList)]
pub fn entry_list(props: &EntryListProps) -> Html {
    let mut last_date = None;
    let mut last_name = String::new();

    html! {
        <div ref={props.content_ref.clone()} id="content" class="content">
            <ul class="entries-list">
                {
                    for props.entries.iter().map(|entry| {
                        let entry_date = entry.timestamp.with_timezone(&Local).date_naive();
                        let show_date = match last_date {
                            Some(last) if last == entry_date => false,
                            _ => {
                                last_date = Some(entry_date);
                                true
                            }
                        };
                        let show_name = if last_name == entry.name {
                            false
                        } else {
                            last_name = entry.name.clone();
                            true
                        };

                        html! {
                            <EntryItem
                                entry={entry.clone()}
                                api={props.api.clone()}
                                show_date={show_date}
                                show_name={show_name}
                            />
                        }
                    })
                }
            </ul>
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct EntryItemProps {
    entry: Entry,
    api: ApiClient,
    show_date: bool,
    show_name: bool,
}

#[function_component(EntryItem)]
fn entry_item(props: &EntryItemProps) -> Html {
    let entry = &props.entry;
    let entry_date = entry.timestamp.with_timezone(&Local).date_naive();
    let entry_html = markdown_to_html(entry, |token| props.api.download_url(token));

    html! {
        <>
            if props.show_date {
                <div class="entry-date">{ entry_date.format("%Y/%m/%d").to_string() }</div>
                <div class="entry-date-border"/>
            }
            if props.show_name {
                <div class="submitter-name">{entry.name.clone()}</div>
            }
            <li class="entry-item">
                <span class="timestamp">
                    { entry.timestamp.with_timezone(&Local).format("%H:%M (%S)").to_string() }
                </span>
                <span class="log-text">{entry_html}</span>
            </li>
        </>
    }
}

#[function_component(EditorHost)]
pub fn editor_host() -> Html {
    html! {
        <>
            <div id="file-previews" class="file-previews"></div>
            <div class="resize-divider"></div>
            <footer class="footer">
                <textarea
                    value=""
                    class="input-box"
                    placeholder="Enter text here..."
                />
            </footer>
        </>
    }
}
