use chrono::{DateTime, Local};

pub use crate::models::AttachmentResponse as Attachment;
use crate::models::EntryResponse;

#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub id: String,
    pub name: String,
    pub log: String,
    pub timestamp: DateTime<Local>,
    pub attachments: Vec<Attachment>,
}

impl TryFrom<EntryResponse> for Entry {
    type Error = String;

    fn try_from(response: EntryResponse) -> Result<Self, Self::Error> {
        let datetime = DateTime::parse_from_rfc3339(&response.created_at)
            .map_err(|error| format!("invalid created_at value: {error}"))?;

        Ok(Entry {
            id: response.id,
            name: response.name,
            log: response.content,
            timestamp: datetime.with_timezone(&Local),
            attachments: response.attachments,
        })
    }
}
