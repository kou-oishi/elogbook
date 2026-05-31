use crate::{error::ApiError, models::Attachment};

use actix_multipart::Field;
use chrono::{DateTime, Datelike, Utc};
use futures_util::StreamExt;
use sha2::{Digest, Sha256};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::{fs, io::AsyncWriteExt};

#[derive(Debug, Clone)]
pub struct AttachmentStore {
    root: Arc<PathBuf>,
}

impl AttachmentStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: Arc::new(root.into()),
        }
    }

    pub async fn save_field(
        &self,
        mut field: Field,
        created_at: DateTime<Utc>,
        id: u32,
    ) -> Result<Attachment, ApiError> {
        let content_disposition = field.content_disposition().ok_or_else(|| {
            ApiError::BadRequest("file field is missing content disposition".into())
        })?;
        let original_name = content_disposition
            .get_filename()
            .filter(|name| !name.trim().is_empty())
            .unwrap_or("attachment")
            .to_string();
        let mime = field
            .content_type()
            .map(ToString::to_string)
            .unwrap_or_else(|| "application/octet-stream".to_string());

        let directory = self.root.join(format!(
            "{:04}/{:02}/{:02}",
            created_at.year(),
            created_at.month(),
            created_at.day()
        ));
        fs::create_dir_all(&directory).await?;

        let file_path = directory.join(hashed_filename(&original_name, created_at, id));
        let mut file = fs::File::create(&file_path).await?;

        while let Some(chunk) = field.next().await {
            file.write_all(&chunk?).await?;
        }

        Ok(Attachment {
            id,
            saved_path: file_path.to_string_lossy().into_owned(),
            original_name,
            mime,
        })
    }
}

fn hashed_filename(original_name: &str, created_at: DateTime<Utc>, id: u32) -> String {
    let mut hasher = Sha256::new();
    hasher.update(created_at.to_rfc3339());
    hasher.update(original_name.as_bytes());
    hasher.update(id.to_be_bytes());

    let mut filename = format!("{:x}", hasher.finalize());
    if let Some(extension) = safe_extension(original_name) {
        filename.push('.');
        filename.push_str(&extension);
    }

    filename
}

fn safe_extension(filename: &str) -> Option<String> {
    Path::new(filename)
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| {
            extension
                .chars()
                .filter(|ch| ch.is_ascii_alphanumeric())
                .collect::<String>()
        })
        .filter(|extension| !extension.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_extension_drops_separator_like_characters() {
        assert_eq!(safe_extension("sample.tar.gz"), Some("gz".to_string()));
        assert_eq!(safe_extension("sample.bad/name"), None);
        assert_eq!(safe_extension("sample.jp-g"), Some("jpg".to_string()));
    }
}
