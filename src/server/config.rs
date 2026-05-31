use std::{env, path::PathBuf};

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub server_addr: String,
    pub mongodb_uri: String,
    pub database_name: String,
    pub attachments_dir: PathBuf,
    pub web_dir: PathBuf,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenvy::dotenv().ok();

        let server_addr = env::var("SERVER_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
        let mongodb_uri =
            env::var("MONGODB_URI").map_err(|_| ConfigError::Missing("MONGODB_URI"))?;
        let database_name = env::var("DB_NAME").map_err(|_| ConfigError::Missing("DB_NAME"))?;
        let attachments_dir = env::var("ATTACHMENTS_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("./attachments"));
        let web_dir = env::var("WEB_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("./dist"));

        Ok(Self {
            server_addr,
            mongodb_uri,
            database_name,
            attachments_dir: absolutize(attachments_dir)?,
            web_dir: absolutize(web_dir)?,
        })
    }
}

fn absolutize(path: PathBuf) -> Result<PathBuf, ConfigError> {
    if path.is_absolute() {
        return Ok(path);
    }

    Ok(env::current_dir()?.join(path))
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("required environment variable {0} is not set")]
    Missing(&'static str),
    #[error("cannot resolve current directory: {0}")]
    CurrentDir(#[from] std::io::Error),
}
