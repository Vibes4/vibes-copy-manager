use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

const HISTORY_FILE: &str = "clipboard_history.json";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HistoryEntry {
    pub text: String,
    pub timestamp: u64,
    pub pinned: bool,
}

fn history_path(app: &tauri::AppHandle) -> PathBuf {
    let dir = app
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."));
    dir.join(HISTORY_FILE)
}

/// Load history from disk. Returns empty vec on any error.
pub fn load(app: &tauri::AppHandle) -> Vec<HistoryEntry> {
    let path = history_path(app);
    match fs::read_to_string(&path) {
        Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

/// Save history to disk. Silently ignores errors.
pub fn save(app: &tauri::AppHandle, entries: &[HistoryEntry]) {
    let path = history_path(app);
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(entries) {
        let _ = fs::write(&path, json);
    }
}
