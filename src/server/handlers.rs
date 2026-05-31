use crate::{
    api::{
        DEFAULT_ENTRY_LIMIT, DOWNLOAD_PATH, ENTRIES_PATH, EXTEND_DOWNLOAD_LIFETIME_PATH,
        FORM_CONTENT_FIELD, FORM_FILE_FIELD, FORM_NAME_FIELD, HEALTH_PATH, MAX_ENTRY_LIMIT,
    },
    models::{AttachmentResponse, EntryResponse},
    server::{download::DownloadLookupError, error::ApiError, models::Entry, state::AppState},
};

use actix_files::NamedFile;
use actix_multipart::{Field, Multipart};
use actix_web::{
    http::header::{ContentDisposition, DispositionParam, DispositionType},
    web, HttpRequest, HttpResponse,
};
use chrono::{DateTime, Utc};
use futures_util::{StreamExt, TryStreamExt};
use mongodb::bson::doc;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GetEntriesParams {
    limit: Option<i64>,
    offset: Option<u64>,
    client: String,
}

#[derive(Debug, Deserialize)]
pub struct AddEntryParams {
    name: Option<String>,
    log: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DownloadParams {
    client: String,
    token: String,
}

#[derive(Debug, Deserialize)]
pub struct ExtendDownloadParams {
    client: String,
}

pub fn configure_routes(config: &mut web::ServiceConfig) {
    config
        .route(HEALTH_PATH, web::get().to(health))
        .route(ENTRIES_PATH, web::post().to(add_entry))
        .route(ENTRIES_PATH, web::get().to(get_entries))
        .route(DOWNLOAD_PATH, web::get().to(download_file))
        .route(
            EXTEND_DOWNLOAD_LIFETIME_PATH,
            web::post().to(extend_download_lifetime),
        );
}

pub async fn health() -> HttpResponse {
    HttpResponse::Ok().body("elogbook server is running")
}

pub async fn add_entry(
    state: web::Data<AppState>,
    params: web::Query<AddEntryParams>,
    mut payload: Multipart,
) -> Result<HttpResponse, ApiError> {
    let created_at = Utc::now();
    let mut name = params.name.clone().unwrap_or_default();
    let mut content = params.log.clone().unwrap_or_default();
    let mut attachments = Vec::new();

    while let Some(field) = payload.next().await {
        let field = field?;
        let field_name = field.name().unwrap_or_default().to_string();

        match field_name.as_str() {
            FORM_NAME_FIELD => name = read_text_field(field).await?,
            FORM_CONTENT_FIELD => content = read_text_field(field).await?,
            FORM_FILE_FIELD => {
                let id = attachments.len() as u32 + 1;
                attachments.push(state.attachments.save_field(field, created_at, id).await?);
            }
            _ => {}
        }
    }

    if content.trim().is_empty() && attachments.is_empty() {
        return Err(ApiError::BadRequest(
            "entry must contain text or at least one attachment".into(),
        ));
    }

    let mut entry = Entry::new(name, content, created_at, attachments);
    let insert_result = state.entries.insert_one(&entry).await?;
    entry.id = insert_result.inserted_id.as_object_id();

    Ok(HttpResponse::Created().json(entry_to_response(entry, &state, None).await))
}

pub async fn get_entries(
    state: web::Data<AppState>,
    params: web::Query<GetEntriesParams>,
) -> Result<HttpResponse, ApiError> {
    state.downloads.clean_expired().await;

    let limit = params
        .limit
        .unwrap_or(DEFAULT_ENTRY_LIMIT as i64)
        .clamp(1, MAX_ENTRY_LIMIT as i64);
    let mut cursor = state
        .entries
        .find(doc! { "created_at": { "$type": "date" } })
        .sort(doc! { "created_at": -1, "_id": -1 })
        .limit(limit)
        .skip(params.offset.unwrap_or_default())
        .await?;

    let mut entries = Vec::new();
    while let Some(entry) = cursor.try_next().await? {
        entries.push(entry_to_response(entry, &state, Some(&params.client)).await);
    }

    Ok(HttpResponse::Ok().json(entries))
}

pub async fn extend_download_lifetime(
    state: web::Data<AppState>,
    params: web::Query<ExtendDownloadParams>,
) -> HttpResponse {
    state.downloads.extend(&params.client).await;
    HttpResponse::NoContent().finish()
}

pub async fn download_file(
    req: HttpRequest,
    state: web::Data<AppState>,
    params: web::Query<DownloadParams>,
) -> Result<HttpResponse, ApiError> {
    let request = state
        .downloads
        .take(&params.client, &params.token)
        .await
        .map_err(download_error)?;
    let file = NamedFile::open_async(&request.file_path).await?;

    Ok(file
        .use_last_modified(true)
        .set_content_disposition(ContentDisposition {
            disposition: DispositionType::Attachment,
            parameters: vec![DispositionParam::Filename(request.original_name)],
        })
        .into_response(&req))
}

async fn read_text_field(mut field: Field) -> Result<String, ApiError> {
    let mut bytes = Vec::new();
    while let Some(chunk) = field.next().await {
        bytes.extend_from_slice(&chunk?);
    }

    String::from_utf8(bytes)
        .map_err(|_| ApiError::BadRequest("multipart text field is not valid UTF-8".into()))
}

async fn entry_to_response(
    entry: Entry,
    state: &AppState,
    client_id: Option<&str>,
) -> EntryResponse {
    let attachments = if let Some(client_id) = client_id {
        let mut attachments = Vec::with_capacity(entry.attachments.len());
        for attachment in &entry.attachments {
            attachments.push(AttachmentResponse {
                id: attachment.id,
                mime: attachment.mime.clone(),
                original_name: attachment.original_name.clone(),
                download_token: state.downloads.register(client_id, attachment).await,
            });
        }
        attachments
    } else {
        Vec::new()
    };

    EntryResponse {
        id: entry.id.map(|id| id.to_hex()).unwrap_or_default(),
        name: entry.name,
        content: entry.content,
        created_at: entry
            .created_at
            .try_to_rfc3339_string()
            .unwrap_or_else(|_| bson_datetime_to_rfc3339(entry.created_at)),
        attachments,
    }
}

fn bson_datetime_to_rfc3339(datetime: mongodb::bson::DateTime) -> String {
    DateTime::<Utc>::from_timestamp_millis(datetime.timestamp_millis())
        .unwrap_or(DateTime::<Utc>::UNIX_EPOCH)
        .to_rfc3339()
}

fn download_error(error: DownloadLookupError) -> ApiError {
    match error {
        DownloadLookupError::UnknownClient => ApiError::NotFound("unknown download client".into()),
        DownloadLookupError::ExpiredClient => {
            ApiError::BadRequest("expired download client".into())
        }
        DownloadLookupError::UnknownToken => ApiError::NotFound("unknown download token".into()),
    }
}
