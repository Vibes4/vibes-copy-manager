pub mod autostart;
pub mod config;
pub mod engine;
pub mod platform;

#[cfg(feature = "gui")]
mod clipboard;
#[cfg(feature = "gui")]
mod persistence;
#[cfg(feature = "gui")]
pub mod window;
#[cfg(all(feature = "gui", target_os = "linux"))]
mod portal_global_shortcut;

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

/// Register global hotkey: Wayland uses the Global Shortcuts portal, others use
/// tauri (X11 grab on Linux). Logs warnings and suggests OS-level shortcut on failure.
#[cfg(feature = "gui")]
fn register_global_shortcut(
    app: &tauri::AppHandle,
    shortcut: Option<&str>,
    max_items_log: Option<usize>,
) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        let backend = platform::LinuxDisplayBackend::detect();
        log::info!("Display backend: {}", backend);
        self::portal_global_shortcut::stop();
    }
    app.global_shortcut()
        .unregister_all()
        .map_err(|e| e.to_string())?;

    let Some(shortcut_str) = shortcut else {
        log::info!("No shortcut configured. Use tray icon or `vcm` CLI to open.");
        return Ok(());
    };
    if shortcut_str.is_empty() {
        log::info!("No shortcut configured. Use tray icon or `vcm` CLI to open.");
        return Ok(());
    }

    #[cfg(target_os = "linux")]
    {
        let backend = platform::LinuxDisplayBackend::detect();
        if backend.is_wayland() {
            log::info!("Wayland session detected — attempting Global Shortcuts portal for: {}", shortcut_str);
            match self::portal_global_shortcut::start(app.clone(), shortcut_str) {
                Ok(()) => {
                    if let Some(m) = max_items_log {
                        log::info!(
                            "Global shortcut registered (Wayland portal): {} | Max items: {m}",
                            shortcut_str
                        );
                    } else {
                        log::info!("Global shortcut registered (Wayland portal): {}", shortcut_str);
                    }
                    return Ok(());
                }
                Err(e) => {
                    log::warn!(
                        "Wayland Global Shortcuts portal failed: {e}"
                    );
                    log::warn!(
                        "Falling back to Tauri X11 grab — this may not work on pure Wayland."
                    );
                    log::warn!(
                        "Recommended: configure your desktop environment to run `vcm` on Super+V (or your preferred shortcut)."
                    );
                }
            }
        }
    }

    // Attempt Tauri native shortcut registration (works on X11, macOS, Windows)
    let (mods, code) = match config::parse_shortcut(shortcut_str) {
        Ok(v) => v,
        Err(e) => {
            let msg = format!("Invalid shortcut '{}': {}", shortcut_str, e);
            log::error!("{}", msg);
            return Err(msg);
        }
    };
    let s = Shortcut::new(Some(mods), code);
    match app.global_shortcut().on_shortcut(s, |app, _h, event| {
        if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
            log::debug!("Global shortcut triggered");
            let _ = window::do_toggle(app);
        }
    }) {
        Ok(()) => {
            if let Some(m) = max_items_log {
                log::info!("Global shortcut registered: {} | Max items: {m}", shortcut_str);
            } else {
                log::info!("Global shortcut registered: {}", shortcut_str);
            }
            Ok(())
        }
        Err(e) => {
            let msg = format!("Failed to register shortcut '{}': {}", shortcut_str, e);
            log::warn!("{}", msg);
            #[cfg(target_os = "linux")]
            {
                log::warn!(
                    "Shortcut registration failed. On Linux, configure your desktop environment to run `vcm` as a custom shortcut."
                );
            }
            Err(msg)
        }
    }
}

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

    log::info!("Vibes Copy Manager starting | Platform: {}", platform::platform_info());

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            log::info!("Second instance detected — showing existing window");
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
            get_platform_info,
        ])
        .setup(move |app| {
            clipboard::start_watcher(app.handle().clone(), watcher_text, watcher_img);

            app.handle().plugin(
                tauri_plugin_global_shortcut::Builder::new().build(),
            )?;

            // Attempt shortcut registration — non-fatal on failure
            match register_global_shortcut(
                app.handle(),
                cfg.shortcut.as_deref(),
                Some(cfg.max_items),
            ) {
                Ok(()) => {}
                Err(e) => {
                    log::warn!("Shortcut registration failed at startup: {e}");
                    log::info!("App continues without global shortcut. Use tray icon or `vcm` CLI.");
                    #[cfg(target_os = "linux")]
                    {
                        let backend = platform::LinuxDisplayBackend::detect();
                        if backend.is_wayland() {
                            log::info!(
                                "Tip: On Wayland, configure your desktop environment to run `vcm` as a custom shortcut (e.g., Super+V → vcm)"
                            );
                        }
                    }
                }
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
    register_global_shortcut(&app, cfg.shortcut.as_deref(), None)?;

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

#[cfg(feature = "gui")]
#[tauri::command]
fn get_platform_info() -> PlatformInfo {
    PlatformInfo {
        os: std::env::consts::OS.to_string(),
        #[cfg(target_os = "linux")]
        display_backend: format!("{}", platform::LinuxDisplayBackend::detect()),
        #[cfg(not(target_os = "linux"))]
        display_backend: "native".to_string(),
        is_wayland: {
            #[cfg(target_os = "linux")]
            { platform::LinuxDisplayBackend::detect().is_wayland() }
            #[cfg(not(target_os = "linux"))]
            { false }
        },
    }
}

#[cfg(feature = "gui")]
#[derive(serde::Serialize)]
struct PlatformInfo {
    os: String,
    #[serde(rename = "displayBackend")]
    display_backend: String,
    #[serde(rename = "isWayland")]
    is_wayland: bool,
}
