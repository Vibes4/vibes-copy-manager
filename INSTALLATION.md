# Installation Guide

## Quick Install (CLI)

```bash
curl -sSL https://raw.githubusercontent.com/vibes4/vibes-copy-manager/master/install.sh | sh
```

This installs the `vcm` CLI binary to `~/.local/bin/`. If a pre-built binary isn't available for your platform, it builds from source (requires Rust).

Verify:

```bash
vcm --help
```

---

## Linux

### AppImage (recommended)

1. Download the `.AppImage` from [Releases](https://github.com/vibes4/vibes-copy-manager/releases/latest)

2. Make it executable and run:

```bash
chmod +x vibes-copy-manager_*.AppImage
./vibes-copy-manager_*.AppImage
```

3. (Optional) Move to a standard location:

```bash
mkdir -p ~/.local/bin
mv vibes-copy-manager_*.AppImage ~/.local/bin/vibes-copy-manager
```

### .deb Package (Debian/Ubuntu)

```bash
sudo dpkg -i vibes-copy-manager_*.deb
```

### System Dependencies

For auto-paste to work, install the appropriate tool:

**X11:**

```bash
sudo apt install -y xdotool
```

**Wayland:**

```bash
sudo apt install -y wtype
```

### Building the GUI from Source

```bash
sudo apt install -y \
  libwebkit2gtk-4.1-dev \
  libgtk-3-dev \
  libappindicator3-dev \
  librsvg2-dev \
  xdotool

cargo install tauri-cli --version "^2"
cargo tauri build
```

### Shortcut Conflicts

If your desktop environment captures the shortcut before the app:

1. Remove or change the conflicting shortcut in your DE settings, or
2. Change the app shortcut: `vcm settings shortcut "Alt+V"`

---

## macOS

### DMG

1. Download the `.dmg` from [Releases](https://github.com/vibes4/vibes-copy-manager/releases/latest)
2. Open the `.dmg` and drag the app to **Applications**
3. First launch: right-click the app and select **Open** (bypasses Gatekeeper on unsigned builds)

### Permissions

macOS requires Accessibility permission for auto-paste (via `osascript`):

1. Open **System Settings** → **Privacy & Security** → **Accessibility**
2. Add **Vibes Copy Manager** to the allowed list

### CLI on macOS

The `curl | sh` installer works on macOS. Alternatively, after installing the `.dmg`, the CLI binary can be built separately:

```bash
cd src-tauri
cargo build --release --bin vcm --no-default-features
cp target/release/vcm ~/.local/bin/
```

---

## Windows

### MSI Installer

1. Download the `.msi` from [Releases](https://github.com/vibes4/vibes-copy-manager/releases/latest)
2. Double-click to run the installer
3. If Windows SmartScreen appears, click **More info** → **Run anyway**
4. Launch from the Start Menu

### CLI on Windows

Copy `vcm.exe` to a folder in your PATH, or add its location to PATH via System Settings.

The `curl | sh` installer does not support Windows. Use the `.msi` installer instead.

---

## Building from Source

### Prerequisites (all platforms)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo install tauri-cli --version "^2"
```

### Build CLI Only

No GUI dependencies required:

```bash
cd src-tauri
cargo build --release --bin vcm --no-default-features
```

Binary: `src-tauri/target/release/vcm`

### Build GUI

```bash
cargo tauri build
```

Output in `src-tauri/target/release/bundle/`:

| Platform | Formats |
|----------|---------|
| Linux | `.deb`, `.AppImage` |
| macOS | `.dmg` |
| Windows | `.msi` |

---

## Post-Install Configuration

### Set a shortcut

```bash
vcm settings shortcut "Ctrl+Shift+V"
```

Or open the GUI → Settings (gear icon) → set your shortcut → Save.

### Enable autostart

```bash
vcm settings autostart on
```

| Platform | Autostart location |
|----------|--------------------|
| Linux | `~/.config/autostart/vibes-copy-manager.desktop` |
| macOS | `~/Library/LaunchAgents/com.vibes.vibes-copy-manager.plist` |
| Windows | `%APPDATA%\...\Startup\vibes-copy-manager.bat` |

### Verify

```bash
vcm push "test"
vcm list
vcm pop
```

---

## Uninstall

### CLI

```bash
rm ~/.local/bin/vcm
```

### GUI

| Platform | Method |
|----------|--------|
| Linux (.deb) | `sudo apt remove vibes-copy-manager` |
| Linux (AppImage) | Delete the file |
| macOS | Drag from Applications to Trash |
| Windows | Add/Remove Programs |

### Remove data

```bash
rm -rf ~/.config/vibes-copy-manager
rm -rf ~/.local/share/vibes-copy-manager
```
