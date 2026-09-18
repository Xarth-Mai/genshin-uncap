# Genshin Uncap

[English](README.md) · [简体中文](README.zh-CN.md) · [繁体中文](README.zh-TW.md) · [日本語](README.ja.md)

又一个使用 Rust 编写的《原神》FPS 解锁器

小巧、原生，支持 Windows 与 Linux（Wine / Steam Proton）

## 快速开始

### Windows

从 [Releases](https://github.com/Xarth-Mai/genshin-uncap/releases) 下载最新版本

先启动游戏，再运行 `genshin-uncap.exe`。程序会自动检测 `YuanShen.exe` 或 `GenshinImpact.exe`，并将 FPS 上限解锁至 120

也可以直接通过本程序启动游戏：

```bat
genshin-uncap.exe --game "C:\Games\Genshin Impact Game\YuanShen.exe" --fps 120
```

### Linux — Wine / Steam Proton

将 Steam 的启动目标设置为 `genshin-uncap.exe`，继续使用游戏原有的 Proton 版本，并在 Launch Options 中设置：

```text
%command% --game "C:\Games\Genshin Impact Game\YuanShen.exe" --fps 120 --hidden
```

Steam 的 Target 和 Start In 使用 Linux 路径；`--game` 使用 Wine / Proton prefix 内可见的 Windows 路径

## F10 暂停 / 恢复

游戏处于焦点状态时，按 F10 可以暂停或恢复 FPS 控制

暂停时会恢复本次会话开始时的原始 FPS 值；再次按下 F10 后，会重新应用设置的 FPS 上限

## 选项

| 选项 | 说明 |
| --- | --- |
| `--game <path>` | 指定要启动的游戏可执行文件；省略时自动检测或等待 `YuanShen.exe` / `GenshinImpact.exe` |
| `--fps <1..120>` | 设置 FPS 上限，默认 120 |
| `--hidden` | 隐藏控制器窗口，并将日志写入 `%LOCALAPPDATA%\genshin-uncap\`；未指定 `--game` 时忽略 |
| `--probe` | 检查当前游戏版本是否可识别，但不修改游戏内存 |
| `--help`, `-h` | 显示帮助 |
| `--version`, `-V` | 显示版本和构建信息 |
| `-- <arguments...>` | 启动游戏时，将后续参数原样传递给游戏；未指定 `--game` 时忽略 |

## FAQ

### 如何修改 FPS 上限？

使用 `--fps` 设置所需上限，例如：

```text
--fps 90
```

FPS 上限在一次运行期间保持固定。需要更换数值时，请使用新的 `--fps` 参数重新启动程序

### 日志保存在哪里？

使用 `--hidden` 时，日志保存在：

```text
%LOCALAPPDATA%\genshin-uncap\
```

程序启动时会自动清理旧日志，仅保留最新的 10 个控制器日志

### 为什么实际 FPS 没有达到设置值？

`--fps` 设置的是 FPS 上限，并不保证游戏能够达到对应帧率

实际帧率仍取决于 GPU / CPU 性能、图形设置、VSync，以及其他可能存在的限帧机制

### 游戏更新后无法解锁 FPS 怎么办？

游戏更新可能导致用于定位 FPS 上限的数据发生变化

反馈问题时，请保留控制台中的错误信息；如果使用了 `--hidden`，请同时附上 `%LOCALAPPDATA%\genshin-uncap\` 中的相关日志

### 如何退出？

直接关闭 `genshin-uncap` 即可

如果希望在退出前恢复本次会话原本的 FPS 值，请先按 F10 暂停 FPS 控制，再关闭程序

游戏退出后，控制器也会自动结束

### 测试过哪些环境？

目前主要测试环境为通过 Steam Proton 运行的 `YuanShen.exe`，包括 XWayland 和原生 Wayland

程序发布为 Windows x64 可执行文件，在 Linux 下通过 Wine / Proton 运行

## 从源码构建

添加 Windows x64 GNU target：

```sh
rustup target add x86_64-pc-windows-gnu
```

构建 Release 版本：

```sh
cargo build --release --locked
```

输出文件位于：

```text
target/x86_64-pc-windows-gnu/release/genshin-uncap.exe
```

在 Linux 上运行逻辑测试：

```sh
cargo test --target x86_64-unknown-linux-gnu --locked
```

## 致谢

感谢以下项目、工具与生态提供的参考、基础设施、运行环境与开发协助：

* [xiaonian233/genshin-fps-unlock](https://github.com/xiaonian233/genshin-fps-unlock)
* [34736384/genshin-fps-unlock](https://github.com/34736384/genshin-fps-unlock)
* [windows-rs](https://github.com/microsoft/windows-rs)
* [Rust](https://www.rust-lang.org/)
* [Steam](https://store.steampowered.com/)
* [Proton](https://github.com/ValveSoftware/Proton)
* [Wine](https://www.winehq.org/)
* [Linux](https://www.linux.org/)
* OpenAI ChatGPT / Codex

## 开源许可证

本项目采用 [Mozilla Public License 2.0](LICENSE)

第三方软件声明见 [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)
