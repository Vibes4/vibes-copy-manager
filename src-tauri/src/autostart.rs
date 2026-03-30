use std::fs;
use std::path::PathBuf;

const APP_NAME: &str = "vibes-copy-manager";

/// Returns whether autostart is currently enabled on this platform.
pub fn is_enabled() -> bool {
    if let Some(path) = autostart_path() {
        path.exists()
    } else {
        false
    }
}

/// Enable autostart. Returns Ok(()) on success, Err with reason on failure.
pub fn enable(exe_path: &str) -> Result<(), String> {
    let path = autostart_path().ok_or("autostart not supported on this platform")?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("create dir: {e}"))?;
    }

    let contents = platform_entry(exe_path);
    fs::write(&path, contents).map_err(|e| format!("write autostart: {e}"))?;

    // On Linux, make .desktop file executable
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o755));
    }

    Ok(())
}

/// Disable autostart by removing the autostart entry.
pub fn disable() -> Result<(), String> {
    let path = autostart_path().ok_or("autostart not supported on this platform")?;
    if path.exists() {
        fs::remove_file(&path).map_err(|e| format!("remove autostart: {e}"))?;
    }
    Ok(())
}

/// Platform-specific path for the autostart entry.
fn autostart_path() -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        dirs::config_dir().map(|d| d.join("autostart").join(format!("{APP_NAME}.desktop")))
    }

    #[cfg(target_os = "macos")]
    {
        dirs::home_dir().map(|d| {
            d.join("Library/LaunchAgents")
                .join(format!("com.vibes.{APP_NAME}.plist"))
        })
    }

    #[cfg(target_os = "windows")]
    {
        dirs::data_dir().map(|d| {
            d.join("Microsoft\\Windows\\Start Menu\\Programs\\Startup")
                .join(format!("{APP_NAME}.bat"))
        })
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        None
    }
}

/// Platform-specific autostart file content.
fn platform_entry(exe_path: &str) -> String {
    #[cfg(target_os = "linux")]
    {
        format!(
            "[Desktop Entry]\n\
             Type=Application\n\
             Name=Clipboard Manager\n\
             Exec={exe_path}\n\
             Icon=clipboard\n\
             Comment=Vibes Clipboard Manager\n\
             X-GNOME-Autostart-enabled=true\n\
             StartupNotify=false\n\
             Terminal=false\n"
        )
    }

    #[cfg(target_os = "macos")]
    {
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \
             \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
             <plist version=\"1.0\">\n\
             <dict>\n\
             \t<key>Label</key>\n\
             \t<string>com.vibes.clipboard-manager</string>\n\
             \t<key>ProgramArguments</key>\n\
             \t<array>\n\
             \t\t<string>{exe_path}</string>\n\
             \t</array>\n\
             \t<key>RunAtLoad</key>\n\
             \t<true/>\n\
             </dict>\n\
             </plist>\n"
        )
    }

    #[cfg(target_os = "windows")]
    {
        format!("@echo off\r\nstart \"\" \"{exe_path}\"\r\n")
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        let _ = exe_path;
        String::new()
    }
}
