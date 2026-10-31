<div align="center">

# 🌸 Petal

**A calm, fast, native-feeling top bar and dock for Windows 10/11.**
Live system info, media controls, a smart dock, and edge volume/brightness pills, all in one small app.

[![Release](https://img.shields.io/github/v/release/R0YC0LD/petal?style=flat-square)](https://github.com/R0YC0LD/petal/releases/latest)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](LICENSE)
![Platform](https://img.shields.io/badge/platform-Windows%2010%2F11-0078D4?style=flat-square)
![Built with](https://img.shields.io/badge/built%20with-Tauri%202%20%2B%20Rust%20%2B%20React-orange?style=flat-square)

**English** · [Türkçe](README.tr.md)

![Top bar](docs/img/notch.png)

![Dock](docs/img/dock.png)

</div>

## Why Petal

Petal puts what you actually look at on screen into two small surfaces: a top bar and a dock. It reads real data from Windows (no fake widgets), animates at your display's refresh rate, and stays out of the way when idle.

- **Free and open source**, MIT licensed. No account, no telemetry.
- **Light on resources**: roughly 0.3 % CPU when idle, Rust backend, one lightweight WebView per surface.
- **Smooth**: spring animations, no open delay, a close delay you control.

## Features

### Top bar
- Clock and date, live **CPU / GPU / RAM** percentages
- Now-playing media with album and playlist info, play/pause/skip/seek (works with Spotify, browsers, any app that reports media to Windows)
- Weather for your city (auto-detected or chosen manually)
- Collapses to a compact clock when idle, expands on hover
- **Quick search** (`Ctrl+Shift+Space`, falls back to `Alt+Space`): apps, open windows and simple math, fully keyboard driven

### Dock
- Pin any app: drag an `.exe` or shortcut onto the dock, or pick from the Start menu list
- **Drag to reorder** pinned apps
- Shows **running apps** and their windows, with live **window previews** on hover
- Right-click menu: new window, pin/unpin, close all windows
- Background apps panel (tray overflow) and a **Show desktop** button
- Magnification, bounce on launch, auto-hide with an adjustable **close delay**

### Edge controls
- Move the mouse to the **left edge** for a vertical volume pill and the **right edge** for brightness
- Both can be open at the same time

### Everything else
- Hides automatically in **fullscreen** games, videos and presentations
- Optionally hides the Windows taskbar (always restored on exit; recovery command below)
- Dark, light or system theme, accent color, scale, surface opacity
- Settings window; settings stored as plain JSON with an automatic backup

## Install

1. Download `Petal_x.y.z_x64-setup.exe` from the [latest release](https://github.com/R0YC0LD/petal/releases/latest).
2. Run it. No admin rights needed; it installs to `%LOCALAPPDATA%\Petal`.
3. Petal starts right away. Right-click the top bar, or the tray icon, to open **Settings**.

> The installer is not code-signed yet, so Windows SmartScreen may warn you. Choose **More info → Run anyway**.

Requirements: Windows 10 or 11 (x64) with the WebView2 runtime (already included in Windows 11).

**Uninstall:** Windows Settings → Apps → Petal.

## Usage

| Action | How |
|---|---|
| Expand the top bar | Hover it |
| Open Settings | Right-click the top bar, or the tray icon |
| Quick search | `Ctrl+Shift+Space` (or `Alt+Space`) |
| Add an app to the dock | Drag an `.exe` / shortcut onto it, or Settings → Dock |
| Reorder dock apps | Press and drag an icon sideways |
| Window previews | Hover a running app's icon |
| Dock menu | Right-click an icon |
| Volume / brightness | Move the mouse to the left / right screen edge |

## Settings

| Area | Options |
|---|---|
| General | Theme, accent color, UI scale, surface opacity, start with Windows, hide Windows taskbar |
| Top bar | Pin open, auto-collapse time, panel layout and order, metrics order, city, hide in fullscreen, search hotkey |
| Dock | Icon size, magnification, labels, show running apps, auto-hide, **dock close delay**, pinned apps |
| Edge pills | Enable / disable, close delay |

Settings file: `%APPDATA%\com.by599.petal\settings.json`.

## Troubleshooting

- **Taskbar missing after a crash:** press `Win+R` and run `%LOCALAPPDATA%\Petal\petal.exe --restore-taskbar`.
- **Search hotkey does nothing:** another app owns it; Petal automatically tries the next alternative.
- **No weather:** pick a city in Settings → Top bar.

## Build from source

Prerequisites: Windows 10/11, [Node.js](https://nodejs.org) 20+, [Rust](https://rustup.rs) (stable, MSVC toolchain), WebView2.

```bash
git clone https://github.com/R0YC0LD/petal.git
cd petal
npm install
npm run tauri dev      # run in development
npm run tauri build    # NSIS installer under src-tauri/target/release/bundle/nsis
```

Stack: **Tauri 2**, **Rust** (`windows` crate: Core Audio, GSMTC media, WinEvent hooks), **React 19**, **TypeScript**, **Motion**, Vite.
More notes live in [`docs/`](docs).

## Known limitations

- Single monitor only for now (multi-monitor is planned)
- Tray apps' own context menus cannot be shown in the background-apps panel
- Installer is unsigned

See [`docs/KNOWN_ISSUES.md`](docs/KNOWN_ISSUES.md) and the [changelog](docs/CHANGELOG.md).

## License

[MIT](LICENSE) © Onur Teryakioğlu ([R0YC0LD](https://github.com/R0YC0LD))
