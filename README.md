# Vibes Copy Manager (vcm)

A fast, cross-platform clipboard manager with a GUI popup and CLI tool. Built with Tauri, Rust, and Vanilla JS.

## Features

- **Clipboard history** — automatically captures text and images
- **Instant search** — filter thousands of items with debounced matching
- **Pin items** — keep important clips at the top, safe from trimming
- **Image preview** — clipboard images shown as thumbnails
- **Configurable shortcut** — set any global hotkey from UI or CLI
- **CLI support** — push, pop, list, clear, and configure from the terminal
- **Cross-platform** — Linux (X11 + Wayland), macOS, and Windows
- **Auto-paste** — selecting an item writes to clipboard and simulates paste
- **System tray** — runs in the background with a tray icon
- **Single instance** — launching again shows the existing window
- **Autostart** — optional start on system login

---

## Installation

### One-line install (CLI)

```bash
curl -sSL https://raw.githubusercontent.com/vibes4/vibes-copy-manager/master/install.sh | sh
```

This installs the `vcm` CLI binary to `~/.local/bin/`.

### Manual install (GUI + CLI)

Download the latest release for your platform:

| Platform | Package | Link |
|----------|---------|------|
| Linux | `.AppImage` | [Download](https://github.com/vibes4/vibes-copy-manager/releases/latest) |
| Linux | `.deb` | [Download](https://github.com/vibes4/vibes-copy-manager/releases/latest) |
| macOS | `.dmg` | [Download](https://github.com/vibes4/vibes-copy-manager/releases/latest) |
| Windows | `.msi` | [Download](https://github.com/vibes4/vibes-copy-manager/releases/latest) |

See [INSTALLATION.md](INSTALLATION.md) for detailed OS-specific instructions.

---

## Usage

Launch the GUI (if installed):

```bash
vcm
```

### CLI Commands

```bash
vcm push "Hello, world!"       # Add text to history + system clipboard
vcm pop                         # Copy latest item to clipboard, print it
vcm pop 3                       # Copy item at index 3
vcm list                        # List clipboard history
vcm list --limit 50             # List up to 50 items
vcm clear                       # Clear all (keeps pinned)
vcm clear 2                     # Remove item at index 2
vcm settings                    # Show current config
vcm settings shortcut "Ctrl+Shift+V"   # Set shortcut
vcm settings shortcut none      # Disable shortcut
vcm settings max-items 100      # Set max history items
vcm settings autostart on       # Enable autostart
vcm settings autostart off      # Disable autostart
```

### GUI

Press your configured shortcut or click the system tray icon to open the popup.

| Key | Action |
|-----|--------|
| **Up / Down** | Navigate items |
| **Enter** | Paste selected item |
| **Esc** | Hide window |
| **Ctrl+P** | Toggle pin on selected item |
| **Delete** | Remove selected item |

---

## Shortcut Setup

The app ships with **no default shortcut**. On first launch, the Settings panel opens so you can configure one.

You can set it via CLI or the GUI Settings panel:

```bash
vcm settings shortcut "Ctrl+Shift+V"
```

**Recommended shortcuts by platform:**

| Platform | Shortcut | Notes |
|----------|----------|-------|
| Linux | `Super+V` or `Ctrl+Shift+V` | Set via OS keyboard settings or vcm |
| macOS | `Cmd+Shift+V` | Set via vcm settings |
| Windows | `Ctrl+Shift+V` | Set via vcm settings |

Leave the shortcut empty to disable it entirely — use the tray icon or `vcm` CLI instead.

---

## Configuration

Settings are stored at:

```
~/.config/vibes-copy-manager/config.json
```

```json
{
  "shortcut": "Ctrl+Shift+V",
  "maxItems": 50,
  "autoStart": false
}
```

Set `shortcut` to `null` to disable the global shortcut.

Clipboard history is stored at:

```
~/.local/share/vibes-copy-manager/clipboard_history.json
```

Both the CLI and GUI share the same history file.

---

## Development

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) (for Tauri CLI)
- Tauri CLI: `cargo install tauri-cli --version "^2"`
- **Linux**: `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `libappindicator3-dev`, `xdotool`
- **Linux (Wayland)**: `wtype`

### Run in development

```bash
cargo tauri dev
```

### Build CLI only (no GUI dependencies)

```bash
cd src-tauri
cargo build --release --bin vcm --no-default-features
```

### Build GUI

```bash
cargo tauri build
```

Output:

| Binary | Location |
|--------|----------|
| GUI | `src-tauri/target/release/bundle/` (.deb, .AppImage, .msi, .dmg) |
| CLI | `src-tauri/target/release/vcm` |

---

## Architecture

```
src/                        Frontend (Vanilla JS + Tailwind)
├── index.html              Main UI
├── app.js                  UI logic, events, settings
├── clipboard.js            In-memory history management
└── styles.css              Custom CSS + animations

src-tauri/src/              Rust backend
├── main.rs                 GUI entry point
├── bin/vcm.rs              CLI entry point
├── lib.rs                  Tauri setup, commands, tray, shortcuts
├── engine.rs               Shared clipboard engine (history CRUD)
├── clipboard.rs            Clipboard watcher (polling + images)
├── window.rs               Window positioning, show/hide, paste
├── config.rs               Config read/write + shortcut parsing
├── persistence.rs          Tauri-specific history persistence
└── autostart.rs            Cross-platform autostart management
```

| Module | CLI | GUI |
|--------|:---:|:---:|
| `engine.rs` | Yes | Yes |
| `config.rs` | Yes | Yes |
| `autostart.rs` | Yes | Yes |
| `clipboard.rs` | — | Yes |
| `window.rs` | — | Yes |
| `persistence.rs` | — | Yes |

The `gui` feature flag controls Tauri-dependent code. The CLI compiles with `--no-default-features`.

---

## Platform Notes

### Linux
- **X11**: Uses `xdotool` for window focus and paste simulation
- **Wayland**: Falls back to `wtype` for paste simulation
- Autostart: `~/.config/autostart/vibes-copy-manager.desktop`

### macOS
- Uses `osascript` for paste simulation
- Autostart: `~/Library/LaunchAgents/com.vibes.vibes-copy-manager.plist`
- Requires Accessibility permission for auto-paste

### Windows
- Clipboard write is sufficient — user pastes with Ctrl+V
- Autostart: `.bat` in the Startup folder

---

## Releases

[View all releases](https://github.com/vibes4/vibes-copy-manager/releases)

---

## License

MIT
