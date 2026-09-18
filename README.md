# genshin-uncap

小型 Windows x64 Rust FPS 控制器，启动游戏后从外部周期写入固定目标值，退出工具即停止写入

当前已在本机 dwproton 上完成 120 FPS 连续 30 分钟及多次退出试验，剧情过场和完整 Steam／BetterGI 接入等仍未验收；适用范围与逐项证据见 [验收记录](specs/001-fps-control/tasks.md)

## 使用

```text
genshin-uncap.exe --game "C:\Program Files\miHoYo Launcher\games\Genshin Impact Game\YuanShen.exe" --fps 120
genshin-uncap.exe --probe --game "C:\Program Files\miHoYo Launcher\games\Genshin Impact Game\YuanShen.exe"
```

`--game` 必填，`--fps` 默认 120，允许 1–120；`--` 后的参数原样传给游戏，运行期间不调节数值

`--probe` 只启动、读取和定位，绝不写入游戏内存；结束后游戏继续运行，再次运行工具前需自行退出游戏

通过 Ctrl+C 或关闭控制台退出工具，不恢复任何 FPS 值；最后写入值可能保留到游戏再次更新限帧，工具不会主动终止游戏

当前固定每 500 ms 写入一次，候选就绪前只读等待，目标上限严格为 120

仅操作本次启动的进程，不 attach；已运行游戏或另一控制器存在时拒绝启动，错误或不可信地址停止操作，不自动切换注入实现

## 构建与测试

需要 Rust 的 `x86_64-pc-windows-gnu` 标准库和 `x86_64-w64-mingw32-gcc`，默认构建目标已经设为 Windows GNU

在已有 rustup 与 MinGW 的环境中，这些命令可直接从 Fish 执行

```fish
rustup target add x86_64-pc-windows-gnu
cargo build --release --locked
cargo test --target x86_64-unknown-linux-gnu --locked
```

本机安装了隔离工具链，保留系统 Rust；以下脚本仅使用 `target/toolchain/` 中已安装的工具，不联网安装或修改 shell 配置，同样可从 Fish 调用

```fish
./scripts/cargo-local.sh build --release --locked
./scripts/cargo-local.sh test --target x86_64-unknown-linux-gnu --locked
./scripts/cargo-local.sh build --examples --locked
./scripts/cargo-local.sh test --target x86_64-pc-windows-gnu --no-run --locked
```

产物为 `target/x86_64-pc-windows-gnu/release/genshin-uncap.exe`，发布配置为速度优先的完整 LTO、单 codegen unit、符号剥离和 panic abort，不启用 `target-cpu=native`

`target/` 包括隔离工具链和测试 prefix，删除它会同时删除这些本机工具；跨机器构建使用前述正常 Rust／MinGW 安装方式

Windows 进程测试使用自建假目标，不访问真实游戏；本机隔离 dwproton Wine 测试命令如下，`target/test-prefix` 与真实游戏 prefix 分离

```fish
env DISPLAY=:0 WAYLAND_DISPLAY=wayland-1 WINEPREFIX="$PWD/target/test-prefix" WINEDEBUG=-all CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUNNER=/usr/share/steam/compatibilitytools.d/dwproton/files/bin/wine ./scripts/cargo-local.sh test --locked -- --test-threads=1
```

`controller_break_exit` 通过 API 定向发送 Ctrl+Break 时在当前 dwproton 下得到退出码 1，已确认 dwproton 将 CTRL_BREAK 转为 SIGQUIT，直接终止线程而未调用处理函数，作为明确忽略的回归保留；实际控制器的隔离 Ctrl+C 测试及真实 Ctrl+C／关闭窗口检查已分别通过，Ctrl+Break 的退出码异常单独保留

调查该失败时，在上述命令的 `test` 后使用 `--test windows_flow controller_break_exit -- --ignored --nocapture --test-threads=1`；只向测试控制器的非零进程组发送事件

真实验收与每项测试的证据见 [tasks.md](specs/001-fps-control/tasks.md)

## 接入边界

已完成真实只读、120 FPS 写入、30 分钟及四种退出路径检查；剧情过场与完整 Steam／BetterGI 启动链等尚未验收，因此当前保留旧批处理调用，未安装替换

保留现有 runner、prefix、工作目录和 BetterGI 顺序；验收通过后仅备份并替换旧解锁器调用，CLI 显式指定路径和 FPS

本实现没有远程线程、DLL、shellcode、驱动、隐藏、权限提升、自动补丁下载或遥测

## 设计与许可

[行为规格](specs/001-fps-control/spec.md) · [研究证据](specs/001-fps-control/research.md) · [设计](specs/001-fps-control/plan.md) · [验收记录](specs/001-fps-control/tasks.md)

项目保留现有 MPL-2.0 许可证，参考源码及其 MIT 许可、未引入的第三方部分见研究记录；参考不等于复用它们的执行代码
