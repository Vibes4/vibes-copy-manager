# Clipboard Manager

A fast, keyboard-first clipboard manager built with Tauri v2 + Rust + Vanilla JS.

Works like CopyQ / Raycast clipboard — runs in the background, watches your clipboard, and gives you instant access to your copy history.

---

## Quick Start

```bash
cargo tauri dev
```

The app starts **hidden in the system tray** — this is by design.
You will see a tray icon appear in your system tray / notification area.

### Open the window

Press **`Ctrl+Shift+V`** to toggle the clipboard popup.

Or click the **tray icon** (left-click toggles, right-click shows menu).

---

## Usage

| Action | How |
|---|---|
| **Open / toggle window** | `Ctrl+Shift+V` (global, works from any app) |
| **Navigate items** | `↑` / `↓` arrow keys |
| **Paste selected item** | `Enter` (copies to clipboard and hides window) |
| **Hide window** | `Esc`, click outside, or `Ctrl+Shift+V` again |
| **Search history** | Just start typing in the search bar |
| **Pin an item** | Hover over item → click bookmark icon |
| **Delete an item** | Hover over item → click X icon |
| **Clear all history** | Click trash icon in title bar (pinned items are kept) |
| **Quit the app** | Right-click tray icon → Quit |

---

## How It Works

1. The app runs as a **background process** with a system tray icon
2. A Rust thread polls the system clipboard every 300ms for changes
3. New clipboard text is added to the history (duplicates move to top)
4. When you open the popup, you see your full history
5. Selecting an item copies it back to the clipboard and hides the window
6. History is persisted to disk as JSON and restored on next launch

---

## Key Behaviors

- **Single instance** — launching the app again just shows the existing window
- **Window hides, never closes** — Esc / click-outside / close button all hide the window; the app keeps running
- **Always on top** — the popup appears above all windows
- **Keyboard-first** — arrow keys + Enter; no mouse required
- **Clipboard watcher** — detects copies from any application (browser, terminal, editors)

---

## Project Structure

```
src/                            Frontend (Vanilla JS + Tailwind)
├── index.html                  Popup UI
├── app.js                      Tauri bridge, keyboard nav, rendering
├── clipboard.js                ClipboardHistory class (dedup, pin, search)
└── styles.css                  Custom styles

src-tauri/src/                  Backend (Rust)
├── main.rs                     Binary entry point
├── lib.rs                      App setup: tray, shortcuts, single-instance
├── clipboard.rs                System clipboard polling + write command
├── window.rs                   Show / hide / toggle commands
└── persistence.rs              JSON file persistence
```

---

## Build for Production

```bash
cargo tauri build
```

The binary will be in `src-tauri/target/release/`. On Linux it also generates `.deb` and `.AppImage` packages in `src-tauri/target/release/bundle/`.

---

## Requirements

- Rust 1.77.2+
- System dependencies for Tauri v2 on Linux:
  ```bash
  sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
  ```
- **Auto-paste dependency** (required for Enter/click to paste into the previous app):
  ```bash
  # X11 (most systems)
  sudo apt install xdotool

  # Wayland
  sudo apt install wtype
  ```
  Without this, selecting an item will still copy it to the clipboard, but you'll need to manually Ctrl+V in the target app.
