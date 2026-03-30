# Vibes Copy Manager (vcm)

A fast, modern clipboard manager for Linux, Windows, and macOS. Built with Tauri, Rust, and Vanilla JS. Includes both a **GUI popup** and a **CLI tool** (`vcm`).

## Features

- **Clipboard history** — automatically captures text and images
- **CLI tool (`vcm`)** — push, pop, list, clear from the terminal
- **Instant search** — filter 1000+ items with debounced substring matching
- **Pin items** — keep important clips at the top, safe from history trimming
- **Image support** — previews clipboard images as thumbnails
- **Configurable shortcut** — change the global hotkey from UI or CLI (or disable it)
- **Auto-paste** — selecting an item writes it to clipboard and simulates Ctrl+V
- **Start on login** — optional autostart on system boot
- **System tray** — runs in the background with a tray icon
- **Single instance** — launching again shows the existing window
- **Dark theme** — clean, compact popup UI
- **Shared data** — CLI and GUI read/write the same clipboard history

## Quick Install (CLI only)

```bash
curl -sSL https://raw.githubusercontent.com/vaibhav/vibes-copy-manager/main/install.sh | sh
```

Verify:

```bash
vcm --help
```

## CLI Usage

### Push text to clipboard

```bash
vcm push "Hello, world!"
```

Adds text to history and copies it to the system clipboard.

### Pop (retrieve) latest item

```bash
vcm pop
```

Copies the latest item to the system clipboard and outputs it.

### Pop by index

```bash
vcm pop 3
```

### List clipboard history

```bash
vcm list
vcm list --limit 50
```

### Clear history

```bash
vcm clear          # Clear all (keeps pinned)
vcm clear 2        # Remove item at index 2
```

### Settings

```bash
vcm settings                              # Show current config
vcm settings show                         # Same as above
vcm settings shortcut "Ctrl+Shift+V"     # Set shortcut
vcm settings shortcut none                # Disable shortcut
vcm settings max-items 100                # Set max history
vcm settings autostart on                 # Enable autostart
vcm settings autostart off                # Disable autostart
```

## GUI Usage

### Opening the App

Press the global shortcut (default: **Ctrl+Shift+V** on Linux/Windows, **Cmd+Shift+V** on macOS).

The popup appears near your cursor. It hides when you:
- Press **Esc**
- Click outside the window
- Select an item

You can also click the **system tray icon** to toggle the window.

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| **Up / Down** | Navigate items |
| **Enter** | Paste selected item |
| **Esc** | Hide window |
| **Ctrl+P** | Toggle pin on selected item |
| **Delete** | Remove selected item |

### Settings

Click the gear icon in the header to open settings:

- **Global Shortcut** — set any Ctrl/Shift/Alt/Super + key combination, or leave empty to disable
- **Max History Items** — how many items to keep (10-5000)
- **Start on system login** — toggle autostart

## Configuration

Settings are stored in:

```
~/.config/vibes-copy-manager/config.json
```

Example:

```json
{
  "shortcut": "Ctrl+Shift+V",
  "maxItems": 50,
  "autoStart": false
}
```

Set `shortcut` to `null` to disable the global shortcut (use tray icon or `vcm` CLI instead).

Clipboard history is stored in:

```
~/.local/share/vibes-copy-manager/clipboard_history.json
```

Both the CLI and GUI share the same history file.

## Development

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) (for Tauri CLI)
- Tauri CLI: `cargo install tauri-cli --version "^2"`
- **Linux only**: `xdotool` (X11) or `wtype` (Wayland) for auto-paste
- **Linux only**: `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `libappindicator3-dev`

### Run GUI in Development

```bash
cargo tauri dev
```

The window starts hidden. Press your configured shortcut to open it, or click the system tray icon.

### Build CLI Only (no Tauri/GUI dependencies)

```bash
cd src-tauri
cargo build --release --bin vcm --no-default-features
```

The binary is at `target/release/vcm`.

### Build GUI

```bash
cargo tauri build
```

### Build Both

```bash
# GUI (includes all Tauri features)
cargo tauri build

# CLI (standalone, no GUI deps)
cd src-tauri
cargo build --release --bin vcm --no-default-features
```

Output binaries:

| Binary | Location |
|--------|----------|
| GUI | `src-tauri/target/release/bundle/` (.deb, .AppImage, .msi, .dmg) |
| CLI (`vcm`) | `src-tauri/target/release/vcm` |

## Architecture

```
src/                        # Frontend (Vanilla JS + Tailwind)
├── index.html              # Main UI
├── app.js                  # UI logic, events, settings
├── clipboard.js            # In-memory history management
└── styles.css              # Custom CSS + animations

src-tauri/src/              # Rust backend
├── main.rs                 # GUI entry point
├── bin/vcm.rs              # CLI entry point (vcm binary)
├── lib.rs                  # Tauri setup, commands, tray, shortcuts
├── engine.rs               # Shared clipboard engine (history CRUD)
├── clipboard.rs            # Clipboard watcher (GUI, polling + images)
├── window.rs               # Window positioning, show/hide, paste
├── config.rs               # Config read/write + shortcut parsing
├── persistence.rs          # Tauri-specific history persistence
├── autostart.rs            # Cross-platform autostart management
└── build.rs                # Build script (conditional Tauri build)
```

### Shared vs GUI-only modules

| Module | Used by CLI | Used by GUI |
|--------|:-----------:|:-----------:|
| `engine.rs` | Yes | Yes |
| `config.rs` | Yes | Yes |
| `autostart.rs` | Yes | Yes |
| `clipboard.rs` | - | Yes |
| `window.rs` | - | Yes |
| `persistence.rs` | - | Yes |

The `gui` feature flag controls Tauri-dependent code. The CLI binary uses `--no-default-features` to compile without Tauri.

## Platform Notes

### Linux

- **X11**: Uses `xdotool` for window focus and paste simulation
- **Wayland**: Falls back to `wtype` for paste simulation
- Autostart creates `~/.config/autostart/vibes-copy-manager.desktop`

### macOS

- Uses `osascript` for paste simulation (System Events keystroke)
- Autostart creates a LaunchAgent plist
- Default shortcut uses **Cmd** instead of Ctrl

### Windows

- Clipboard write is sufficient — user pastes with their normal Ctrl+V
- Autostart creates a `.bat` in the Startup folder

## License

MIT
