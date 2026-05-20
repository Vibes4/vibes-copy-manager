# `DESIGN.md` — vibes-copy-manager (VCM)

````md
# vibes-copy-manager (VCM)

Modern cross-platform clipboard manager built using:
- Rust
- Tauri v2
- Vanilla JS
- Tailwind CSS

VCM is designed to be:
- fast
- keyboard-first
- lightweight
- cross-platform
- extensible
- developer-friendly

---

# 1. Vision

VCM is intended to be a modern replacement for:
- CopyQ
- clipboard history managers
- lightweight productivity launchers

Core philosophy:
- instant access
- minimal UI
- background utility
- low CPU usage
- strong CLI integration

---

# 2. High-Level Architecture

```text
+------------------------------------------------------+
|                    User Interaction                  |
+------------------------------------------------------+
           |                         |
           | GUI Popup              | CLI Commands
           v                         v
+-------------------+     +---------------------------+
|  Tauri Frontend   |     |         CLI (vcm)         |
|  (Vanilla JS UI)  |     |  push/pop/settings/etc   |
+-------------------+     +---------------------------+
            \                    /
             \                  /
              \                /
               v              v
        +----------------------------------+
        |      Shared Clipboard Engine     |
        |        (Rust Core Logic)         |
        +----------------------------------+
                     |
                     |
        +-------------------------------+
        | Clipboard Watcher (Rust)      |
        | Config Manager                |
        | Persistence Layer             |
        +-------------------------------+
                     |
                     |
              +--------------+
              | OS Clipboard |
              +--------------+
````

---

# 3. Core Components

## 3.1 Clipboard Engine (Rust)

Main backend engine.

Responsibilities:

* clipboard monitoring
* clipboard history
* persistence
* push/pop logic
* deduplication
* pinning
* image support

Location:

```text
src-tauri/src/clipboard.rs
```

---

## 3.2 Window Manager

Controls popup lifecycle.

Responsibilities:

* show/hide popup
* cursor-based positioning
* focus management
* draggable popup
* animations integration

Location:

```text
src-tauri/src/window.rs
```

---

## 3.3 Config Manager

Handles:

* shortcuts
* theme
* autostart
* max history items

Location:

```text
src-tauri/src/config.rs
```

Config path:

```text
~/.config/vcm/config.json
```

Example:

```json
{
  "shortcut": "Ctrl+Shift+V",
  "theme": "dark",
  "autostart": true,
  "maxItems": 50
}
```

---

## 3.4 CLI Layer (`vcm`)

Developer-friendly command interface.

Location:

```text
src-tauri/src/cli.rs
```

Built using:

* clap

Commands:

```bash
vcm
vcm settings
vcm push "hello"
vcm pop
vcm pop 2
vcm clear
vcm clear 2
vcm list
```

---

# 4. Frontend Architecture

Frontend stack:

* Vanilla JS
* Tailwind CSS

Reason:

* faster startup
* lower memory
* smaller bundle
* simpler architecture

---

## Frontend Structure

```text
src/
├── index.html
├── app.js
├── clipboard.js
├── settings.js
├── styles.css
```

---

## Responsibilities

### app.js

* window events
* keyboard navigation
* rendering
* clipboard selection

### clipboard.js

* clipboard history state
* filtering
* pin logic
* deduplication

### settings.js

* settings modal
* theme handling
* shortcut config

---

# 5. Clipboard System

## Supported Types

### Text

```json
{
  "type": "text",
  "content": "hello"
}
```

### Images

```json
{
  "type": "image",
  "content": "<base64>"
}
```

---

## Deduplication

Behavior:

* repeated copy moves item to top
* avoids duplicate spam

---

## Pinning

Pinned items:

* remain at top
* never auto-removed
* survive cleanup

---

# 6. Search System

Search is:

* in-memory
* frontend-based
* case-insensitive

Planned:

* fuzzy search
* ranking
* semantic matching

---

# 7. Window Lifecycle

VCM behaves like:

* Raycast
* Spotlight
* CopyQ popup

Flow:

```text
Shortcut Pressed
       ↓
Popup Opens Near Cursor
       ↓
User Selects Item
       ↓
Clipboard Updated
       ↓
Popup Hides
```

---

## Window Rules

### Show

* configurable shortcut
* opens near mouse cursor

### Hide

* ESC
* click outside
* selection

### Reopen

* same shortcut

---

# 8. Cursor-Based Popup

Popup positioning:

* near mouse cursor
* avoids screen overflow
* offset from cursor

Implemented in:

```text
window.rs
```

---

# 9. Theme System

Supported themes:

* dark
* light

Future:

* system theme sync

Theme persisted in:

```json
{
  "theme": "dark"
}
```

---

# 10. Persistence

Current:

* JSON file storage

Future:

* SQLite

Current storage path:

```text
~/.local/share/vibes-copy-manager/
```

---

# 11. Performance Goals

## Startup

Target:

```text
< 50ms feel
```

## Search

Target:

```text
instant filtering
```

## Clipboard Polling

Current:

```text
300–500ms
```

Future:

* native clipboard hooks

---

# 12. Cross-Platform Support

Supported:

* Ubuntu
* Debian
* macOS
* Windows

---

## Linux

Formats:

* AppImage
* .deb

Dependencies:

* GTK/WebKit

Supports:

* X11
* partial Wayland support

---

## macOS

Formats:

* .dmg

Special handling:

* accessibility permissions

---

## Windows

Formats:

* .msi

Special handling:

* SmartScreen warnings for unsigned builds

---

# 13. Installation System

## CLI Install

```bash
curl -sSL <install-url> | sh
```

Installs:

* `vcm`
* GUI launcher

---

## GUI Install

Distributed via:

* GitHub Releases

---

# 14. GitHub CI/CD

Workflow:

```text
Tag Push
   ↓
GitHub Actions
   ↓
Build Linux
Build macOS
Build Windows
   ↓
Create Release
   ↓
Upload Artifacts
```

---

## Trigger

```bash
git tag v1.0.0
git push origin v1.0.0
```

---

# 15. Build Outputs

Linux:

```text
vcm-linux.AppImage
vcm-linux.deb
```

macOS:

```text
vcm-macos.dmg
```

Windows:

```text
vcm-windows.msi
```

---

# 16. Security & Release Controls

Release rules:

* protected master branch
* protected tags (`v*`)
* release only from master
* release only by owner

---

# 17. Startup / Background Behavior

Optional autostart:

* Linux:
  ~/.config/autostart/
* macOS:
  LaunchAgents
* Windows:
  Startup folder

Behavior:

* app runs in background
* popup shown via shortcut

---

# 18. Future Roadmap

## Planned Features

### Clipboard

* file clipboard support
* rich text
* markdown preview

### Search

* fuzzy search
* AI ranking

### Performance

* daemon mode
* IPC
* native clipboard hooks

### UX

* keyboard-only workflows
* multi-monitor support
* animations polish

### Integrations

* shell integration
* browser extension
* sync

---

# 19. Development Workflow

## Run GUI

```bash
cargo tauri dev
```

## Build GUI

```bash
cargo tauri build
```

## Build CLI

```bash
cargo build --release --bin vcm --no-default-features
```

---

# 20. Design Principles

VCM prioritizes:

* speed
* minimalism
* keyboard-first interaction
* low memory
* low CPU
* extensibility
* clean architecture

Avoid:

* heavy frameworks
* unnecessary abstraction
* slow rendering
* startup lag

---

# 21. Recommended Contribution Rules

## Backend

* keep Rust modular
* avoid blocking UI thread
* prefer Result<T,E>

## Frontend

* avoid framework bloat
* optimize rendering
* maintain accessibility

## CI/CD

* keep reproducible builds
* tag-based releases only

---

# 22. Mental Model for New Contributors

VCM is NOT:

* just a popup UI

VCM IS:

```text
Clipboard Engine
      +
Popup Interface
      +
CLI Utility
      +
Background Productivity Tool
```

Think:

* fast utility
* not traditional desktop app

---

# 23. Current Tech Stack

| Layer         | Technology     |
| ------------- | -------------- |
| Backend       | Rust           |
| Desktop Shell | Tauri v2       |
| UI            | Vanilla JS     |
| Styling       | Tailwind CSS   |
| CLI           | clap           |
| Persistence   | JSON           |
| CI/CD         | GitHub Actions |

---

# 24. Project Goals

Short-term:

* stable cross-platform release

Mid-term:

* daemon + IPC architecture

Long-term:

* best lightweight clipboard manager

```
```
