pub mod autostart;
pub mod config;
pub mod engine;

#[cfg(feature = "gui")]
mod clipboard;
#[cfg(feature = "gui")]
mod persistence;
#[cfg(feature = "gui")]
pub mod window;

#[cfg(feature = "gui")]
use std::sync::{Arc, Mutex};
#[cfg(feature = "gui")]
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, WindowEvent,
};
#[cfg(feature = "gui")]
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

#[cfg(feature = "gui")]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let last_text: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    let last_img_hash: Arc<Mutex<u64>> = Arc::new(Mutex::new(0));
    let watcher_text = Arc::clone(&last_text);
    let watcher_img = Arc::clone(&last_img_hash);

    let first_run = !config::exists();
    let cfg = config::load();
    let needs_setup = cfg.shortcut.is_none();

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            let _ = window::do_show(app);
        }))
        .plugin(tauri_plugin_opener::init())
        .manage(last_text)
        .manage(last_img_hash)
        .invoke_handler(tauri::generate_handler![
            clipboard::write_clipboard,
            clipboard::write_image_clipboard,
            window::hide_window,
            window::show_window,
            window::toggle_window,
            window::paste_and_hide,
            load_history,
            save_history,
            get_config,
            set_config,
            get_autostart,
            set_autostart,
        ])
        .setup(move |app| {
            clipboard::start_watcher(app.handle().clone(), watcher_text, watcher_img);

            if let Some(ref shortcut_str) = cfg.shortcut {
                match config::parse_shortcut(shortcut_str) {
                    Ok((mods, code)) => {
                        let shortcut = Shortcut::new(Some(mods), code);
                        let handle = app.handle().clone();

                        app.handle().plugin(
                            tauri_plugin_global_shortcut::Builder::new()
                                .with_handler(move |_app, hotkey, event| {
                                    if event.state
                                        == tauri_plugin_global_shortcut::ShortcutState::Pressed
                                        && *hotkey == shortcut
                                    {
                                        let _ = window::do_toggle(&handle);
                                    }
                                })
                                .build(),
                        )?;
                        app.global_shortcut().register(shortcut)?;
                        log::info!(
                            "Shortcut: {} | Max items: {}",
                            shortcut_str,
                            cfg.max_items
                        );
                    }
                    Err(e) => {
                        log::warn!("Invalid shortcut {:?}: {}, skipping registration", shortcut_str, e);
                        app.handle().plugin(
                            tauri_plugin_global_shortcut::Builder::new().build(),
                        )?;
                    }
                }
            } else {
                log::info!("No shortcut configured. Use tray icon or vcm settings.");
                app.handle().plugin(
                    tauri_plugin_global_shortcut::Builder::new().build(),
                )?;
            }

            let show_item = MenuItemBuilder::with_id("show", "Show").build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            let menu = MenuBuilder::new(app)
                .items(&[&show_item, &quit_item])
                .build()?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().cloned().unwrap())
                .tooltip("Vibes Copy Manager")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show" => {
                        let _ = window::do_show(app);
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let _ = window::do_toggle(tray.app_handle());
                    }
                })
                .build(app)?;

            if let Some(win) = app.get_webview_window("main") {
                let w = win.clone();
                win.on_window_event(move |event| match event {
                    WindowEvent::CloseRequested { api, .. } => {
                        api.prevent_close();
                        let _ = w.emit("window-hiding", ());
                        let _ = w.hide();
                        let _ = w.set_always_on_top(false);
                    }
                    _ => {}
                });
            }

            if first_run || needs_setup {
                let handle = app.handle().clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(300));
                    let _ = window::do_show(&handle);
                    let _ = handle.emit("open-settings", ());
                });
            }

            Ok(())
        });

    if let Err(e) = builder.run(tauri::generate_context!()) {
        log::error!("Application exited with error: {e}");
        std::process::exit(1);
    }
}

// ─── Tauri Commands ──────────────────────────────────────────────

#[cfg(feature = "gui")]
#[tauri::command]
fn load_history(app: tauri::AppHandle) -> Vec<persistence::HistoryEntry> {
    persistence::load(&app)
}

#[cfg(feature = "gui")]
#[tauri::command]
fn save_history(app: tauri::AppHandle, entries: Vec<persistence::HistoryEntry>) {
    persistence::save(&app, &entries);
}

#[cfg(feature = "gui")]
#[tauri::command]
fn get_config() -> config::AppConfig {
    config::load()
}

#[cfg(feature = "gui")]
#[tauri::command]
fn set_config(app: tauri::AppHandle, cfg: config::AppConfig) -> Result<(), String> {
    if let Some(ref shortcut_str) = cfg.shortcut {
        let (mods, code) = config::parse_shortcut(shortcut_str)?;
        let new_shortcut = Shortcut::new(Some(mods), code);

        let gs = app.global_shortcut();
        gs.unregister_all().map_err(|e| e.to_string())?;
        gs.on_shortcut(new_shortcut, move |app, _hotkey, event| {
            if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                let _ = window::do_toggle(app);
            }
        })
        .map_err(|e| e.to_string())?;
    } else {
        let gs = app.global_shortcut();
        gs.unregister_all().map_err(|e| e.to_string())?;
    }

    if cfg.auto_start {
        if let Ok(exe) = std::env::current_exe() {
            let _ = autostart::enable(&exe.to_string_lossy());
        }
    } else {
        let _ = autostart::disable();
    }

    config::save(&cfg);
    Ok(())
}

#[cfg(feature = "gui")]
#[tauri::command]
fn get_autostart() -> bool {
    autostart::is_enabled()
}

#[cfg(feature = "gui")]
#[tauri::command]
fn set_autostart(enabled: bool) -> Result<(), String> {
    if enabled {
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        autostart::enable(&exe.to_string_lossy())
    } else {
        autostart::disable()
    }
}
