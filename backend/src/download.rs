use crate::models::{Attachment, DownloadClient, DownloadRequest};

use chrono::{Duration, Utc};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use uuid::Uuid;

const TOKEN_TTL: Duration = Duration::minutes(5);

#[derive(Debug, Clone, Default)]
pub struct DownloadRegistry {
    clients: Arc<Mutex<HashMap<String, DownloadClient>>>,
}

impl DownloadRegistry {
    pub async fn register(&self, client_id: &str, attachment: &Attachment) -> String {
        let token = Uuid::new_v4().simple().to_string();
        let expires_at = Utc::now() + TOKEN_TTL;
        let request = DownloadRequest {
            file_path: attachment.saved_path.clone(),
            original_name: attachment.original_name.clone(),
        };

        let mut clients = self.clients.lock().await;
        let client = clients
            .entry(client_id.to_string())
            .or_insert_with(|| DownloadClient::new(expires_at));
        client.expires_at = expires_at;
        client.requests.insert(token.clone(), request);

        token
    }

    pub async fn extend(&self, client_id: &str) {
        let mut clients = self.clients.lock().await;
        if let Some(client) = clients.get_mut(client_id) {
            client.expires_at = Utc::now() + TOKEN_TTL;
        }
    }

    pub async fn take(
        &self,
        client_id: &str,
        token: &str,
    ) -> Result<DownloadRequest, DownloadLookupError> {
        let mut clients = self.clients.lock().await;
        let Some(client) = clients.get_mut(client_id) else {
            return Err(DownloadLookupError::UnknownClient);
        };

        if Utc::now() >= client.expires_at {
            clients.remove(client_id);
            return Err(DownloadLookupError::ExpiredClient);
        }

        client
            .requests
            .remove(token)
            .ok_or(DownloadLookupError::UnknownToken)
    }

    pub async fn clean_expired(&self) {
        let mut clients = self.clients.lock().await;
        let now = Utc::now();
        clients.retain(|_, client| now < client.expires_at);
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DownloadLookupError {
    #[error("unknown download client")]
    UnknownClient,
    #[error("expired download client")]
    ExpiredClient,
    #[error("unknown download token")]
    UnknownToken,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn attachment() -> Attachment {
        Attachment {
            id: 1,
            saved_path: "/tmp/demo.txt".to_string(),
            original_name: "demo.txt".to_string(),
            mime: "text/plain".to_string(),
        }
    }

    #[tokio::test]
    async fn token_is_single_use() {
        let registry = DownloadRegistry::default();
        let token = registry.register("client", &attachment()).await;

        assert!(registry.take("client", &token).await.is_ok());
        assert!(matches!(
            registry.take("client", &token).await,
            Err(DownloadLookupError::UnknownToken)
        ));
    }
}
