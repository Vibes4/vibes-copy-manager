mod clipboard;
mod persistence;
pub mod window;

use std::sync::{Arc, Mutex};
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, WindowEvent,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let last_text: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    let watcher_ref = Arc::clone(&last_text);

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            let _ = window::do_show(app);
        }))
        .plugin(tauri_plugin_opener::init())
        .manage(last_text)
        .invoke_handler(tauri::generate_handler![
            clipboard::write_clipboard,
            window::hide_window,
            window::show_window,
            window::toggle_window,
            window::paste_and_hide,
            load_history,
            save_history,
        ])
        .setup(move |app| {
            clipboard::start_watcher(app.handle().clone(), watcher_ref);

            // --- Global shortcut: Ctrl+Shift+V ---
            let shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyV);
            let handle = app.handle().clone();
            app.handle().plugin(
                tauri_plugin_global_shortcut::Builder::new()
                    .with_handler(move |_app, hotkey, event| {
                        if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed
                            && *hotkey == shortcut
                        {
                            let _ = window::do_toggle(&handle);
                        }
                    })
                    .build(),
            )?;
            app.global_shortcut().register(shortcut)?;

            // --- System tray ---
            let show_item = MenuItemBuilder::with_id("show", "Show").build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            let menu = MenuBuilder::new(app)
                .items(&[&show_item, &quit_item])
                .build()?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().cloned().unwrap())
                .tooltip("Clipboard Manager")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show" => { let _ = window::do_show(app); }
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

            eprintln!("[clipboard-manager] Running in background.");
            eprintln!("[clipboard-manager] Press Ctrl+Shift+V to toggle the window.");
            eprintln!("[clipboard-manager] Or click the tray icon.");

            // --- Intercept close → hide, and focus-lost → hide ---
            if let Some(win) = app.get_webview_window("main") {
                let w = win.clone();
                win.on_window_event(move |event| {
                    match event {
                        WindowEvent::CloseRequested { api, .. } => {
                            api.prevent_close();
                            let _ = w.hide();
                            let _ = w.set_always_on_top(false);
                        }
                        WindowEvent::Focused(false) => {
                            let _ = w.hide();
                            let _ = w.set_always_on_top(false);
                        }
                        _ => {}
                    }
                });
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn load_history(app: tauri::AppHandle) -> Vec<persistence::HistoryEntry> {
    persistence::load(&app)
}

#[tauri::command]
fn save_history(app: tauri::AppHandle, entries: Vec<persistence::HistoryEntry>) {
    persistence::save(&app, &entries);
}
