use crate::models::{Attachment, Entry};

use pulldown_cmark::{html, Options, Parser};
use yew::prelude::*;
use yew::virtual_dom::VNode;

pub fn markdown_to_html(entry: &Entry, download_url: impl Fn(&str) -> String) -> Html {
    let log_with_attachments = parse_log_text(&entry.log, &entry.attachments, download_url);
    let parser = Parser::new_ext(
        &log_with_attachments,
        Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TABLES,
    );
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    let document = web_sys::window().unwrap().document().unwrap();
    let div = document.create_element("div").unwrap();
    div.set_inner_html(&html_output);
    VNode::VRef(div.into())
}

fn parse_log_text(
    log_text: &str,
    attachments: &[Attachment],
    download_url: impl Fn(&str) -> String,
) -> String {
    let mut result = String::new();
    let mut chars = log_text.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some('%') = chars.peek() {
                result.push('%');
                chars.next();
            } else {
                result.push(c);
            }
        } else if c == '%' {
            let mut id_str = String::new();
            while let Some(&next) = chars.peek() {
                if next.is_numeric() {
                    id_str.push(next);
                    chars.next();
                } else {
                    break;
                }
            }

            if let Ok(id) = id_str.parse::<u32>() {
                if let Some(attachment) = attachments.iter().find(|att| att.id == id) {
                    result.push_str(&expand_attachment_html(attachment, &download_url));
                } else {
                    result.push_str(&format!("%{id}"));
                }
            } else {
                result.push(c);
            }
        } else {
            result.push(c);
        }
    }

    result
}

fn expand_attachment_html(
    attachment: &Attachment,
    download_url: impl Fn(&str) -> String,
) -> String {
    use html_escape::{encode_double_quoted_attribute, encode_text};

    let path = download_url(&attachment.download_token);
    let escaped_path = encode_double_quoted_attribute(&path);
    let escaped_token = encode_double_quoted_attribute(&attachment.download_token);
    let escaped_name = encode_double_quoted_attribute(&attachment.original_name);

    match attachment.mime.as_str() {
        "image/png" | "image/jpeg" | "image/gif" => {
            format!(
                "<div class=\"image-attachment\" data-url=\"{}\" data-id=\"{}\" data-name=\"{}\">Loading image preview...</div>",
                escaped_path, escaped_token, escaped_name
            )
        }
        "application/pdf" => {
            format!(
                "<div class=\"pdf-attachment\" data-url=\"{}\" data-id=\"{}\">Loading PDF preview...</div>",
                escaped_path, escaped_token
            )
        }
        "text/plain" => {
            format!(
                "<div class=\"text-attachment\" data-url=\"{}\" data-id=\"{}\">Loading preview...</div>",
                escaped_path, escaped_token
            )
        }
        _ => {
            format!(
                "<a href=\"{}\" download=\"{}\" class=\"attachment-download\" data-id=\"{}\">Download {}</a>",
                escaped_path,
                escaped_name,
                escaped_token,
                encode_text(&attachment.original_name)
            )
        }
    }
}
