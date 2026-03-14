use crate::app::RequestHistoryEntry;
use crate::collection::Collection;
use std::fs;
use std::path::PathBuf;

/// Get the application data directory (~/.api-client/)
pub fn get_data_dir() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".api-client")
}

/// Ensure the data directory exists
fn ensure_dir() -> std::io::Result<PathBuf> {
    let dir = get_data_dir();
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Path to the history file
fn history_path() -> PathBuf {
    get_data_dir().join("history.json")
}

/// Path to the collections file
fn collections_path() -> PathBuf {
    get_data_dir().join("collections.json")
}

/// Save history entries to disk
pub fn save_history(entries: &[RequestHistoryEntry]) -> anyhow::Result<()> {
    ensure_dir()?;
    let json = serde_json::to_string_pretty(entries)?;
    fs::write(history_path(), json)?;
    Ok(())
}

/// Load history entries from disk
pub fn load_history() -> Vec<RequestHistoryEntry> {
    let path = history_path();
    if !path.exists() {
        return Vec::new();
    }
    match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

/// Save collections to disk
pub fn save_collections(collections: &[Collection]) -> anyhow::Result<()> {
    ensure_dir()?;
    let json = serde_json::to_string_pretty(collections)?;
    fs::write(collections_path(), json)?;
    Ok(())
}

/// Load collections from disk
pub fn load_collections() -> Vec<Collection> {
    let path = collections_path();
    if !path.exists() {
        return Vec::new();
    }
    match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

