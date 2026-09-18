# genshin-uncap

原神帧率控制工具，默认目标为 120 FPS，支持在游戏中按 F10 暂停或恢复控制，也可隐藏工具的控制台窗口

工具会启动游戏并从外部调整限帧值，目标范围为 1–120 FPS，实际帧率取决于硬件性能、游戏场景及其他限帧设置

## 使用条件

使用 Windows x64 可执行文件 `genshin-uncap.exe`；Linux 下通过游戏使用的 Proton 运行，控制器和游戏必须处于同一个 prefix（兼容环境）

已有使用记录限于 Steam／dwproton 下的 `YuanShen.exe`，包含 XWayland 和原生 Wayland 下的 F10 与隐藏启动；原生 Windows、其他 Proton 版本和其他游戏版本尚未验证

## 快速开始

先退出已运行的游戏和控制器，然后在 Windows 命令提示符中执行以下命令，将路径替换为实际游戏位置

```bat
genshin-uncap.exe --game "C:\Games\Genshin Impact Game\YuanShen.exe" --fps 120
```

选择的是游戏本体 `YuanShen.exe`，而非启动器；工具只控制本次启动的游戏，不能接管已运行的游戏

隐藏控制台启动时添加 `--hidden`

```bat
genshin-uncap.exe --hidden --game "C:\Games\Genshin Impact Game\YuanShen.exe" --fps 120
```

### Steam／Proton

沿用原游戏的非 Steam 条目和 prefix，将目标改为本程序，并在「兼容性」中保留游戏原先使用的 Proton 版本，示例如下

| 字段 | 示例 |
| --- | --- |
| 目标 | `"/path/to/compatdata/1234567890/pfx/drive_c/genshin-uncap.exe"` |
| 起始位置 | `"/path/to/compatdata/1234567890/pfx/drive_c"` |

「启动选项」填写以下整行

```text
%command% --game "C:\Games\Genshin Impact Game\YuanShen.exe" --fps 120
```

隐藏控制器窗口时使用

```text
%command% --hidden --game "C:\Games\Genshin Impact Game\YuanShen.exe" --fps 120
```

将示例路径替换为实际位置；目标和起始位置使用 Linux 绝对路径，`--game` 使用原游戏 prefix 内的 Windows 路径

程序参数放在 `%command%` 后面；`%command%` 保留原样，由 Steam 展开为所选 Proton 和目标程序的启动命令，prefix 目录结构可参考 [Proton FAQ](https://github.com/ValveSoftware/Proton/wiki/Proton-FAQ)

此入口由控制器直接启动游戏，不会执行原有批处理或自动启动 BetterGI；需要保留 BetterGI 联动时继续使用原有批处理入口，并避免其他启动器同时启动游戏

`--hidden` 只隐藏控制器自身的控制台，通过 `cmd.exe` 或批处理启动时，外层命令窗口仍可能显示；需要完全隐藏入口时，使用该 prefix 中的 WScript 隐藏启动批处理

## 操作与参数

游戏处于前台时，单独按 F10 暂停控制，再按一次恢复目标帧率；长按只切换一次，切回游戏后先松开 F10 再按，初始化期间按键无效

暂停会恢复本次启动时、工具首次调整前保存的限帧值，随后停止干预；例如保存值为 60，后来在游戏设置中改为 30，F10 暂停仍恢复到 60

关闭控制台、按 Ctrl+C 或结束控制器进程会停止控制，既不恢复限帧值，也不主动关闭游戏；希望恢复保存值时，先按 F10 暂停再退出工具，游戏退出后工具会自动退出

| 参数 | 作用 |
| --- | --- |
| `--game <路径>` | 必填，指定游戏 EXE，含空格的路径需加双引号 |
| `--fps <整数>` | 目标帧率，默认 120，允许 1–120，本次运行期间固定 |
| `--hidden` | 不创建控制器控制台，将运行信息写入日志 |
| `--probe` | 只启动游戏并检查定位结果，不修改游戏内存，检查结束后游戏继续运行 |
| `--help`、`-h` | 显示帮助，无需游戏路径且不启动游戏 |
| `--version`、`-V` | 显示程序版本、rustc 版本、编译目标和构建 profile，无需游戏路径且不启动游戏 |
| `-- <游戏参数…>` | 将后续参数原样传给游戏 |

当前没有配置文件，也不支持运行期间增减目标帧率；需要更换目标时，退出游戏和工具后使用新参数重新启动

## 常见问题

**没有达到目标帧率**

目标值是限帧设置，不保证硬件能达到该帧率；检查游戏垂直同步、外部限帧及当前场景负载，以实际帧率显示为准

**提示游戏或控制器已运行**

先退出已有游戏或控制器，再重新启动；`--probe` 结束后游戏仍在运行，也需要先退出游戏

**游戏更新后无法定位或控制失败**

定位不明确、地址校验失败或读写被拒绝时，工具会停止控制并报告错误，已启动的游戏继续运行；保留错误信息以便排查，当前实现不会自动下载补丁或切换控制方式

**隐藏模式下如何查看状态或退出**

日志位于 `%LOCALAPPDATA%\genshin-uncap\`，每次运行生成独立文件；失败时显示错误提示，正常运行时可在任务管理器结束 `genshin-uncap.exe`，或退出游戏让工具自动结束

## 从源码构建

Linux 交叉编译需要 Rust／Cargo、`x86_64-pc-windows-gnu` 标准库和 `x86_64-w64-mingw32-gcc`，使用 rustup 管理工具链时可添加目标后构建

```sh
rustup target add x86_64-pc-windows-gnu
cargo build --release --locked
```

默认构建目标已设为 Windows GNU，产物为 `target/x86_64-pc-windows-gnu/release/genshin-uncap.exe`；运行预编译 EXE 不需要安装 Rust 或 MinGW

Linux x64 下的纯逻辑测试可运行 `cargo test --target x86_64-unknown-linux-gnu --locked`，具体行为与验收标准见[行为规格](specs/fps-control.md)

## 致谢

感谢 [xiaonian233/genshin-fps-unlock](https://github.com/xiaonian233/genshin-fps-unlock) 和 [34736384/genshin-fps-unlock](https://github.com/34736384/genshin-fps-unlock) 提供的定位与设计参考，以及 [windows-rs](https://github.com/microsoft/windows-rs) 提供的 Windows API 绑定

## 开源协议

本项目采用 [Mozilla Public License 2.0（MPL-2.0）](LICENSE)，第三方依赖与参考项目的来源、许可说明见 [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)
