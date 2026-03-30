use arboard::Clipboard;
use base64::Engine;
use serde::Serialize;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

const POLL_INTERVAL_MS: u64 = 300;

#[derive(Clone, Serialize)]
pub struct ClipboardTextUpdate {
    #[serde(rename = "type")]
    pub kind: String,
    pub content: String,
}

#[derive(Clone, Serialize)]
pub struct ClipboardImageUpdate {
    #[serde(rename = "type")]
    pub kind: String,
    pub content: String, // base64 PNG
    pub width: usize,
    pub height: usize,
}

pub fn start_watcher(app: AppHandle, last_text: Arc<Mutex<String>>, last_img_hash: Arc<Mutex<u64>>) {
    thread::spawn(move || {
        if let Ok(mut cb) = Clipboard::new() {
            if let Ok(t) = cb.get_text() {
                if !t.trim().is_empty() {
                    *last_text.lock().unwrap() = t;
                }
            }
        }

        loop {
            thread::sleep(Duration::from_millis(POLL_INTERVAL_MS));

            // Try text first
            if let Ok(text) = Clipboard::new().and_then(|mut cb| cb.get_text()) {
                if !text.trim().is_empty() {
                    let mut last = match last_text.lock() {
                        Ok(g) => g,
                        Err(p) => p.into_inner(),
                    };
                    if *last != text {
                        *last = text.clone();
                        let _ = app.emit(
                            "clipboard-update",
                            ClipboardTextUpdate {
                                kind: "text".into(),
                                content: text,
                            },
                        );
                        continue;
                    }
                }
            }

            // Try image
            if let Ok(img) = Clipboard::new().and_then(|mut cb| cb.get_image()) {
                let hash = simple_hash(&img.bytes);
                let mut last_h = match last_img_hash.lock() {
                    Ok(g) => g,
                    Err(p) => p.into_inner(),
                };
                if *last_h != hash && !img.bytes.is_empty() {
                    *last_h = hash;

                    // Convert RGBA raw bytes to base64-encoded PNG
                    if let Some(b64) = rgba_to_base64_png(
                        &img.bytes,
                        img.width as u32,
                        img.height as u32,
                    ) {
                        let _ = app.emit(
                            "clipboard-update",
                            ClipboardImageUpdate {
                                kind: "image".into(),
                                content: b64,
                                width: img.width,
                                height: img.height,
                            },
                        );
                    }
                }
            }
        }
    });
}

fn simple_hash(data: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in data.iter().step_by(64.max(data.len() / 1024)) {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn rgba_to_base64_png(rgba: &[u8], width: u32, height: u32) -> Option<String> {
    let mut png_buf = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut png_buf, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().ok()?;
        writer.write_image_data(rgba).ok()?;
    }
    Some(base64::engine::general_purpose::STANDARD.encode(&png_buf))
}

#[tauri::command]
pub fn write_clipboard(
    text: String,
    last_text: tauri::State<'_, Arc<Mutex<String>>>,
) -> Result<(), String> {
    let mut cb = Clipboard::new().map_err(|e| e.to_string())?;
    cb.set_text(&text).map_err(|e| e.to_string())?;
    if let Ok(mut last) = last_text.lock() {
        *last = text;
    }
    Ok(())
}

#[tauri::command]
pub fn write_image_clipboard(
    base64_png: String,
    last_img_hash: tauri::State<'_, Arc<Mutex<u64>>>,
) -> Result<(), String> {
    let png_bytes = base64::engine::general_purpose::STANDARD
        .decode(&base64_png)
        .map_err(|e| format!("base64 decode: {e}"))?;

    let decoder = png::Decoder::new(png_bytes.as_slice());
    let mut reader = decoder.read_info().map_err(|e| format!("png header: {e}"))?;

    let mut rgba_buf = vec![0u8; reader.output_buffer_size()];
    let info = reader
        .next_frame(&mut rgba_buf)
        .map_err(|e| format!("png decode: {e}"))?;
    rgba_buf.truncate(info.buffer_size());

    let img_data = arboard::ImageData {
        width: info.width as usize,
        height: info.height as usize,
        bytes: std::borrow::Cow::Owned(rgba_buf),
    };

    let mut cb = Clipboard::new().map_err(|e| e.to_string())?;
    cb.set_image(img_data).map_err(|e| e.to_string())?;

    if let Ok(mut h) = last_img_hash.lock() {
        *h = simple_hash(&png_bytes);
    }

    Ok(())
}
