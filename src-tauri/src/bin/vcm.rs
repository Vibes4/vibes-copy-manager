use clap::{Parser, Subcommand};
use vibes_copy_manager_lib::{autostart, config, engine};

#[derive(Parser)]
#[command(name = "vcm", version, about = "Vibes Copy Manager — clipboard manager CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
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
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Push { text } => cmd_push(&text),
        Commands::Pop { index } => cmd_pop(index),
        Commands::List { limit } => cmd_list(limit),
        Commands::Clear { index } => cmd_clear(index),
        Commands::Settings { action } => cmd_settings(action),
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
                eprintln!("Item at index {} is an image (cannot output to terminal).", index.unwrap_or(0));
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
            println!("\nUse `vcm settings shortcut <value>` to change settings.");
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
            } else {
                if let Err(e) = autostart::disable() {
                    eprintln!("warning: could not disable autostart: {e}");
                } else {
                    println!("Autostart disabled.");
                }
            }
        }
    }
}
