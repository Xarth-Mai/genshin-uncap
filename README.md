# Genshin Uncap

[English](README.md) · [简体中文](README.zh-CN.md) · [繁体中文](README.zh-TW.md) · [日本語](README.ja.md)

Yet another Genshin Impact FPS unlocker written in Rust

Small and native, supporting Windows and Linux (Wine / Steam Proton)

## Quick Start

### Windows

Download the latest version from [Releases](https://github.com/Xarth-Mai/genshin-uncap/releases)

Start the game, then run `genshin-uncap.exe`. The program automatically detects `YuanShen.exe` or `GenshinImpact.exe` and unlocks the FPS limit to 120

You can also launch the game directly through this program:

```bat
genshin-uncap.exe --game "C:\Games\Genshin Impact Game\YuanShen.exe" --fps 120
```

### Linux — Wine / Steam Proton

Set Steam's launch target to `genshin-uncap.exe`, keep the game's existing Proton version, and set Launch Options to:

```text
%command% --game "C:\Games\Genshin Impact Game\YuanShen.exe" --fps 120 --hidden
```

Steam's Target and Start In use Linux paths; `--game` uses a Windows path visible inside the Wine / Proton prefix

## F10 Pause / Resume

Press F10 while the game is focused to pause or resume FPS control

Pausing restores the original FPS value from the start of the session; pressing F10 again reapplies the selected FPS limit

## Options

| Option | Description |
| --- | --- |
| `--game <path>` | Game executable to launch; omit to detect or wait for `YuanShen.exe` / `GenshinImpact.exe` |
| `--fps <1..120>` | Set the FPS limit; default: 120 |
| `--hidden` | Hide the controller window and write logs to `%LOCALAPPDATA%\genshin-uncap\`; ignored without `--game` |
| `--probe` | Check whether the current game version can be recognized without modifying game memory |
| `--help`, `-h` | Show help |
| `--version`, `-V` | Show version and build information |
| `-- <arguments...>` | Forward the remaining arguments unchanged when launching the game; ignored without `--game` |

## FAQ

### How do I change the FPS limit?

Use `--fps` to set the desired limit, for example:

```text
--fps 90
```

The FPS limit stays fixed for each run. To change it, restart the program with a new `--fps` value

### Where are logs saved?

With `--hidden`, logs are saved in:

```text
%LOCALAPPDATA%\genshin-uncap\
```

At startup, the program automatically removes old logs and keeps only the 10 newest controller logs

### Why does the actual FPS not reach the selected value?

`--fps` sets an FPS limit; it does not guarantee that the game can reach that frame rate

Actual FPS still depends on GPU / CPU performance, graphics settings, VSync, and other frame-rate limits

### What if FPS unlocking stops working after a game update?

Game updates may change the data used to locate the FPS limit

When reporting an issue, keep the console error messages; if you use `--hidden`, also include the relevant logs from `%LOCALAPPDATA%\genshin-uncap\`

### How do I exit?

Close `genshin-uncap`

If you want to restore the session's original FPS value before exiting, press F10 to pause FPS control, then close the program

The controller also exits automatically when the game exits

### Which environments have been tested?

The main tested setup is `YuanShen.exe` running through Steam Proton, including XWayland and native Wayland

The program is distributed as a Windows x64 executable and runs through Wine / Proton on Linux

## Building from Source

On Windows, install Rust and Visual Studio C++ Build Tools (the Desktop development with C++ workload), then add the Windows x64 MSVC target:

```sh
rustup target add x86_64-pc-windows-msvc
```

Build the Release version:

```sh
cargo build --release --locked
```

The output file is located at:

```text
target/x86_64-pc-windows-msvc/release/genshin-uncap.exe
```

Run logic tests on Linux:

```sh
cargo test --target x86_64-unknown-linux-gnu --locked
```

## Credits

Thanks to the following projects, tools, and ecosystems for references, infrastructure, runtime environments, and development assistance:

* [xiaonian233/genshin-fps-unlock](https://github.com/xiaonian233/genshin-fps-unlock)
* [34736384/genshin-fps-unlock](https://github.com/34736384/genshin-fps-unlock)
* [windows-rs](https://github.com/microsoft/windows-rs)
* [Rust](https://www.rust-lang.org/)
* [Steam](https://store.steampowered.com/)
* [Proton](https://github.com/ValveSoftware/Proton)
* [dwproton](https://dawn.wine/dawn-winery/dwproton)
* [Wine](https://www.winehq.org/)
* [Linux](https://www.linux.org/)
* OpenAI ChatGPT / Codex

## Open Source License

This project is licensed under the [Mozilla Public License 2.0](LICENSE)

Third-party notices are listed in [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)
