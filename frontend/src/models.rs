use chrono::{DateTime, Local};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Attachment {
    pub id: u32,
    pub mime: String,
    pub original_name: String,
    pub download_token: String,
}

#[derive(Debug, Deserialize)]
pub struct EntryResponse {
    pub id: String,
    pub name: String,
    pub content: String,
    pub created_at: String,
    pub attachments: Vec<Attachment>,
}

impl EntryResponse {
    pub fn into_entry(self) -> Result<Entry, String> {
        let datetime = DateTime::parse_from_rfc3339(&self.created_at)
            .map_err(|error| format!("invalid created_at value: {error}"))?;

        Ok(Entry {
            id: self.id,
            name: self.name,
            log: self.content,
            timestamp: datetime.with_timezone(&Local),
            attachments: self.attachments,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub id: String,
    pub name: String,
    pub log: String,
    pub timestamp: DateTime<Local>,
    pub attachments: Vec<Attachment>,
}
