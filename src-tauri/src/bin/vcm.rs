use clap::{Parser, Subcommand};
use vibes_copy_manager_lib::{autostart, config, engine};

#[derive(Parser)]
#[command(name = "vcm", version, about = "Vibes Copy Manager — clipboard manager CLI")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Push text to clipboard history and system clipboard
    Push {
        /// The text to push
        text: String,
    },

    /// Pop (retrieve) an item from clipboard history and copy to system clipboard
    Pop {
        /// Index of the item to retrieve (0 = latest, default)
        index: Option<usize>,
    },

    /// List clipboard history items
    List {
        /// Maximum number of items to show
        #[arg(short, long, default_value_t = 20)]
        limit: usize,
    },

    /// Clear clipboard history or a specific item
    Clear {
        /// Index of a specific item to remove (omit to clear all)
        index: Option<usize>,
    },

    /// View or update configuration
    Settings {
        #[command(subcommand)]
        action: Option<SettingsAction>,
    },
}

#[derive(Subcommand)]
enum SettingsAction {
    /// Show current configuration
    Show,

    /// Set the global shortcut (e.g. "Ctrl+Shift+V")
    Shortcut {
        /// Shortcut string, or "none" to disable
        value: String,
    },

    /// Set max history items
    MaxItems {
        /// Number of items to keep
        value: usize,
    },

    /// Enable or disable autostart
    Autostart {
        /// "on" or "off"
        value: String,
    },

    /// Set theme ("dark", "light", or "system")
    Theme {
        /// Theme value
        value: String,
    },
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn"))
        .format_timestamp(None)
        .init();

    let cli = Cli::parse();

    match cli.command {
        None => cmd_open_gui(),
        Some(Commands::Push { text }) => cmd_push(&text),
        Some(Commands::Pop { index }) => cmd_pop(index),
        Some(Commands::List { limit }) => cmd_list(limit),
        Some(Commands::Clear { index }) => cmd_clear(index),
        Some(Commands::Settings { action }) => cmd_settings(action),
    }
}

/// Open the GUI. If the GUI is already running, signal it to show its window
/// (the single-instance plugin handles this automatically — launching the same
/// binary again triggers the `single_instance` callback in the running instance).
/// If not running, start it.
fn cmd_open_gui() {
    let exe = std::env::current_exe().ok();
    let exe_dir = exe.as_ref().and_then(|p| p.parent());

    let gui_names: &[&str] = if cfg!(target_os = "windows") {
        &["vibes-copy-manager.exe", "vcm-gui.exe"]
    } else {
        &["vibes-copy-manager", "vcm-gui"]
    };

    let mut search_dirs: Vec<std::path::PathBuf> = Vec::new();
    if let Some(dir) = exe_dir {
        search_dirs.push(dir.to_path_buf());
    }
    if let Ok(path) = std::env::var("PATH") {
        for p in std::env::split_paths(&path) {
            search_dirs.push(p);
        }
    }

    // First, check if the GUI is already running by looking for the process
    if is_gui_running(gui_names) {
        // The GUI is running. Launch the binary again — the single-instance
        // plugin will detect the duplicate and fire the callback in the
        // existing instance (showing the window), then this new process exits.
        for dir in &search_dirs {
            for name in gui_names {
                let candidate = dir.join(name);
                if candidate.is_file() {
                    let status = std::process::Command::new(&candidate)
                        .stdin(std::process::Stdio::null())
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .status();

                    match status {
                        Ok(s) => {
                            // The single-instance plugin causes the second instance
                            // to exit quickly after signaling the first
                            if s.success() || s.code() == Some(1) {
                                return;
                            }
                        }
                        Err(e) => {
                            log::warn!("Failed to signal existing instance via {}: {e}", candidate.display());
                        }
                    }
                }
            }
        }
        // If we got here, we found the process but couldn't signal it
        println!("VCM is already running. The window should now be visible.");
        return;
    }

    // GUI is not running — launch it
    for dir in &search_dirs {
        for name in gui_names {
            let candidate = dir.join(name);
            if candidate.is_file() {
                let status = std::process::Command::new(&candidate)
                    .stdin(std::process::Stdio::null())
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .spawn();

                match status {
                    Ok(_) => {
                        println!("Launched VCM: {}", candidate.display());
                        return;
                    }
                    Err(e) => {
                        log::warn!("Failed to launch {}: {e}", candidate.display());
                    }
                }
            }
        }
    }

    eprintln!("Could not find the GUI binary (vibes-copy-manager or vcm-gui).");
    eprintln!("Install it: curl -sSL https://raw.githubusercontent.com/vibes4/vibes-copy-manager/master/install.sh | sh");
    eprintln!();
    eprintln!("Or use CLI commands directly:");
    eprintln!("  vcm push \"text\"");
    eprintln!("  vcm pop");
    eprintln!("  vcm list");
    eprintln!("  vcm settings");
    std::process::exit(1);
}

/// Check if the GUI binary is already running.
fn is_gui_running(gui_names: &[&str]) -> bool {
    #[cfg(target_os = "linux")]
    {
        // Use /proc to check for running process
        if let Ok(entries) = std::fs::read_dir("/proc") {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.join("exe").exists() {
                    continue;
                }
                if let Ok(exe_link) = std::fs::read_link(path.join("exe")) {
                    let exe_name = exe_link
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("");
                    if gui_names.contains(&exe_name) {
                        // Make sure it's not our own process
                        if let Some(pid_str) = entry.file_name().to_str() {
                            if let Ok(pid) = pid_str.parse::<u32>() {
                                if pid != std::process::id() {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
        false
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        for name in gui_names {
            if let Ok(output) = Command::new("pgrep").arg("-x").arg(name).output() {
                if output.status.success() && !output.stdout.is_empty() {
                    return true;
                }
            }
        }
        false
    }

    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        for name in gui_names {
            if let Ok(output) = Command::new("tasklist")
                .args(["/FI", &format!("IMAGENAME eq {name}"), "/NH"])
                .output()
            {
                let out = String::from_utf8_lossy(&output.stdout);
                if out.contains(name) {
                    return true;
                }
            }
        }
        false
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        let _ = gui_names;
        false
    }
}

fn cmd_push(text: &str) {
    let cfg = config::load();
    engine::push_text(text, cfg.max_items);

    if let Err(e) = engine::set_system_clipboard(text) {
        eprintln!("warning: could not set system clipboard: {e}");
    }

    println!("Pushed to clipboard history.");
}

fn cmd_pop(index: Option<usize>) {
    match engine::pop(index) {
        Some(entry) => {
            if entry.kind == "text" {
                if let Err(e) = engine::set_system_clipboard(&entry.content) {
                    eprintln!("warning: could not set system clipboard: {e}");
                }
                print!("{}", entry.content);
            } else {
                eprintln!(
                    "Item at index {} is an image (cannot output to terminal).",
                    index.unwrap_or(0)
                );
                std::process::exit(1);
            }
        }
        None => {
            eprintln!("No item at index {}.", index.unwrap_or(0));
            std::process::exit(1);
        }
    }
}

fn cmd_list(limit: usize) {
    let items = engine::list_items();
    if items.is_empty() {
        println!("Clipboard history is empty.");
        return;
    }

    let count = items.len().min(limit);
    for (i, entry) in items.iter().take(count).enumerate() {
        let pin = if entry.pinned { " [pinned]" } else { "" };
        let kind = if entry.kind == "image" { " [image]" } else { "" };
        let preview = if entry.kind == "text" {
            let s = entry.content.replace('\n', "\\n");
            if s.len() > 80 {
                format!("{}…", &s[..80])
            } else {
                s
            }
        } else {
            "(image data)".to_string()
        };
        println!("  {i:>3}  {preview}{kind}{pin}");
    }

    if items.len() > count {
        println!("  ... and {} more items", items.len() - count);
    }
}

fn cmd_clear(index: Option<usize>) {
    match index {
        Some(idx) => {
            let items = engine::list_items();
            if idx >= items.len() {
                eprintln!("No item at index {idx}.");
                std::process::exit(1);
            }
            engine::clear_index(idx);
            println!("Removed item at index {idx}.");
        }
        None => {
            engine::clear_all();
            println!("Cleared clipboard history (pinned items kept).");
        }
    }
}

fn cmd_settings(action: Option<SettingsAction>) {
    let mut cfg = config::load();

    match action {
        None | Some(SettingsAction::Show) => {
            println!("Configuration ({})\n", config::config_path().display());
            println!(
                "  shortcut:   {}",
                cfg.shortcut.as_deref().unwrap_or("(none)")
            );
            println!("  maxItems:   {}", cfg.max_items);
            println!("  autoStart:  {}", cfg.auto_start);
            println!("  theme:      {}", cfg.theme);
            println!("\nUse `vcm settings shortcut <value>` to change settings.");

            // Show platform-specific hints
            #[cfg(target_os = "linux")]
            {
                use vibes_copy_manager_lib::platform::LinuxDisplayBackend;
                let backend = LinuxDisplayBackend::detect();
                if backend.is_wayland() {
                    println!();
                    println!("  ⚠ Wayland detected: Global shortcuts may be unreliable.");
                    println!("  Recommended: Configure Super+V in your desktop environment");
                    println!("  to execute `vcm`. See README for setup instructions.");
                }
            }
        }
        Some(SettingsAction::Shortcut { value }) => {
            if value.to_lowercase() == "none" || value.to_lowercase() == "null" || value.is_empty()
            {
                cfg.shortcut = None;
                config::save(&cfg);
                println!("Shortcut disabled. Restart the GUI to apply.");
            } else {
                if let Err(e) = config::validate_shortcut(&value) {
                    eprintln!("Invalid shortcut: {e}");
                    std::process::exit(1);
                }
                cfg.shortcut = Some(value.clone());
                config::save(&cfg);
                println!("Shortcut set to: {value}. Restart the GUI to apply.");
            }
        }
        Some(SettingsAction::MaxItems { value }) => {
            if value < 10 {
                eprintln!("maxItems must be at least 10.");
                std::process::exit(1);
            }
            cfg.max_items = value;
            config::save(&cfg);
            println!("Max items set to: {value}.");
        }
        Some(SettingsAction::Autostart { value }) => {
            let enabled = match value.to_lowercase().as_str() {
                "on" | "true" | "1" | "yes" => true,
                "off" | "false" | "0" | "no" => false,
                _ => {
                    eprintln!("Use 'on' or 'off'.");
                    std::process::exit(1);
                }
            };
            cfg.auto_start = enabled;
            config::save(&cfg);

            if enabled {
                match std::env::current_exe() {
                    Ok(exe) => {
                        if let Err(e) = autostart::enable(&exe.to_string_lossy()) {
                            eprintln!("warning: could not enable autostart: {e}");
                        } else {
                            println!("Autostart enabled.");
                        }
                    }
                    Err(e) => eprintln!("warning: could not determine exe path: {e}"),
                }
            } else if let Err(e) = autostart::disable() {
                eprintln!("warning: could not disable autostart: {e}");
            } else {
                println!("Autostart disabled.");
            }
        }
        Some(SettingsAction::Theme { value }) => {
            match value.to_lowercase().as_str() {
                "dark" | "light" | "system" => {
                    cfg.theme = value.to_lowercase();
                    config::save(&cfg);
                    println!("Theme set to: {}. Restart the GUI to apply.", cfg.theme);
                }
                _ => {
                    eprintln!("Invalid theme. Use 'dark', 'light', or 'system'.");
                    std::process::exit(1);
                }
            }
        }
    }
}
