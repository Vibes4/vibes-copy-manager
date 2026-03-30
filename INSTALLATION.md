# Installation Guide

## One-Line Install (CLI only)

```bash
curl -sSL https://raw.githubusercontent.com/vaibhav/vibes-copy-manager/main/install.sh | sh
```

This installs the `vcm` CLI binary to `~/.local/bin/`. If the binary isn't found for your platform, it will build from source (requires Rust).

Verify:

```bash
vcm --help
```

---

## Building from Source

### Prerequisites

All platforms require Rust:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Build CLI Only (no GUI dependencies)

```bash
cd src-tauri
cargo build --release --bin vcm --no-default-features
```

Binary: `src-tauri/target/release/vcm`

Install it:

```bash
cp src-tauri/target/release/vcm ~/.local/bin/
```

### Build GUI

Requires Tauri CLI:

```bash
cargo install tauri-cli --version "^2"
cargo tauri build
```

Binaries are output to `src-tauri/target/release/bundle/`.

---

## Linux

### GUI Dependencies

Install system dependencies (Ubuntu/Debian):

```bash
sudo apt update
sudo apt install -y \
  libwebkit2gtk-4.1-dev \
  libgtk-3-dev \
  libappindicator3-dev \
  librsvg2-dev \
  xdotool
```

For Wayland, also install:

```bash
sudo apt install -y wtype
```

### AppImage

```bash
chmod +x vibes-copy-manager_*.AppImage
./vibes-copy-manager_*.AppImage
```

Optional — move to a standard location:

```bash
mkdir -p ~/.local/bin
mv vibes-copy-manager_*.AppImage ~/.local/bin/vibes-copy-manager
```

### .deb Package

```bash
sudo dpkg -i vibes-copy-manager_*.deb
```

### Setting Up the Global Shortcut

The app registers its own global shortcut. If your desktop environment captures that key first:

1. Remove or change the conflicting shortcut in your DE settings, or
2. Change the app shortcut via GUI Settings or `vcm settings shortcut "Alt+V"`

---

## Windows

### MSI Installer

1. Double-click `vibes-copy-manager_*.msi`
2. Follow the installation wizard
3. Launch from Start Menu

### Portable EXE

Copy `vibes-copy-manager.exe` and run it directly.

### CLI on Windows

Copy `vcm.exe` to a folder in your PATH, or add its location to PATH.

---

## macOS

### DMG

1. Double-click `vibes-copy-manager_*.dmg`
2. Drag the app to **Applications**
3. First launch: right-click -> **Open** (to bypass Gatekeeper)

### Notes

- macOS may ask for Accessibility permissions (needed for paste via `osascript`)
- Grant permission in **System Settings -> Privacy & Security -> Accessibility**
- Default shortcut: **Cmd+Shift+V**

---

## Post-Install

### Configure via CLI

```bash
# View current settings
vcm settings

# Set shortcut
vcm settings shortcut "Ctrl+Shift+V"

# Disable shortcut (use tray icon only)
vcm settings shortcut none

# Set max history items
vcm settings max-items 100

# Enable autostart
vcm settings autostart on
```

### Configure via GUI

Open the app -> Settings (gear icon) -> adjust settings -> Save.

### Enable Autostart

```bash
vcm settings autostart on
```

Or toggle in GUI Settings.

This creates a platform-appropriate autostart entry:

| Platform | Location |
|----------|----------|
| Linux | `~/.config/autostart/vibes-copy-manager.desktop` |
| macOS | `~/Library/LaunchAgents/com.vibes.vibes-copy-manager.plist` |
| Windows | `%APPDATA%\...\Startup\vibes-copy-manager.bat` |

### Verify It Works

```bash
# CLI
vcm push "test"
vcm list
vcm pop

# GUI
# Press your configured shortcut or click the tray icon
```

---

## Uninstall

### CLI

```bash
rm ~/.local/bin/vcm
```

### GUI

**Linux (.deb):**
```bash
sudo apt remove clipboard-manager
```

**Linux (AppImage):** Delete the file.

**Windows:** Use Add/Remove Programs.

**macOS:** Drag from Applications to Trash.

### Remove config and data

```bash
rm -rf ~/.config/vibes-copy-manager
rm -rf ~/.local/share/vibes-copy-manager
```
