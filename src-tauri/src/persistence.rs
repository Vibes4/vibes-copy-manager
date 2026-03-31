use crate::engine;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HistoryEntry {
    #[serde(rename = "type")]
    pub kind: String,
    pub content: String,
    pub pinned: bool,
    #[serde(rename = "createdAt")]
    pub created_at: u64,
}

impl From<engine::HistoryEntry> for HistoryEntry {
    fn from(e: engine::HistoryEntry) -> Self {
        Self {
            kind: e.kind,
            content: e.content,
            pinned: e.pinned,
            created_at: e.created_at,
        }
    }
}

impl From<HistoryEntry> for engine::HistoryEntry {
    fn from(e: HistoryEntry) -> Self {
        Self {
            kind: e.kind,
            content: e.content,
            pinned: e.pinned,
            created_at: e.created_at,
        }
    }
}

pub fn load(_app: &tauri::AppHandle) -> Vec<HistoryEntry> {
    engine::load_history().into_iter().map(Into::into).collect()
}

pub fn save(_app: &tauri::AppHandle, entries: &[HistoryEntry]) {
    let engine_entries: Vec<engine::HistoryEntry> =
        entries.iter().cloned().map(Into::into).collect();
    engine::save_history(&engine_entries);
}
