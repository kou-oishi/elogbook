use crate::{
    attachments::AttachmentStore, config::AppConfig, download::DownloadRegistry, models::Entry,
};

use mongodb::{
    bson::doc,
    options::{ClientOptions, IndexOptions},
    Client, Collection, IndexModel,
};

#[derive(Clone)]
pub struct AppState {
    pub entries: Collection<Entry>,
    pub attachments: AttachmentStore,
    pub downloads: DownloadRegistry,
}

impl AppState {
    pub async fn connect(config: &AppConfig) -> mongodb::error::Result<Self> {
        let client_options = ClientOptions::parse(&config.mongodb_uri).await?;
        let client = Client::with_options(client_options)?;
        let db = client.database(&config.database_name);
        let entries = db.collection::<Entry>("entries");

        entries
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "created_at": -1, "_id": -1 })
                    .options(
                        IndexOptions::builder()
                            .name("entries_by_created_at".to_string())
                            .build(),
                    )
                    .build(),
            )
            .await?;

        Ok(Self {
            entries,
            attachments: AttachmentStore::new(config.attachments_dir.clone()),
            downloads: DownloadRegistry::default(),
        })
    }
}
