use std::process::Command;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

fn main_window(app: &AppHandle) -> Result<tauri::WebviewWindow, String> {
    app.get_webview_window("main")
        .ok_or_else(|| "main window not found".into())
}

/// Force-activate our window on X11 using xdotool with our PID.
/// Falls back silently if xdotool isn't installed.
fn x11_force_focus() {
    let pid = std::process::id().to_string();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(30));

        // Find our window by PID — no title scanning, no noise
        let wid = Command::new("xdotool")
            .args(["search", "--pid", &pid, "--onlyvisible"])
            .output();

        if let Ok(output) = wid {
            let ids = String::from_utf8_lossy(&output.stdout);
            if let Some(id) = ids.lines().last() {
                let id = id.trim();
                if !id.is_empty() {
                    let _ = Command::new("xdotool")
                        .args(["windowactivate", "--sync", id])
                        .status();
                    let _ = Command::new("xdotool")
                        .args(["windowfocus", "--sync", id])
                        .status();
                }
            }
        }
    });
}

pub fn do_show(app: &AppHandle) -> Result<(), String> {
    let win = main_window(app)?;
    win.center().map_err(|e| e.to_string())?;
    win.set_always_on_top(true).map_err(|e| e.to_string())?;
    win.show().map_err(|e| e.to_string())?;
    win.set_focus().map_err(|e| e.to_string())?;

    x11_force_focus();

    let _ = app.emit("window-shown", ());
    Ok(())
}

pub fn do_hide(app: &AppHandle) -> Result<(), String> {
    let win = main_window(app)?;
    win.hide().map_err(|e| e.to_string())?;
    win.set_always_on_top(false).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn do_toggle(app: &AppHandle) -> Result<(), String> {
    let win = main_window(app)?;
    if win.is_visible().map_err(|e| e.to_string())? {
        do_hide(app)
    } else {
        do_show(app)
    }
}

#[tauri::command]
pub fn hide_window(app: AppHandle) -> Result<(), String> {
    do_hide(&app)
}

#[tauri::command]
pub fn show_window(app: AppHandle) -> Result<(), String> {
    do_show(&app)
}

#[tauri::command]
pub fn toggle_window(app: AppHandle) -> Result<(), String> {
    do_toggle(&app)
}

#[tauri::command]
pub fn paste_and_hide(app: AppHandle) -> Result<(), String> {
    do_hide(&app)?;

    thread::spawn(move || {
        thread::sleep(Duration::from_millis(80));

        let is_wayland = std::env::var("XDG_SESSION_TYPE")
            .map(|v| v.to_lowercase() == "wayland")
            .unwrap_or(false);

        let result = if is_wayland {
            Command::new("wtype")
                .args(["-M", "ctrl", "-k", "v", "-m", "ctrl"])
                .status()
        } else {
            Command::new("xdotool")
                .args(["key", "--clearmodifiers", "ctrl+v"])
                .status()
        };

        if let Err(e) = result {
            let tool = if is_wayland { "wtype" } else { "xdotool" };
            eprintln!("[paste] failed to run {}: {}", tool, e);
        }
    });

    Ok(())
}
