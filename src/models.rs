use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttachmentResponse {
    pub id: u32,
    pub mime: String,
    pub original_name: String,
    pub download_token: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryResponse {
    pub id: String,
    pub name: String,
    pub content: String,
    pub created_at: String,
    pub attachments: Vec<AttachmentResponse>,
}
