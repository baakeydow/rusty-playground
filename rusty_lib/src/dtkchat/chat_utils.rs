/// Get mongodb db name
pub fn get_chat_db_name() -> String {
    std::env::var("RUSTY_CHAT_DB").unwrap_or_else(|_| "rusty_chat".into())
}

/// Get mongodb collection name
pub fn get_chat_collection_name() -> String {
    std::env::var("RUSTY_CHAT_COLL").unwrap_or_else(|_| "chat_data".into())
}