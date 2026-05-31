pub const DEFAULT_LIMIT: u32 = 20;
pub const API_BASE: &str = match option_env!("ELOGBOOK_API_BASE") {
    Some(value) => value,
    None => "http://127.0.0.1:8080",
};
