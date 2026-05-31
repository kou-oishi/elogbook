use chrono::{DateTime, Utc};
use mongodb::bson::{oid::ObjectId, DateTime as BsonDateTime};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub id: u32,
    pub saved_path: String,
    pub original_name: String,
    pub mime: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub content: String,
    pub created_at: BsonDateTime,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<Attachment>,
}

impl Entry {
    pub fn new(
        name: String,
        content: String,
        created_at: DateTime<Utc>,
        attachments: Vec<Attachment>,
    ) -> Self {
        Self {
            id: None,
            name,
            content,
            created_at: BsonDateTime::from_millis(created_at.timestamp_millis()),
            attachments,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentResponse {
    pub id: u32,
    pub mime: String,
    pub original_name: String,
    pub download_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryResponse {
    pub id: String,
    pub name: String,
    pub content: String,
    pub created_at: String,
    pub attachments: Vec<AttachmentResponse>,
}

#[derive(Debug, Clone)]
pub struct DownloadRequest {
    pub file_path: String,
    pub original_name: String,
}

#[derive(Debug, Clone)]
pub struct DownloadClient {
    pub expires_at: DateTime<Utc>,
    pub requests: HashMap<String, DownloadRequest>,
}

impl DownloadClient {
    pub fn new(expires_at: DateTime<Utc>) -> Self {
        Self {
            expires_at,
            requests: HashMap::new(),
        }
    }
}
