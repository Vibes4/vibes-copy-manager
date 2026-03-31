use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const HISTORY_FILE: &str = "clipboard_history.json";
const DATA_DIR: &str = "vibes-copy-manager";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HistoryEntry {
    #[serde(rename = "type")]
    pub kind: String,
    pub content: String,
    pub pinned: bool,
    #[serde(rename = "createdAt")]
    pub created_at: u64,
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn data_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(DATA_DIR)
}

fn history_path() -> PathBuf {
    data_dir().join(HISTORY_FILE)
}

pub fn load_history() -> Vec<HistoryEntry> {
    let path = history_path();
    match fs::read_to_string(&path) {
        Ok(data) => match serde_json::from_str(&data) {
            Ok(entries) => entries,
            Err(e) => {
                log::warn!("Malformed history at {}: {e}", path.display());
                Vec::new()
            }
        },
        Err(_) => Vec::new(),
    }
}

pub fn save_history(entries: &[HistoryEntry]) {
    let path = history_path();
    if let Some(parent) = path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            log::error!("Could not create data dir {}: {e}", parent.display());
            return;
        }
    }
    match serde_json::to_string_pretty(entries) {
        Ok(json) => {
            if let Err(e) = fs::write(&path, json) {
                log::error!("Could not write history to {}: {e}", path.display());
            }
        }
        Err(e) => {
            log::error!("Could not serialize history: {e}");
        }
    }
}

pub fn push_text(text: &str, max_items: usize) {
    let mut entries = load_history();

    entries.retain(|e| !(e.kind == "text" && e.content == text));

    let entry = HistoryEntry {
        kind: "text".into(),
        content: text.to_string(),
        pinned: false,
        created_at: now_ms(),
    };

    let insert_pos = entries.iter().position(|e| !e.pinned).unwrap_or(entries.len());
    entries.insert(insert_pos, entry);

    trim(&mut entries, max_items);
    save_history(&entries);
}

pub fn pop(index: Option<usize>) -> Option<HistoryEntry> {
    let entries = load_history();
    let idx = index.unwrap_or(0);
    entries.into_iter().nth(idx)
}

pub fn clear_all() {
    let entries = load_history();
    let pinned: Vec<_> = entries.into_iter().filter(|e| e.pinned).collect();
    save_history(&pinned);
}

pub fn clear_index(index: usize) {
    let mut entries = load_history();
    if index < entries.len() {
        entries.remove(index);
        save_history(&entries);
    }
}

pub fn list_items() -> Vec<HistoryEntry> {
    load_history()
}

fn trim(entries: &mut Vec<HistoryEntry>, max: usize) {
    while entries.len() > max {
        if let Some(pos) = entries.iter().rposition(|e| !e.pinned) {
            entries.remove(pos);
        } else {
            break;
        }
    }
}

pub fn set_system_clipboard(text: &str) -> Result<(), String> {
    let mut cb = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    cb.set_text(text).map_err(|e| e.to_string())?;
    Ok(())
}
