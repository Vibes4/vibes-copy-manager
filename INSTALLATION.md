# Installation Guide

## Quick Install (Linux / macOS)

Installs both CLI (`vcm`) and GUI (`vcm-gui`) to `~/.local/bin/`:

```bash
curl -sSL https://raw.githubusercontent.com/vibes4/vibes-copy-manager/master/install.sh | sh
```

Verify:

```bash
vcm --help
vcm          # opens GUI
```

---

## Linux

### AppImage (recommended)

1. Download `vcm-linux.AppImage` from [Releases](https://github.com/vibes4/vibes-copy-manager/releases/latest)

2. Make it executable and run:

```bash
chmod +x vcm-linux.AppImage
./vcm-linux.AppImage
```

3. (Optional) Move to a standard location:

```bash
mkdir -p ~/.local/bin
mv vcm-linux.AppImage ~/.local/bin/vcm-gui
```

### .deb Package (Debian/Ubuntu)

```bash
sudo dpkg -i vcm-linux.deb
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

---

## macOS

### DMG

1. Download `vcm-macos.dmg` from [Releases](https://github.com/vibes4/vibes-copy-manager/releases/latest)
2. Open the `.dmg` and drag the app to **Applications**
3. First launch: right-click the app and select **Open** (bypasses Gatekeeper on unsigned builds)

### Permissions

macOS requires Accessibility permission for auto-paste (via `osascript`):

1. Open **System Settings** → **Privacy & Security** → **Accessibility**
2. Add **Vibes Copy Manager** to the allowed list

---

## Windows

### MSI Installer

1. Download `vcm-windows.msi` from [Releases](https://github.com/vibes4/vibes-copy-manager/releases/latest)
2. Double-click to run the installer
3. If Windows SmartScreen appears, click **More info** → **Run anyway**
4. Launch from the Start Menu

### CLI on Windows

Download `vcm-windows.exe` from the release and add it to a folder in your PATH.

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

### Set theme

```bash
vcm settings theme dark     # or light, system
```

Or toggle in the GUI Settings panel.

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

### CLI + GUI (curl install)

```bash
rm ~/.local/bin/vcm ~/.local/bin/vcm-gui
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
