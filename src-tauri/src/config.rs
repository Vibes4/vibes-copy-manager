use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const CONFIG_DIR: &str = "vibes-copy-manager";
const CONFIG_FILE: &str = "config.json";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppConfig {
    pub shortcut: Option<String>,
    #[serde(rename = "maxItems")]
    pub max_items: usize,
    #[serde(rename = "autoStart", default)]
    pub auto_start: bool,
    #[serde(default = "default_theme")]
    pub theme: String,
}

fn default_theme() -> String {
    "dark".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            shortcut: None,
            max_items: 50,
            auto_start: false,
            theme: default_theme(),
        }
    }
}

pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(CONFIG_DIR)
        .join(CONFIG_FILE)
}

pub fn exists() -> bool {
    config_path().exists()
}

pub fn load() -> AppConfig {
    let path = config_path();
    match fs::read_to_string(&path) {
        Ok(data) => match serde_json::from_str(&data) {
            Ok(cfg) => cfg,
            Err(e) => {
                log::warn!("Malformed config at {}: {e}, using defaults", path.display());
                AppConfig::default()
            }
        },
        Err(_) => {
            let cfg = AppConfig::default();
            save(&cfg);
            cfg
        }
    }
}

pub fn save(cfg: &AppConfig) {
    let path = config_path();
    if let Some(parent) = path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            log::error!("Could not create config dir {}: {e}", parent.display());
            return;
        }
    }
    match serde_json::to_string_pretty(cfg) {
        Ok(json) => {
            if let Err(e) = fs::write(&path, json) {
                log::error!("Could not write config to {}: {e}", path.display());
            }
        }
        Err(e) => {
            log::error!("Could not serialize config: {e}");
        }
    }
}

#[cfg(feature = "gui")]
pub fn parse_shortcut(
    s: &str,
) -> Result<
    (
        tauri_plugin_global_shortcut::Modifiers,
        tauri_plugin_global_shortcut::Code,
    ),
    String,
> {
    use tauri_plugin_global_shortcut::{Code, Modifiers};

    let parts: Vec<&str> = s.split('+').map(|p| p.trim()).collect();
    if parts.is_empty() {
        return Err("empty shortcut".into());
    }

    let mut mods = Modifiers::empty();
    let mut key_part: Option<&str> = None;

    for part in &parts {
        match part.to_lowercase().as_str() {
            "ctrl" | "control" => mods |= Modifiers::CONTROL,
            "shift" => mods |= Modifiers::SHIFT,
            "alt" => mods |= Modifiers::ALT,
            "super" | "meta" | "cmd" | "command" | "win" | "windows" | "window" => mods |= Modifiers::SUPER,
            _ => key_part = Some(part),
        }
    }

    let key_str = key_part.ok_or("no key specified in shortcut")?;

    let code = match key_str.to_uppercase().as_str() {
        "A" => Code::KeyA, "B" => Code::KeyB, "C" => Code::KeyC,
        "D" => Code::KeyD, "E" => Code::KeyE, "F" => Code::KeyF,
        "G" => Code::KeyG, "H" => Code::KeyH, "I" => Code::KeyI,
        "J" => Code::KeyJ, "K" => Code::KeyK, "L" => Code::KeyL,
        "M" => Code::KeyM, "N" => Code::KeyN, "O" => Code::KeyO,
        "P" => Code::KeyP, "Q" => Code::KeyQ, "R" => Code::KeyR,
        "S" => Code::KeyS, "T" => Code::KeyT, "U" => Code::KeyU,
        "V" => Code::KeyV, "W" => Code::KeyW, "X" => Code::KeyX,
        "Y" => Code::KeyY, "Z" => Code::KeyZ,
        "0" => Code::Digit0, "1" => Code::Digit1, "2" => Code::Digit2,
        "3" => Code::Digit3, "4" => Code::Digit4, "5" => Code::Digit5,
        "6" => Code::Digit6, "7" => Code::Digit7, "8" => Code::Digit8,
        "9" => Code::Digit9,
        "SPACE" => Code::Space, "TAB" => Code::Tab,
        "BACKQUOTE" | "`" => Code::Backquote,
        "F1" => Code::F1, "F2" => Code::F2, "F3" => Code::F3,
        "F4" => Code::F4, "F5" => Code::F5, "F6" => Code::F6,
        "F7" => Code::F7, "F8" => Code::F8, "F9" => Code::F9,
        "F10" => Code::F10, "F11" => Code::F11, "F12" => Code::F12,
        other => return Err(format!("unknown key: {other}")),
    };

    if mods.is_empty() {
        return Err("at least one modifier (Ctrl/Alt/Shift/Super) is required".into());
    }

    Ok((mods, code))
}

pub fn validate_shortcut(s: &str) -> Result<(), String> {
    let parts: Vec<&str> = s.split('+').map(|p| p.trim()).collect();
    if parts.is_empty() {
        return Err("empty shortcut".into());
    }

    let mut has_modifier = false;
    let mut has_key = false;

    for part in &parts {
        match part.to_lowercase().as_str() {
            "ctrl" | "control" | "shift" | "alt" | "super" | "meta" | "cmd" | "command" | "win" | "windows" | "window" => {
                has_modifier = true;
            }
            k => {
                let upper = k.to_uppercase();
                let valid = upper.len() == 1 && upper.chars().next().is_some_and(|c| c.is_alphanumeric())
                    || matches!(upper.as_str(),
                        "SPACE" | "TAB" | "BACKQUOTE" |
                        "F1" | "F2" | "F3" | "F4" | "F5" | "F6" |
                        "F7" | "F8" | "F9" | "F10" | "F11" | "F12"
                    );
                if !valid {
                    return Err(format!("unknown key: {k}"));
                }
                has_key = true;
            }
        }
    }

    if !has_modifier {
        return Err("at least one modifier (Ctrl/Alt/Shift/Super) is required".into());
    }
    if !has_key {
        return Err("no key specified in shortcut".into());
    }
    Ok(())
}
