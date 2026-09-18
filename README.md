# Genshin Uncap

[English](README.md) · [简体中文](README.zh-CN.md) · [繁体中文](README.zh-TW.md) · [日本語](README.ja.md)

> Yet another Genshin Impact FPS unlocker written in Rust

Launch Genshin Impact with an FPS limit from **1 to 120**, then press **F10** to pause or resume control.

Small, native, and built for players who want a higher frame-rate ceiling without changing the game files.

[![Latest Release](https://img.shields.io/github/v/release/Xarth-Mai/genshin-uncap?display_name=tag&sort=semver)](https://github.com/Xarth-Mai/genshin-uncap/releases)
[![License: MPL-2.0](https://img.shields.io/badge/license-MPL--2.0-blue.svg)](LICENSE)

## Quick Start

### Windows

Download the latest build from [Releases](https://github.com/Xarth-Mai/genshin-uncap/releases). Double-clicking the executable shows the console and detects or waits for `YuanShen.exe` or `GenshinImpact.exe`, using the default 120 FPS limit:

```bat
genshin-uncap.exe --game "C:\Games\Genshin Impact Game\YuanShen.exe" --fps 120
```

To launch the game from the controller, use the game executable such as `YuanShen.exe`, not the launcher. The default FPS limit is 120.

### Steam / Proton

Use the game's existing Proton prefix. Set Steam's target to `genshin-uncap.exe`, keep the existing Proton version, and add this to **Launch Options**:

```text
%command% --game "C:\Games\Genshin Impact Game\YuanShen.exe" --fps 120
```

The Steam target and start-in directory use Linux paths. The `--game` path uses the Windows path inside the prefix.

## Advanced Settings

| Option | Description |
| --- | --- |
| `--game <path>` | Optional game executable to launch; omit to detect or wait for `YuanShen.exe` or `GenshinImpact.exe` |
| `--fps <1..120>` | FPS limit; default: `120` |
| `--hidden` | Hide the controller window and write logs to `%LOCALAPPDATA%\genshin-uncap`; ignored when `--game` is omitted |
| `--probe` | Check the game without changing its memory |
| `--help`, `-h` | Show help |
| `--version`, `-V` | Show version and build information |
| `-- <arguments...>` | Forward remaining arguments when launching a game; accepted and ignored without `--game` |

When hidden mode is used, startup cleanup keeps at most the 10 newest controller logs.

Pause and resume FPS control with **F10** while the game is focused. On pause, the controller restores the session's original FPS value. Press F10 again to resume the selected limit.

The FPS limit stays fixed for the session. Restart the controller with a new `--fps` value to change it.

## FAQ

### Why is the actual FPS lower than the limit?

The limit is not a performance guarantee. GPU/CPU load, graphics settings, VSync, and other limiters affect the actual FPS.

### The controller says the game is already running

When `--game` is specified, close Genshin Impact and any existing `genshin-uncap.exe`, then try again. Without `--game`, the controller attaches to one existing supported game process and refuses to choose when both supported clients are running.

### The game updated and control stopped working

Keep the error output or log from `%LOCALAPPDATA%\genshin-uncap\` when reporting the issue.

### How do I stop the controller?

Press F10 first if you want to restore the original FPS value, then close the controller. The controller also exits when the game exits.

### What is tested?

The main tested setup is `YuanShen.exe` through Steam/Proton, including XWayland and native Wayland launch paths. The executable is built for Windows x64.

## Building from Source

```sh
rustup target add x86_64-pc-windows-gnu
cargo build --release --locked
```

Output: `target/x86_64-pc-windows-gnu/release/genshin-uncap.exe`

Run Linux logic tests with:

```sh
cargo test --target x86_64-unknown-linux-gnu --locked
```

## Credits

Thanks to [xiaonian233/genshin-fps-unlock](https://github.com/xiaonian233/genshin-fps-unlock), [34736384/genshin-fps-unlock](https://github.com/34736384/genshin-fps-unlock), and [windows-rs](https://github.com/microsoft/windows-rs).

## Open Source License

[Mozilla Public License 2.0](LICENSE). Third-party notices are listed in [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
