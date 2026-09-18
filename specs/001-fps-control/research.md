# 源码证据与实验记录

## 基线与证据边界

日期：2026-09-19

用户后续明确实际写入不得超过120FPS，因此CLI与候选运行值上限收紧到120；所有已执行真实试验目标均为120，原计划的144对照未执行，后续只用120

用户在首轮完整Steam链通过后要求减少相同值的重复写入：检查周期保持500ms，每次照常读取并校验页面、定位指令和FPS值，仅当前值与固定目标不同时调用写入；不放宽原防护，不预先声称性能收益

新版构建为391680字节，SHA-256 `a8b8494876aeb695a5b0bb26db5e844336cd8886af3694fffc283244fe579ea3`，已安装至同一 `C:\genshin-uncap.exe`；Linux7项、Windows库9项和进程4项窄测通过，1项已知Ctrl+Break诊断忽略，之前通过的60秒超时本轮过滤，第二轮真实Steam复验在 `04:26:19–04:29:57 +08:00` 完成，标题和世界HUD121／8.2ms，实际目标120；游戏先退出后控制器自动结束，随后关闭BetterGI，36个跟踪PID全部移除，最外层wrapper退出0，电源配置恢复power-saver、idle inhibit恢复disabled

需求来源为用户会话与附件 `genshin-uncap-spec-pack.zip` 内的 `genshin-uncap-source-audit.md`、`genshin-uncap-codex-spec-prompt.md`，附件启动 Prompt 是较早的审计和访谈指引，最终获批行为以 [spec.md](spec.md) 为准

| 参考项目 | 固定 commit | 根许可证 |
| --- | --- | --- |
| xiaonian233/genshin-fps-unlock | `2f5f5e60882eb6c5e64159efe1eb36928e4f29ea` | MIT |
| 34736384/genshin-fps-unlock | `2b85d61dd06f6e11ad86fdd6bd90339f9abc58eb` | MIT |

本项目现有 `LICENSE` 为 MPL-2.0，保持不变；不移植参考项目 payload 或注入架构，若实际借入代码须保留对应版权和许可并检查文件自带的第三方来源

本轮按 `inject_patch()` 顺序重建 xiaonian 的 416 字节 payload，并以合成 FPS 地址用 Python 与 `objdump` 做静态反汇编核对，没有执行 payload；这不证明用户使用的 release 二进制与固定源码一致，也不证明当前游戏能接受该定位和写入方式

## 静态证据表

下列源码链接固定到上述 commit，事实与设计推导分开记录

| 证据 | 源码与位置 | 源码事实 | 对 V1 的影响或推导 |
| --- | --- | --- | --- |
| A-01 | [xiaonian main.cpp 267–326](https://github.com/xiaonian233/genshin-fps-unlock/blob/2f5f5e60882eb6c5e64159efe1eb36928e4f29ea/unlockfps/main.cpp#L267-L326) | `inject_patch()` 修改原始字节，重建后 `0x110` 同步函数读取 payload `+0x194` 的 32 位值并写 FPS 地址，约 500 ms 循环 | 当前路径是周期覆盖，旧 setter/getter 标签不能证明安装了完整 hook，不移植旧执行代码 |
| A-02 | [xiaonian 284](https://github.com/xiaonian233/genshin-fps-unlock/blob/2f5f5e60882eb6c5e64159efe1eb36928e4f29ea/unlockfps/main.cpp#L284)、[470–662](https://github.com/xiaonian233/genshin-fps-unlock/blob/2f5f5e60882eb6c5e64159efe1eb36928e4f29ea/unlockfps/main.cpp#L470-L662) | worker 读取全局 `FpsValue`，热键修改局部 `TargetFPS`，主循环也写远程缓冲区 | 存在两条状态来源竞争路径，不能据此断言 release 实际故障，V1 固定一个不可变 FPS 来源 |
| A-03 | [xiaonian 290–297](https://github.com/xiaonian233/genshin-fps-unlock/blob/2f5f5e60882eb6c5e64159efe1eb36928e4f29ea/unlockfps/main.cpp#L290-L297) | 运行时补丁改写控制器读取失败路径，使后续跳过读取并继续同步最后缓冲值 | 旧工具退出不保证停止覆盖，V1 不向游戏留存执行代码 |
| A-04 | [xiaonian 134–173](https://github.com/xiaonian233/genshin-fps-unlock/blob/2f5f5e60882eb6c5e64159efe1eb36928e4f29ea/unlockfps/main.cpp#L134-L173) | 扫描条件使用严格小于和无前置检查的长度减法 | 末尾、等长、超长模式成为扫描回归，不把首匹配当最终地址 |
| A-05 | [xiaonian 516–589](https://github.com/xiaonian233/genshin-fps-unlock/blob/2f5f5e60882eb6c5e64159efe1eb36928e4f29ea/unlockfps/main.cpp#L516-L589) | 固定头部复制和节表偏移，未找到 `.text` 时仍可能使用未初始化范围 | 按实际 PE 字段与边界解析，缺节、畸形尺寸明确失败 |
| A-06 | [xiaonian 621–629](https://github.com/xiaonian233/genshin-fps-unlock/blob/2f5f5e60882eb6c5e64159efe1eb36928e4f29ea/unlockfps/main.cpp#L621-L629) | 注入失败后流程仍可能打印 `Done` 并监控 | V1 的阶段成功与失败必须准确；不需要 worker 握手是因为无 worker，而非把线程句柄当运行证明 |
| A-07 | [347 dllmain.cpp 62–121](https://github.com/34736384/genshin-fps-unlock/blob/2b85d61dd06f6e11ad86fdd6bd90339f9abc58eb/UnlockerStub/dllmain.cpp#L62-L121)、[IpcService.cs 38–103](https://github.com/34736384/genshin-fps-unlock/blob/2b85d61dd06f6e11ad86fdd6bd90339f9abc58eb/unlockfps_nc/Service/IpcService.cs#L38-L103) | stub 有页面检查，控制器等待 Ready/Error，但 SetupData 取首个过滤候选，跳转链没有显式上限或循环检测 | 借鉴分阶段验证，V1 不复制其 GUI、IPC、DLL 或跳转跟踪架构 |
| A-08 | [347 Utils.cpp 66–139](https://github.com/34736384/genshin-fps-unlock/blob/2b85d61dd06f6e11ad86fdd6bd90339f9abc58eb/UnlockerStub/Utils.cpp#L66-L139) | SIMD 扫描首次 16 字节装载在短模式或尾部仍可能超出输入范围 | V1 用安全切片，不复制 SIMD 扫描器 |
| A-09 | [347 ProcessUtils.cs 85–109](https://github.com/34736384/genshin-fps-unlock/blob/2b85d61dd06f6e11ad86fdd6bd90339f9abc58eb/unlockfps_nc/Utility/ProcessUtils.cs#L85-L109)、[dllmain.cpp 340–355](https://github.com/34736384/genshin-fps-unlock/blob/2b85d61dd06f6e11ad86fdd6bd90339f9abc58eb/UnlockerStub/dllmain.cpp#L340-L355) | Wine `LoadLibraryEx` workaround 在 C# 辅助扫描路径，当前 FPS 主路径由 native stub 扫描并约每 62 ms 写入 | 不能把辅助路径注释当作当前全部扫描必经步骤，复制到本地也不消除源页面读取权限要求 |

## 当前环境与未验证项

本机配置为 `YuanShen.exe`、120 FPS、游戏配置版本标记 `7.0.0`，当前 runner 为 dwproton `11.0.12`，prefix 为 Steam `compatdata/2330477040`，runner 需要 app `4183110` 对应的 `SteamLinuxRuntime_4`；保留Steam、runner、prefix和BetterGI顺序，获批接入仅替换原批处理末行的旧解锁器调用

开始实施时仓库只有 Cargo 初始代码，缺少 Windows 标准库与 MinGW；本轮已在忽略目录 `target/toolchain/` 安装隔离 Rust `1.98.1` 与 GNU 交叉工具链，系统 Rust 保持不变，默认 `x86_64-pc-windows-gnu` release 构建成功，产物确认是 PE32+ x86-64 控制台 EXE

初版Linux宿主7项纯逻辑测试通过，Windows8项库测试在Wine通过，后者增加 committed 页范围、跨 `PAGE_NOACCESS` 边界及地址溢出的读写失败检查；不同才写的新版Windows库测试增至9项并通过，这些结果仅证明测试输入下的边界处理

Wine 自建假目标已验证 Unicode／空格／空参数／引号／尾随反斜杠参数和工作目录、只读探针不写入、120 覆盖、重复控制器与已有游戏拒绝、强制结束控制器后假目标自行设置的 30 在超过 1.2 秒内保持、游戏先退出后控制器退出码为 0，以及假目标定位前以 7 早退使控制器失败且无写入

实际 Rust 控制器的隔离 Ctrl+C 测试随后通过：先确认私有控制台只含测试父进程和控制器，再发送 Ctrl+C，控制器退出 0 且打印 STOPPED，假目标继续运行，25 个重置后样本保持 30；完整 Windows 测试复跑结果为库 8 项通过、进程 5 项通过和 1 项已知 Ctrl+Break 诊断忽略

独立控制台定向 `CTRL_BREAK` 诊断实际失败：控制器退出码为 1 而非 0，没有 `STOPPED` 或 `FAILED` 日志，假目标仍存活且其自行设置的 30 保持超过 1.2 秒；在 `target/test-prefix` 中用最小 C 父子程序重复验证，子进程已注册处理函数，发送 API 返回成功，子进程仍以 1 退出且未进入处理函数，重新启用 Ctrl+C 也不改变结果

根因在当前 dwproton `11.0-12` 的 Wine 事件分发：[固定 Wine 源码的 `server/console.c`](https://github.com/wine-mirror/wine/blob/f8b1ce3c42b03899de648cea6d6315b7527e796b/server/console.c#L560-L582) 将 `CTRL_BREAK_EVENT` 转为 `SIGQUIT`，[`signal_x86_64.c`](https://github.com/wine-mirror/wine/blob/f8b1ce3c42b03899de648cea6d6315b7527e796b/dlls/ntdll/unix/signal_x86_64.c#L2526-L2538) 的 `quit_handler` 直接终止线程，不调用 Win32 控制处理函数；本机 `wineserver` 反汇编在 `propagate_console_signal+0xe` 将事件 1 映射为信号 3，本机 `ntdll.so` 的 `quit_handler` 调用 `abort_thread` 或 `user_mode_abort_thread`，与源码路径一致，二进制 SHA-256、反汇编、C 源码和日志保留在 `target/console-diagnostic/`

该 `CTRL_BREAK` 回归保留为已知环境失败并默认显式忽略，不改生产逻辑或放宽退出码断言；同一隔离 prefix 的另一最小 C 试验先确认新控制台仅含自建父子两个进程，再向该私有控制台发送 `CTRL_C_EVENT`，双方处理函数记录事件 0，子进程正常退出 0，证据为 `ctrl-c-parent.log` 和 `ctrl-c-child.log`，这只证明该最小例的 Ctrl+C 分发，不代替真实控制器 Ctrl+C 或关闭窗口验收

发布产物及哈希按对应构建记录在本机 `target/evidence/build.json`，首轮完整Steam链构建另记于 `steam-chain-install.json`，新版构建及结果另记于 `conditional-write-build.json` 和 `steam-conditional-observations.json`；重新链接可能因 PE 时间戳改变哈希，不宣称逐字节可复现

首轮真实探针未捕获控制器输出，不能证明定位；用户授权解锁后，经正常桌面 IPC 解锁并重跑捕获输出的只读探针，日志见 `target/evidence/probe-ready.log`

实测游戏 build 为 `CNRELWin7.0.0_R47805902_S47829085_D48215702`，主模块基址 `0x140000000`、大小 `0x1a344000`，唯一候选指令 `0x14161dfa8`，完整字节 `8b 0d 96 72 c9 03 eb 13 33 c0`，目标 `0x1452b5244`；初始化读数由 `-1` 变为 `60`，探针只读等待有效值稳定 500 ms 后输出 `PROBE COMPLETE: zero writes` 并以 0 退出，游戏仍存活

这项观测促成初始化只读等待，不将 `-1` 硬解释为游戏默认值或直接覆盖；假目标延迟就绪测试在 708 ms 变为 60，1261 ms 首次观察到 120，证明未知值期间无写入，稳定窗口至少 500 ms

2026-09-19 约 `03:15 +08:00` 开始限定真实写入，固定 500 ms 周期和 120 FPS，登录后进入游戏，世界、菜单及设置画面 HUD 约 121 FPS；MangoHUD 每秒记录 CSV，30分钟记录完成1800个样本／1800.145秒，初次世界加载约189–202秒出现1–19FPS，加载后1591个样本中位121.031FPS；保留一次449FPS异常样本，不将每秒采样称为逐帧基准，实际写入值始终是120

上述30分钟及前4轮生命周期试验调用链为 `SteamLinuxRuntime_4/_v2-entry-point → dwproton run → cmd.exe → 新 EXE`，同 prefix、XWayland 和 MangoHUD；当时未执行Steam shortcut、BetterGI或 `game-performance`，不将这些历史结果扩大为完整启动链或本次不同才写的验证

2026-09-19 `04:11:35–04:21:46 +08:00`，按用户本轮明确授权，通过真实入口 `steam://rungameid/10009322670912438272` 完成首轮完整链试验，构建对应实现提交 `a6cf71d`、SHA-256 `e949e68be5bea2dac65b1dcc5f538c453f004c11ac44462f27f1ae0b7dcd63c6`，实际链为 `Steam → game-performance → SteamLinuxRuntime_4 → dwproton → cmd.exe → BetterGI与控制器 → YuanShen.exe`

该轮进入世界后HUD为121FPS／8.2ms，实际写入目标始终120；实际Ctrl+C后控制器消失而游戏与BetterGI存活，正常关闭游戏后BetterGI继续运行且电源配置仍为performance，正常关闭BetterGI后37个已追踪PID全部从Steam跟踪列表移除，Steam结束运行状态、最外层包装进程退出0，电源配置恢复power-saver，idle inhibit结束时为disabled

Steam对本轮Wine子进程报告退出码-1，未捕获控制器实际Win32退出码或STOPPED尾句，因此只确认控制器退出及游戏独立存活，不声称本轮控制器退出0；BetterGI `0.65.0` 只验证启动，未验证截图或自动化

原批处理备份为当前prefix中的 `C:\genshin_bgi_unlock.cmd.before-genshin-uncap-20260919`，仅末行替换为 `"C:\genshin-uncap.exe" --game "C:\Program Files\miHoYo Launcher\games\Genshin Impact Game\YuanShen.exe" --fps 120`，其余内容保持不变，接入保留；安装、启动、进程与观测证据分别为 `target/evidence/steam-chain-{install,run,processes,observations}.json` 和 `steam-chain-gameprocess.log`，均属于忽略的本机证据，不保留含账号标识的游戏截图

20 秒宿主进程采样中，控制器 CPU 为单核约 0.05%，RSS 8240 KiB，主动上下文切换约 41.09 次/秒；`VmHWM` 为 430732 KiB，第二轮100ms采样将峰值定位在Launching阶段、早于Launched PID与模块扫描，内部具体分配来源未跟踪，不能只用稳态8MiB描述启动内存，也不能把上下文切换直接等同精确唤醒次数

未覆盖事项包括剧情过场、niri原生最小化、模块迟到和错误prefix场景，以及启动瞬时峰值的内部来源；逐项结果以任务表为准

初始生产路径仅研究 `8B 0D ?? ?? ?? ?? EB ?? 33 C0` 的 RIP-relative 读取，特征命中只是候选；页面可写、初值等于 60 或恰好只有一个命中均不能单独证明地址语义，必须结合模块、指令、范围与真实实验

## 实验顺序与记录模板

先做纯逻辑与自建假目标，再做真实 Proton 只读探针，证据充分后进入已授权的限定外部写入试验，最后运行生命周期和资源回归

每项实验记录 `日期／代码版本／命令／runner／prefix／游戏 build／场景／观测／结论／状态`，原始输出只保留定位和性能所需信息，不记录或上传账号信息

| 实验 | 当前状态 | 已有证据与边界 |
| --- | --- | --- |
| PE、扫描、参数与地址纯逻辑 | PASS | Linux7项通过；新版Wine执行的Windows库9项另含读写与条件写入检查，未覆盖事项见任务表 |
| Windows EXE 构建及 imports 检查 | PASS | GNU release 构建、PE32+ 控制台格式及 imports 已检查，最终大小和哈希记录在 build.json |
| 自建 Windows 假目标已列场景 | PASS | 启动、参数、探针、覆盖、重复拒绝、强制停止和游戏先退出 |
| 假目标定向 CTRL_BREAK 诊断 | FAIL | 退出码 1，假目标仍存活，最小 C 及本机二进制确认 Wine SIGQUIT 分发路径，测试默认忽略但未修复 runner |
| 自建假目标未列初始化与控制台场景 | NOT RUN | 模块迟到和窗口关闭尚未覆盖；多候选进程级拒绝、Ctrl+C 与 60 秒超时已分别通过 |
| 当前 Proton 只读探针 | PASS | 候选、模块、初始化 -1→60、零写入及退出码 0 已留存 |
| 500 ms 真实外部写入初步试验 | PASS | 120 FPS 已在游戏、菜单和设置 HUD 实测，完整场景仍待验收；无需尝试更快周期 |
| 真实30分钟与3次退出循环 | PASS | 完成连续记录及Ctrl+C、关闭控制台、强制结束、游戏先退出4轮，具体边界见任务表 |
| 首轮真实完整Steam链 | PASS | 提交a6cf71d对应e949e68b构建经真实入口启动、世界HUD121、独立退出与wrapper清理通过；BetterGI仅验证启动 |
| 500 ms检查、不同才写的新版窄测 | PASS | 新版a8b84948构建，Linux7项、Windows库9项和进程4项通过，已知诊断忽略1项，之前60秒超时过滤；无写权限句柄证明相同值无需写API，不同值仍尝试写入 |
| 不同才写的新版完整Steam复验 | PASS | a8b84948构建经同一入口进入世界HUD121，游戏先退出后控制器结束，关闭BetterGI后36个PID清理、wrapper退出0；未再运行30分钟，不扩张旧构建证据 |
| CPU、内存初步测量 | PASS | 20 秒样本 CPU 0.05% 单核、RSS 8240 KiB、VmHWM 430732 KiB；启动增量和精确唤醒仍未验证 |

无匹配或读写被拒绝时保留阶段、Win32 错误和候选证据并标记 `BLOCKED`；外部写入不可行时不新增第二套后端，行为或执行架构变更另行评审

## 许可证与参考索引

- [xiaonian MIT LICENSE](https://github.com/xiaonian233/genshin-fps-unlock/blob/2f5f5e60882eb6c5e64159efe1eb36928e4f29ea/LICENSE)
- [347 MIT LICENSE](https://github.com/34736384/genshin-fps-unlock/blob/2b85d61dd06f6e11ad86fdd6bd90339f9abc58eb/LICENSE)
- [Microsoft CreateRemoteThread](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-createremotethread) 说明成功返回不保证入口有效，适用于理解旧实现证据，不是 V1 使用该 API 的理由
- [Microsoft x64 calling convention](https://learn.microsoft.com/en-us/cpp/build/x64-calling-convention) 用于审查参考 payload 的 ABI 假设，V1 不移植该 payload
