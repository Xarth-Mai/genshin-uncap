# 实施任务与验收结果

状态定义：`PASS` 表示列出的实际检查通过，`FAIL` 表示执行后不满足验收，`BLOCKED` 表示有具体环境或技术阻塞，`NOT RUN` 表示尚未执行

这份文档记录实际执行证据，同一验收下已完成和未完成场景分列，不能把单个场景通过解释为整项行为已验收

## 任务表

| 任务 | 关联要求／验收 | 工作与测试 | 状态 | 结果或阻塞 |
| --- | --- | --- | --- | --- |
| T-001 | 全部 | 固定需求、审计基线、最小外部方案和分层验收 | PASS | 用户已批准方案，4 份规格文档已建立，固定源码静态证据见 research.md |
| T-002 | AC-009 | 配置 release 优化并补齐隔离 Windows 交叉工具链 | PASS | `target/toolchain/` 中 Rust 1.98.1 与 GNU 交叉工具链构建成功，系统 Rust 保留 |
| T-003 | REQ-001／AC-001 | CLI 默认 120、显式 120、有效边界 1／120、非法值、重复参数和逐项参数转发 | PASS | 用户限定最高120后复验：Linux7项、Windows库8项通过；Windows进程4通过／1已知忽略，已通过的60秒超时未重复 |
| T-004a | REQ-005、REQ-006／AC-003 | 安全 PE、扫描和 RIP 地址纯逻辑 | PASS | 截断／畸形头、节表位置、末尾、等长、超长、跨块与空洞、负位移、溢出和多个模式命中 |
| T-004b | REQ-005、REQ-006／AC-003 | 完整控制器缺 `.text`、多个有效不同目标拒绝 | PASS | 缺节立即失败且目标仍为 60；自建 PE 添加第二个不同可写目标，记录两候选后拒绝，无覆盖且目标存活 |
| T-005a | REQ-002、REQ-003／AC-002 | Wine 假目标启动、Unicode／空格／空参数／引号／尾随反斜杠、工作目录与重复启动 | PASS | 子进程参数与目录符合预期，重复控制器和已有游戏均被拒绝 |
| T-005b | REQ-005、REQ-008／AC-004 | 假目标定位前以 7 早退 | PASS | 控制器在初始化阶段失败，无覆盖写入 |
| T-005c | REQ-003、REQ-005／AC-002、AC-004 | 模块迟到及错误 prefix | NOT RUN | 不能从变量延迟就绪推导模块迟到通过 |
| T-005d | REQ-005／AC-004 | 初始化超时及值延迟就绪 | PASS | 60 秒超时无写入且目标存活；-1→60 后稳定至少 500 ms 才观察到首个 120 |
| T-006a | REQ-006／AC-003 | Windows 库测试成功读写、保护页面边界与地址溢出 | PASS | 8 项库测试在 Wine 通过，跨 `PAGE_NOACCESS` 的不完整读写返回错误 |
| T-006b | REQ-006、REQ-007／AC-007 | 假目标只读探针、120 覆盖、强制结束工具和游戏先退出 | PASS | 探针无写入，强制停止后假目标自设 30 超过 1.2 秒未被覆盖，游戏先退出后工具结束 |
| T-006c | REQ-007／AC-007 | 独立控制台定向 CTRL_BREAK 诊断 | FAIL | 实际控制器退出码 1，无 STOPPED／FAILED 日志；假目标存活且 30 保持超过 1.2 秒，已确认 Wine SIGQUIT 分发路径，默认忽略的诊断保留环境失败 |
| T-006d | REQ-007／AC-007 | 实际 Rust 控制器假目标 Ctrl+C | PASS | 私有控制台验证仅有测试和控制器两成员后发送事件，退出 0 与 STOPPED；假目标25个重置后样本均为30 |
| T-006e | REQ-007、REQ-008／AC-007、AC-008 | 关闭窗口与真实多次重启 | PASS | 真实Ctrl+C、关控制台与强制终止均不连带结束游戏；第4轮游戏先退出，控制器退出0，每轮重新扫描 |
| T-007 | REQ-009／AC-005 | 当前真实 Proton 只读探针 | PASS | 解锁后重跑并捕获模块、唯一候选、-1→60 及零写入日志，退出码 0，游戏存活 |
| T-008 | REQ-004、REQ-010、REQ-011／AC-006 | 限定真实外部写入初步试验 | PASS | 500 ms／120 FPS 在游戏、菜单和设置中实测约 120 FPS，完整场景仍待 T-009 |
| T-009a | REQ-007、REQ-008／AC-006、AC-007、AC-008 | 当前Proton连续30分钟与4次退出循环 | PASS | 30分钟记录完成，真实Ctrl+C、关控制台、强制结束和游戏先退出4轮通过，未完成场景不计入结论 |
| T-009b | REQ-011／AC-006 | 其余过场与原生最小化 | NOT RUN | 未安排可重放剧情过场；当前niri无原生最小化操作，后台恢复已单列通过 |
| T-010a | AC-009 | 产物格式、imports 和大小检查 | PASS | PE32+ x86-64 控制台 EXE，大小与 imports 已检查，最终哈希见 target/evidence/build.json |
| T-010b | REQ-010／AC-009 | 控制器内存与 CPU 采样 | PASS | 20 秒样本单核 0.05%、RSS 8240 KiB、VmHWM 430732 KiB、主动上下文切换 41.09/s；精确唤醒和额外启动耗时 NOT RUN |
| T-011a | REQ-012／AC-009 | Fish 可执行构建说明与本机工具链入口 | PASS | README 与 `scripts/cargo-local.sh` 已提供，Fish 语法及实际版本调用通过 |
| T-011b | REQ-012／AC-009 | 备份替换旧调用并验证完整Steam链 | PASS | 用户明确授权后安装e949e68b构建，经真实Steam入口确认启动、世界HUD121与退出清理，备份保留；BetterGI仅验证启动 |
| T-012a | REQ-006、REQ-010 | 500ms完整读取校验、仅当前值不同才写入的构建与窄测 | PASS | a8b84948构建，Linux7项、Windows库9项、进程4项通过，1项已知诊断忽略，之前60秒超时过滤；无写权限句柄验证相同值不调用写API，不同值仍尝试写入，其他保护不变 |
| T-012b | REQ-010、REQ-012 | 不同才写的新版完整Steam复验 | PASS | 04:26:19–04:29:57经真实Steam入口启动，标题及世界HUD121，游戏先退出后控制器结束，关闭BetterGI后36个PID清理，wrapper退出0、电源配置恢复 |

## 执行记录

| 日期 | 层级／任务 | 实际命令或动作 | 观测 | 状态 |
| --- | --- | --- | --- | --- |
| 2026-09-19 | 静态／T-001 | 读取附件两份文档，核对固定 commit 源码；按运行时修改重建 416 字节 payload 并静态反汇编 | 发现与最小外部控制器设计依据见 research.md，未执行 payload 或连接真实游戏 | PASS |
| 2026-09-19 | 构建／T-002 | `./scripts/cargo-local.sh build --release --locked` | GNU Windows x64 release 构建成功，格式 PE32+ 控制台 | PASS |
| 2026-09-19 | 宿主／T-003、T-004a | `./scripts/cargo-local.sh test --target x86_64-unknown-linux-gnu --locked` | 7 项通过，包含补齐的有效 FPS 边界 | PASS |
| 2026-09-19 | Windows／T-006a | 构建 Windows 测试二进制后在隔离 Wine prefix 执行 | 8 项通过，包含跨保护页面和溢出失败检查，最终复跑通过 | PASS |
| 2026-09-19 | 假目标／T-005a、T-005b、T-006b | `cargo build --offline --examples` 后，在隔离 dwproton Wine runner 执行 `cargo test --offline --test windows_flow real_windows_process_flow -- --nocapture --test-threads=1` | 独立运行 1 项通过、1 项过滤，用时 2.37 秒；最终复跑 1 项通过，用时 2.39 秒，涵盖定位前早退 | PASS |
| 2026-09-19 | 控制台诊断／T-006c | 同一假目标环境执行 `cargo test --test windows_flow controller_break_exit -- --ignored --nocapture --test-threads=1` | 控制器实际退出 1，期望 0，保留已知失败；日志 target/windows-console-test.log | FAIL |
| 2026-09-19 02:54 +08:00 | 真实只读／T-007 | `target/evidence/probe-command.json` 记录 runner、prefix、完整 argv 和环境；`target/evidence/proton-probe.log` 保存 runner 输出 | dwproton 11.0.12、prefix 2330477040、SteamLinuxRuntime_4，观察到游戏宿主 PID 691001，控制器退出后游戏仍存活；缺控制器候选日志 | BLOCKED |
| 2026-09-19 | 假目标／T-004b | `cargo test --test windows_flow ambiguous_targets_are_rejected_without_writing --offline --locked -- --nocapture --test-threads=1` | 两个有效不同目标立即拒绝，目标保持 60，无 WRITING，1 项通过，用时 0.25 秒 | PASS |
| 2026-09-19 | 假目标／T-006d | `cargo test --test windows_flow controller_ctrl_c_exit --offline --locked -- --nocapture --test-threads=1` | 实际 Rust 控制器 Ctrl+C 正常退出，25 个后续样本保持 30，1 项通过，用时 2.11 秒 | PASS |
| 2026-09-19 | 假目标／T-005d | 隔离 Wine 中分别执行 `initialization_timeout_stops_without_writing`、`delayed_ready_stays_read_only_until_stable` | 60 秒超时；-1 延迟就绪后稳定至少 500 ms 才写入 | PASS |
| 2026-09-19 | 真实只读／T-007 | 同 runtime／prefix，`probe.cmd` 捕获控制器输出 | 模块、候选、-1→60 与 PROBE COMPLETE 已留存，退出 0 | PASS |
| 2026-09-19 03:15 +08:00 起 | 真实写入／T-008 | `trial.cmd`、`trial-command.json`、MangoHUD 每秒 CSV | 世界、菜单、30FPS设置覆盖、垂直同步开／关、一次传送与30分钟记录通过；退出结果分别记录 | PASS |
| 2026-09-19 04:11:35–04:21:46 +08:00 | 真实Steam完整链／T-011b | `steam://rungameid/10009322670912438272`，a6cf71d对应e949e68b构建，保留原game-performance、runtime4、dwproton、cmd和BetterGI顺序 | 世界HUD121／8.2ms，Ctrl+C仅结束控制器，游戏与BetterGI随后分别关闭，37个跟踪PID移除，最外层wrapper退出0 | PASS |
| 2026-09-19 04:26:19–04:29:57 +08:00 | 新版真实Steam链／T-012b | 同一 `steam://rungameid/10009322670912438272` 入口，a8b84948构建，每500ms检查、仅不同才写 | 标题及世界HUD121／8.2ms；正常关闭游戏后控制器结束，关闭BetterGI后36个跟踪PID清理，wrapper退出0，恢复power-saver和disabled idle inhibit；本轮未捕获Win32子进程退出码 | PASS |

新版窄测实际命令为 `env DISPLAY=:0 WAYLAND_DISPLAY=wayland-1 WINEPREFIX="$PWD/target/test-prefix" WINEDEBUG=-all CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUNNER=/usr/share/steam/compatibilitytools.d/dwproton/files/bin/wine ./scripts/cargo-local.sh test --offline --locked -- --test-threads=1 --skip initialization_timeout_stops_without_writing`，Windows库9项与进程4项通过，进程用时7.71秒，日志 `target/evidence/windows-check-before-write-tests.log`；Linux7项和rustfmt检查也通过

120上限及首轮完整Steam链构建为391680字节，SHA-256 `e949e68be5bea2dac65b1dcc5f538c453f004c11ac44462f27f1ae0b7dcd63c6`，对应实现提交 `a6cf71d`，实际构建命令 `./scripts/cargo-local.sh build --release --offline --locked`；格式检查 `rustfmt --check --edition 2024 src/*.rs examples/*.rs tests/*.rs` 通过

本次不同才写的新版同为391680字节，SHA-256 `a8b8494876aeb695a5b0bb26db5e844336cd8886af3694fffc283244fe579ea3`，已安装至同一 `C:\genshin-uncap.exe`；窄测结果见T-012a，Windows库新增无写权限句柄与条件写入检查，覆盖不同目标值及非法或不可读输入拒绝；新版真实Steam结果见T-012b，未另跑30分钟

首轮30分钟和真实Ctrl+C使用前一份构建 `4da7db30f658d7f3dc7377fe8bf9042dc791d7baa158100bfc06f40c50249859`，实际目标始终120；随后仅将CLI及候选值上限由1000收紧为120，e949e68b构建重新通过Windows库8项与进程4项窄测，复用此前已通过的60秒超时证据，并用于后续3次真实生命周期测试，不将其写成该二进制另跑了30分钟；新版的窄测和Steam复验单独记录，不将旧30分钟记录算作新版耐久验收

| 真实循环 | 构建／目标 | 退出动作 | 观察结果 | 证据 |
| --- | --- | --- | --- | --- |
| 1 | 前一构建／120 | 实际控制台Ctrl+C | 连续记录1800.145秒；控制器退出0、游戏存活，随后游戏设30实测30、恢复60实测60 | `trial-controller.log`、`trial-exit.txt`、`trial-30min-summary.json`、`scenarios.json` |
| 2 | e949e68b构建／120 | 正常关闭控制台 | 控制器消失、游戏存活，关闭窗口没有保留退出码或STOPPED尾句 | `cycle2-controller.log`、`cycle2-command.json`、`scenarios.json` |
| 3 | e949e68b构建／120 | 核对控制器身份后SIGKILL | 控制器退出1、游戏存活，未向游戏发送终止信号 | `cycle3-stop.json`、`cycle3-exit.txt` |
| 4 | e949e68b构建／120 | 正常关闭游戏 | 控制器打印STOPPED并退出0，双方均已结束 | `cycle4-controller.log`、`cycle4-exit.txt` |

a6cf71d提交前完整复跑命令为 `env DISPLAY=:0 WAYLAND_DISPLAY=wayland-1 WINEPREFIX="$PWD/target/test-prefix" WINEDEBUG=-all CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUNNER=/usr/share/steam/compatibilitytools.d/dwproton/files/bin/wine ./scripts/cargo-local.sh test --offline --locked -- --test-threads=1`，结果 Windows 库 8 项通过，进程测试 5 项通过、0 失败、1 项明确忽略，用时 68.56 秒，完整日志 `target/evidence/windows-final-tests.log`

`target/evidence/` 为忽略的本机证据目录，首轮探针日志丢失已通过带重定向的探针重跑补齐，`probe-ready.log` 和 `probe-ready-exit.txt` 留存候选及退出码

前述30分钟与4轮生命周期试验手动经同 `SteamLinuxRuntime_4 → dwproton → cmd.exe` 调用，保留真实 prefix 与 XWayland／MangoHUD，当时未运行完整Steam链；本轮另通过真实Steam入口完成首轮完整链，不将其写成再次进行30分钟试验

用户明确授权本轮接入后，已安装 `C:\genshin-uncap.exe`，原批处理备份为 `C:\genshin_bgi_unlock.cmd.before-genshin-uncap-20260919`，仅末行替换为 `"C:\genshin-uncap.exe" --game "C:\Program Files\miHoYo Launcher\games\Genshin Impact Game\YuanShen.exe" --fps 120`，其余内容保持不变，当前接入保留；复制备份回 `C:\genshin_bgi_unlock.cmd` 即可恢复旧调用

首轮完整链为 `Steam → game-performance → SteamLinuxRuntime_4 → dwproton → cmd.exe → BetterGI与控制器 → YuanShen.exe`；世界HUD121FPS／8.2ms，固定写入120；Ctrl+C后控制器消失而游戏与BetterGI存活，关闭游戏后BetterGI仍存活、电源配置仍为performance，关闭BetterGI后37个已跟踪PID全部从Steam列表移除，最外层wrapper退出0，Steam结束运行状态、电源配置恢复power-saver、idle inhibit已disabled

Steam对本轮Wine子进程报告退出码-1，未捕获实际Win32退出码或STOPPED尾句，不声称本轮控制器退出0；BetterGI `0.65.0` 仅验证启动，未验证截图或自动化，过场等原未测场景仍为NOT RUN

本轮证据为 `target/evidence/steam-chain-{install,run,processes,observations}.json` 和 `steam-chain-gameprocess.log`，与其余本机证据一样被Git忽略；新版证据为 `conditional-write-build.json`、`steam-conditional-{run,processes,observations}.json` 和 `steam-conditional-gameprocess.log`

桌面锁屏阻塞已在用户授权后通过正常 DMS IPC 解除，试验期间临时启用idle inhibit，结束已恢复为disabled；所有测试游戏已正常关闭，游戏内FPS恢复60、垂直同步恢复关闭，原神音量按用户要求保持15%

假目标命令使用本机隔离工具链及 `target/test-prefix`，真实游戏 prefix 未用于假目标测试；本轮假目标报告位于该 prefix 的 `drive_c/users/steamuser/AppData/Local/Temp/genshin-uncap-244-*-测试 spaces/`，其中 `controller.log` 为控制器日志、`report.txt` 为假目标参数与样本记录，早退目标不生成样本报告

## 真实场景记录

| 场景 | 需要记录的证据 | 当前状态 |
| --- | --- | --- |
| 初始化、登录与进入游戏 | 真实探针记录唯一候选和 -1→60，写入后登录并加载进入世界，HUD 约 121 | PASS |
| 菜单与游戏设置变更 | UI 改 30 后 HUD 仍约 121，随后恢复 UI 60 | PASS |
| 垂直同步开／关，120 FPS 目标 | 180 Hz 规格显示器，当前模式约120 Hz，开72个样本中位数 119.885 FPS／8.34134 ms，关 47 个样本 121.464 FPS／8.23291 ms，已恢复关闭 | PASS |
| 传送和加载 | 对当前深境入口执行传送，加载后回到世界且 HUD121；43 个样本中位数121.56，控制器无错误 | PASS |
| 过场 | 不用无关剧情进展替代测试，尚无对应证据 | NOT RUN |
| 后台与恢复 | 控制台抢焦点后恢复游戏，地图 HUD122、世界 HUD121，控制器无错误；这不是原生最小化验收 | PASS |
| 真实 Ctrl+C | 控制器退出码0且STOPPED，游戏存活；游戏手动设30后HUD30，恢复60后HUD60 | PASS |
| 关闭控制台 | e949e68b构建在标题画面HUD121后关闭控制台，控制器消失且游戏继续存活；窗口关闭未留存退出码或STOPPED尾句 | PASS |
| 强制结束 | 精确核对第三轮控制器身份后SIGKILL，控制器消失且返回1，真实游戏继续运行；假目标另证明无后续覆盖 | PASS |
| 游戏先退出及重新启动 | 第4轮先正常关闭游戏，控制器打印STOPPED并退出0；每轮新进程重新扫描 | PASS |
| 连续30分钟覆盖 | 1800个每秒样本，记录1800.145秒；加载后1591样本中位121.031FPS，首轮结束时控制器仍运行 | PASS |
| 至少3次循环 | 已完成Ctrl+C、关闭控制台、强制结束和游戏先退出4轮；e949e68b构建用于第2–4轮，每轮重新取得句柄并扫描 | PASS |
| 首轮完整Steam链 | a6cf71d对应e949e68b构建，经真实Steam入口进入世界HUD121，Ctrl+C仅结束控制器，随后关闭游戏和BetterGI完成wrapper清理；仅验证BetterGI启动 | PASS |
| 本次不同才写的新版Steam复验 | a8b84948构建每500ms检查、仅不同才写；同一真实入口启动，标题和世界HUD121，游戏先退出后控制器结束，随后关闭BetterGI完成清理 | PASS |

## 明确不适用的旧验收项

热键长按、连续 toggle、恢复当前游戏设置、worker-ready、IPC 断连与 DLL 释放属于附件中的早期候选功能，获批 V1 不包含这些功能，故不设置对应测试或声称通过

普通 Wine 和原生 Windows 可提供补充证据，但不是当前真实 Proton 目标的替代；外部写入失败、环境不可用或证据不足时在这里写清具体阻塞与未完成项
