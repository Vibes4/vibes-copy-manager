use arboard::Clipboard;
use serde::Serialize;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

const POLL_INTERVAL_MS: u64 = 300;

#[derive(Clone, Serialize)]
pub struct ClipboardUpdate {
    pub text: String,
}

pub fn start_watcher(app: AppHandle, last_text: Arc<Mutex<String>>) {
    thread::spawn(move || {
        // Seed with whatever is currently on the clipboard so we don't
        // emit the pre-existing content as a "new" item on launch.
        if let Ok(mut cb) = Clipboard::new() {
            if let Ok(t) = cb.get_text() {
                if !t.trim().is_empty() {
                    *last_text.lock().unwrap() = t;
                }
            }
        }

        loop {
            thread::sleep(Duration::from_millis(POLL_INTERVAL_MS));

            // Create a fresh Clipboard handle each poll.
            // On Linux/X11 a long-lived handle can go stale when the
            // clipboard owner changes (browser copies, etc.).
            let current = match Clipboard::new().and_then(|mut cb| cb.get_text()) {
                Ok(t) => t,
                Err(_) => continue,
            };

            if current.trim().is_empty() {
                continue;
            }

            let mut last = match last_text.lock() {
                Ok(g) => g,
                Err(poisoned) => poisoned.into_inner(),
            };

            if *last != current {
                eprintln!("[clipboard] new: {} chars", current.len());
                *last = current.clone();
                let _ = app.emit("clipboard-update", ClipboardUpdate { text: current });
            }
        }
    });
}

#[tauri::command]
pub fn write_clipboard(text: String, last_text: tauri::State<'_, Arc<Mutex<String>>>) -> Result<(), String> {
    let mut cb = Clipboard::new().map_err(|e| e.to_string())?;
    cb.set_text(&text).map_err(|e| e.to_string())?;

    // Update the watcher's last-seen value so it doesn't re-emit
    // the text we just wrote as a "new" clipboard change.
    if let Ok(mut last) = last_text.lock() {
        *last = text;
    }
    Ok(())
}
