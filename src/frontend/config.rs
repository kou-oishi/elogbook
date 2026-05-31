pub const DEFAULT_LIMIT: u32 = crate::api::DEFAULT_ENTRY_LIMIT;
pub const API_BASE: &str = match option_env!("ELOGBOOK_API_BASE") {
    Some(value) => value,
    None => "",
};
