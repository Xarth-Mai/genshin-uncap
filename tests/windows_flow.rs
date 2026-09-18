#![cfg(all(windows, target_arch = "x86_64"))]

use std::{
    ffi::OsStr,
    fs::{self, File},
    os::windows::{ffi::OsStrExt, process::CommandExt},
    path::{Path, PathBuf},
    process::{Child, Command, ExitStatus, Stdio},
    thread::sleep,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use windows_sys::Win32::{
    Foundation::{CloseHandle, HWND, WAIT_OBJECT_0},
    System::{
        Console::{
            AllocConsole, CTRL_BREAK_EVENT, CTRL_C_EVENT, FreeConsole, GenerateConsoleCtrlEvent,
            GetConsoleProcessList, GetStdHandle, STD_ERROR_HANDLE, STD_OUTPUT_HANDLE,
            SetConsoleCtrlHandler, SetStdHandle,
        },
        Threading::{
            CREATE_NEW_PROCESS_GROUP, OpenProcess, PROCESS_SYNCHRONIZE, WaitForSingleObject,
        },
    },
    UI::{
        Input::KeyboardAndMouse::{
            INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, SendInput, VK_CONTROL,
            VK_F10,
        },
        WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId, SetForegroundWindow},
    },
};

const CONTROLLER: &str = env!("CARGO_BIN_EXE_genshin-uncap");
const EXTRA_ARGS: [&str; 5] = ["", "原神 test", "a\"b", "ends\\", "\\\\\"quoted"];
static TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

enum Fixture {
    Fps,
    Window(i32),
    DelayedReady,
    MissingText,
    Ambiguous,
    NoPattern,
    Timeout,
}

fn encoded(value: &OsStr) -> String {
    value
        .encode_wide()
        .map(|word| format!("{word:04x}"))
        .collect()
}

fn sample_values(report: &str) -> impl Iterator<Item = &str> {
    report
        .lines()
        .filter_map(|line| line.strip_prefix("SAMPLE "))
        .filter_map(|line| line.split_whitespace().nth(1))
}

fn wait_until(mut condition: impl FnMut() -> bool, context: &str) {
    let deadline = Instant::now() + Duration::from_secs(6);
    while !condition() {
        assert!(Instant::now() < deadline, "timed out: {context}");
        sleep(Duration::from_millis(25));
    }
}

struct Session {
    root: PathBuf,
    game: PathBuf,
    report: PathBuf,
    reset: PathBuf,
    exit: PathBuf,
    log: PathBuf,
    child: Child,
}

impl Session {
    fn start(probe: bool, fixture: Fixture) -> Self {
        Self::with_creation_flags(probe, fixture, CREATE_NEW_PROCESS_GROUP)
    }

    fn with_creation_flags(probe: bool, fixture: Fixture, flags: u32) -> Self {
        let example = match fixture {
            Fixture::Fps
            | Fixture::Window(_)
            | Fixture::DelayedReady
            | Fixture::MissingText
            | Fixture::Ambiguous => "fake_game",
            Fixture::NoPattern | Fixture::Timeout => "no_pattern",
        };
        let source = Path::new(CONTROLLER)
            .parent()
            .unwrap()
            .join(format!("examples/{example}.exe"));
        assert!(
            source.is_file(),
            "build fixtures first: cargo build --examples"
        );
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "genshin-uncap-{}-{suffix}-测试 spaces",
            std::process::id()
        ));
        fs::create_dir_all(&root).unwrap();
        let game = root.join(format!("{example}.exe"));
        fs::copy(source, &game).unwrap();
        if matches!(fixture, Fixture::Ambiguous) {
            let mut image = fs::read(&game).unwrap();
            let coff_end = genshin_uncap::scan::pe_coff_header_len(&image).unwrap();
            let optional_size =
                u16::from_le_bytes(image[coff_end - 4..coff_end - 2].try_into().unwrap()) as usize;
            let header_end = genshin_uncap::scan::pe_header_len(&image).unwrap();
            let section = image[coff_end + optional_size..header_end]
                .chunks_exact(40)
                .find(|section| &section[..8] == b".text\0\0\0")
                .unwrap();
            let word = |offset| {
                u32::from_le_bytes(section[offset..offset + 4].try_into().unwrap()) as usize
            };
            let (virtual_size, rva, raw_size, raw_start) = (word(8), word(12), word(16), word(20));
            let length = genshin_uncap::scan::PATTERN_LEN;
            // Add a second locator only in non-executed file alignment padding
            assert!(raw_size >= virtual_size + length);
            let patch_offset = raw_size - length;
            assert!(
                image[raw_start + patch_offset..raw_start + raw_size]
                    .iter()
                    .all(|&byte| byte == 0)
            );
            let mut scanner = genshin_uncap::scan::PatternScanner::default();
            let original = scanner
                .feed(rva, &image[raw_start..raw_start + raw_size])
                .unwrap();
            assert_eq!(original.len(), 1);
            let file_offset = raw_start + original[0] - rva;
            let mut evidence = image[file_offset..file_offset + length].to_vec();
            let target = genshin_uncap::scan::fps_address(original[0], &evidence).unwrap();
            let displacement =
                i32::try_from(target as i64 + 4 - (rva + patch_offset + 6) as i64).unwrap();
            evidence[2..6].copy_from_slice(&displacement.to_le_bytes());
            image[raw_start + patch_offset..raw_start + raw_size].copy_from_slice(&evidence);
            fs::write(&game, image).unwrap();
        }
        if matches!(fixture, Fixture::MissingText) {
            let mut image = fs::read(&game).unwrap();
            let optional = genshin_uncap::scan::pe_coff_header_len(&image).unwrap();
            let optional_size =
                u16::from_le_bytes(image[optional - 4..optional - 2].try_into().unwrap()) as usize;
            let header_end = genshin_uncap::scan::pe_header_len(&image).unwrap();
            let mut renamed = 0;
            for section in image[optional + optional_size..header_end].chunks_exact_mut(40) {
                if &section[..8] == b".text\0\0\0" {
                    section[..8].copy_from_slice(b".code\0\0\0");
                    renamed += 1;
                }
            }
            assert_eq!(renamed, 1);
            fs::write(&game, image).unwrap();
        }
        let report = root.join("report.txt");
        let reset = root.join("reset.signal");
        let exit = root.join("exit.signal");
        let log = root.join("controller.log");
        let output = File::create(&log).unwrap();
        let mut command = Command::new(CONTROLLER);
        command.args(["--game"]).arg(&game).args(["--fps", "120"]);
        if probe {
            command.arg("--probe");
        }
        command
            .arg("--")
            .args([&report, &reset, &exit])
            .args(EXTRA_ARGS);
        if matches!(fixture, Fixture::Timeout) {
            command.arg("--fixture-timeout");
        }
        if matches!(fixture, Fixture::DelayedReady) {
            command.arg("--fixture-delayed-ready");
        }
        if let Fixture::Window(initial) = fixture {
            command
                .args(["--fixture-window", "--fixture-initial-fps"])
                .arg(initial.to_string());
        }
        let child = command
            .creation_flags(flags)
            .stdin(Stdio::null())
            .stdout(output.try_clone().unwrap())
            .stderr(output)
            .spawn()
            .unwrap();
        Self {
            root,
            game,
            report,
            reset,
            exit,
            log,
            child,
        }
    }

    fn contents(&self) -> String {
        fs::read_to_string(&self.report).unwrap_or_default()
    }

    fn samples(&self) -> Vec<i32> {
        sample_values(&self.contents())
            .filter_map(|value| value.parse().ok())
            .collect()
    }

    fn finished(&mut self) -> ExitStatus {
        self.finished_within(Duration::from_secs(6))
    }

    fn finished_within(&mut self, timeout: Duration) -> ExitStatus {
        let deadline = Instant::now() + timeout;
        loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                return status;
            }
            assert!(
                Instant::now() < deadline,
                "controller timeout: {}",
                fs::read_to_string(&self.log).unwrap_or_default()
            );
            sleep(Duration::from_millis(25));
        }
    }

    fn stop_game(&self) {
        fs::write(&self.exit, []).unwrap();
        wait_until(|| self.contents().contains("EXIT "), "fake game exit");
        let report = self.contents();
        let pid: u32 = report
            .lines()
            .next()
            .unwrap()
            .strip_prefix("PID ")
            .unwrap()
            .parse()
            .unwrap();
        let handle = unsafe { OpenProcess(PROCESS_SYNCHRONIZE, 0, pid) };
        if !handle.is_null() {
            let result = unsafe { WaitForSingleObject(handle, 5000) };
            unsafe {
                CloseHandle(handle);
            }
            assert_eq!(result, WAIT_OBJECT_0, "fake target failed to exit");
        }
    }

    fn check_arguments(&self) {
        let contents = self.contents();
        for (index, argument) in EXTRA_ARGS.iter().enumerate() {
            assert!(
                contents
                    .lines()
                    .any(|line| line
                        == format!("ARG {} {}", index + 3, encoded(OsStr::new(argument)))),
                "argument {index} was not forwarded exactly: {contents}"
            );
        }
        let cwd = format!(
            "CWD {}",
            encoded(fs::canonicalize(&self.root).unwrap().as_os_str())
        );
        let actual = contents
            .lines()
            .find(|line| line.starts_with("CWD "))
            .unwrap();
        assert_eq!(actual, cwd);
    }

    fn check_stopped_writing(&self) {
        fs::write(&self.reset, []).unwrap();
        wait_until(|| self.contents().contains("RESET "), "fake self reset");
        sleep(Duration::from_millis(1200));
        let report = self.contents();
        let samples: Vec<_> = sample_values(report.split_once("RESET ").unwrap().1).collect();
        assert!(
            samples.len() >= 20,
            "fake game did not continue after controller exit"
        );
        assert!(
            samples.iter().all(|&value| value == "30"),
            "writes continued after controller exit: {samples:?}"
        );
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        let _ = fs::write(&self.exit, []);
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
        // Keep logs on failure; fixtures also enforce their own finite lifetime
        eprintln!("fixture evidence: {}", self.root.display());
    }
}

fn fixture_key(window: HWND, pid: u32, key: u16, down: bool) -> bool {
    if down {
        let mut owner = 0;
        unsafe {
            GetWindowThreadProcessId(window, &mut owner);
        }
        if owner != pid || unsafe { GetForegroundWindow() } != window {
            return false;
        }
    }
    // Always release keys this helper injected, even if focus changed while held
    let input = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: key,
                dwFlags: if down { 0 } else { KEYEVENTF_KEYUP },
                ..Default::default()
            },
        },
    };
    unsafe { SendInput(1, &input, std::mem::size_of::<INPUT>() as i32) == 1 }
}

struct HeldFixtureKey {
    window: HWND,
    pid: u32,
    key: u16,
    held: bool,
}

impl HeldFixtureKey {
    fn press(window: HWND, pid: u32, key: u16) -> Self {
        assert!(
            fixture_key(window, pid, key, true),
            "fixture lost foreground or keyboard input failed"
        );
        Self {
            window,
            pid,
            key,
            held: true,
        }
    }

    fn release(mut self) {
        self.held = !fixture_key(self.window, self.pid, self.key, false);
        assert!(!self.held, "fixture key release failed");
    }
}

impl Drop for HeldFixtureKey {
    fn drop(&mut self) {
        if self.held && !fixture_key(self.window, self.pid, self.key, false) {
            eprintln!("cannot release injected fixture key");
        }
    }
}

#[test]
#[ignore = "requires an isolated foreground desktop; presses keys only with verified fixture focus"]
fn foreground_f10_modes() {
    let _serial = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    for (baseline, fault) in [(30, false), (60, false), (120, false), (60, true)] {
        let mut controller = Session::start(false, Fixture::Window(baseline));
        let log = || fs::read_to_string(&controller.log).unwrap_or_default();
        wait_until(
            || log().contains("CHECKING:") && controller.samples().last() == Some(&120),
            "window fixture ready",
        );
        let report = controller.contents();
        let field = |prefix| {
            report
                .lines()
                .find_map(|line| line.strip_prefix(prefix))
                .unwrap()
        };
        let pid = field("PID ").parse().unwrap();
        let window = usize::from_str_radix(field("HWND "), 16).unwrap() as HWND;
        unsafe {
            SetForegroundWindow(window);
        }
        wait_until(
            || unsafe { GetForegroundWindow() == window },
            "fixture foreground",
        );
        sleep(Duration::from_millis(150));
        let press = |duration| {
            let key = HeldFixtureKey::press(window, pid, VK_F10);
            sleep(duration);
            key.release();
            sleep(Duration::from_millis(100));
        };

        press(Duration::from_millis(800));
        wait_until(|| log().contains("PAUSED:"), "first F10 pause");
        assert_eq!(
            log().matches("PAUSED:").count(),
            1,
            "long press toggled twice"
        );
        assert_eq!(controller.samples().last(), Some(&baseline));

        let changed = if fault {
            121
        } else if baseline == 30 {
            60
        } else {
            30
        };
        fs::write(&controller.reset, changed.to_string()).unwrap();
        wait_until(
            || controller.contents().contains("RESET "),
            "paused target reset",
        );
        sleep(Duration::from_millis(1200));
        let report = controller.contents();
        let paused_values: Vec<i32> = sample_values(report.split_once("RESET ").unwrap().1)
            .filter_map(|value| value.parse().ok())
            .collect();
        assert!(paused_values.len() >= 20);
        assert!(paused_values.iter().all(|&value| value == changed));
        if fault {
            assert!(
                controller.child.try_wait().unwrap().is_none(),
                "paused controller read the invalid fixture value"
            );
            let sample_count = controller.samples().len();
            press(Duration::from_millis(100));
            assert_eq!(controller.finished().code(), Some(1));
            let failure = fs::read_to_string(&controller.log).unwrap();
            assert!(failure.contains("unverified FPS value 121; stopping"));
            assert!(!failure.contains("RESUMED:"));
            wait_until(
                || controller.samples().len() >= sample_count + 20,
                "fake target survives rejected resume",
            );
            assert!(
                controller.samples()[sample_count..]
                    .iter()
                    .all(|&value| value == 121)
            );
            controller.stop_game();
            continue;
        }
        let control = HeldFixtureKey::press(window, pid, VK_CONTROL);
        press(Duration::from_millis(100));
        control.release();
        assert!(!log().contains("RESUMED:"), "modified F10 resumed control");
        sleep(Duration::from_millis(100));

        press(Duration::from_millis(100));
        wait_until(|| log().contains("RESUMED:"), "F10 resume");
        wait_until(
            || controller.samples().last() == Some(&120),
            "resumed target",
        );
        press(Duration::from_millis(100));
        wait_until(|| log().matches("PAUSED:").count() == 2, "second F10 pause");
        wait_until(
            || controller.samples().last() == Some(&baseline),
            "immutable startup baseline restored",
        );
        controller.stop_game();
        assert!(controller.finished().success());
    }
}

#[test]
fn informational_flags_exit_without_launching() {
    let _serial = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    for hidden in [false, true] {
        for (flag, expected) in [
            ("--help", genshin_uncap::cli::USAGE),
            ("--version", genshin_uncap::cli::VERSION),
        ] {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let directory = std::env::temp_dir().join(format!(
                "genshin-uncap-info-{}-{timestamp}",
                std::process::id()
            ));
            fs::create_dir_all(&directory).unwrap();
            let mut command = Command::new(CONTROLLER);
            command.arg(flag);
            if hidden {
                command.arg("--hidden");
            }
            let mut child = command
                .env("LOCALAPPDATA", &directory)
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap();
            let deadline = Instant::now() + Duration::from_secs(6);
            while child.try_wait().unwrap().is_none() {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    panic!(
                        "informational command did not exit; evidence: {}",
                        directory.display()
                    );
                }
                sleep(Duration::from_millis(25));
            }
            let output = child.wait_with_output().unwrap();
            assert!(output.status.success());
            assert!(output.stderr.is_empty());
            if hidden {
                assert!(output.stdout.is_empty());
                let logs: Vec<_> = fs::read_dir(directory.join("genshin-uncap"))
                    .unwrap()
                    .map(|entry| entry.unwrap().path())
                    .collect();
                assert_eq!(logs.len(), 1);
                let contents = fs::read_to_string(&logs[0]).unwrap();
                assert_eq!(
                    contents,
                    format!("genshin-uncap: hidden startup\n{expected}\n")
                );
            } else {
                assert_eq!(
                    String::from_utf8(output.stdout).unwrap(),
                    format!("{expected}\n")
                );
                assert!(!directory.join("genshin-uncap").exists());
            }
            fs::remove_dir_all(directory).unwrap();
        }
    }
}

#[test]
fn real_windows_process_flow() {
    let _serial = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let mut probe = Session::start(true, Fixture::Fps);
    assert!(
        probe.finished().success(),
        "{}",
        fs::read_to_string(&probe.log).unwrap()
    );
    wait_until(|| probe.samples().len() >= 3, "probe samples");
    assert!(probe.samples().iter().all(|&value| value == 60));
    assert!(
        fs::read_to_string(&probe.log)
            .unwrap()
            .contains("PROBE COMPLETE: zero writes")
    );
    probe.check_arguments();
    probe.stop_game();
    drop(probe);

    let mut controller = Session::start(false, Fixture::Fps);
    wait_until(
        || controller.samples().contains(&120),
        "controller writes 120",
    );
    controller.check_arguments();
    let duplicate = Command::new(CONTROLLER)
        .arg("--game")
        .arg(&controller.game)
        .output()
        .unwrap();
    assert!(!duplicate.status.success());
    assert!(String::from_utf8_lossy(&duplicate.stderr).contains("another controller is running"));
    controller.child.kill().unwrap();
    assert!(!controller.finished().success());
    controller.check_stopped_writing();
    let existing = Command::new(CONTROLLER)
        .arg("--game")
        .arg(&controller.game)
        .output()
        .unwrap();
    assert!(!existing.status.success());
    assert!(String::from_utf8_lossy(&existing.stderr).contains("game is already running"));
    controller.stop_game();
    drop(controller);

    let mut controller = Session::start(false, Fixture::Fps);
    wait_until(
        || controller.samples().contains(&120),
        "second controller writes 120",
    );
    controller.stop_game();
    assert!(controller.finished().success());
    assert!(
        fs::read_to_string(&controller.log)
            .unwrap()
            .contains("STOPPED: no further writes")
    );
    drop(controller);

    let mut unsupported = Session::start(false, Fixture::NoPattern);
    assert!(!unsupported.finished().success());
    let log = fs::read_to_string(&unsupported.log).unwrap();
    assert!(
        log.contains("game exited before initialization completed (exit code 7)"),
        "{log}"
    );
    assert!(!log.contains("WRITING:"), "{log}");
    drop(unsupported);

    let mut missing_text = Session::start(false, Fixture::MissingText);
    assert!(!missing_text.finished().success());
    let log = fs::read_to_string(&missing_text.log).unwrap();
    assert!(
        log.contains("expected exactly one readable executable .text section"),
        "{log}"
    );
    assert!(!log.contains("WRITING:"), "{log}");
    wait_until(
        || missing_text.samples().len() >= 3,
        "missing .text target still alive",
    );
    assert!(missing_text.samples().iter().all(|&value| value == 60));
    missing_text.stop_game();
}

#[test]
fn ambiguous_targets_are_rejected_without_writing() {
    let _serial = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let mut target = Session::start(false, Fixture::Ambiguous);
    assert!(!target.finished().success());
    let log = fs::read_to_string(&target.log).unwrap();
    assert!(
        log.contains("ambiguous FPS candidates; refusing writes"),
        "{log}"
    );
    assert_eq!(log.matches("Candidate:").count(), 2, "{log}");
    assert!(!log.contains("WRITING:"), "{log}");
    wait_until(|| target.samples().len() >= 3, "ambiguous target alive");
    assert!(target.samples().iter().all(|&value| value == 60));
    target.stop_game();
}

#[test]
fn delayed_ready_stays_read_only_until_stable() {
    let _serial = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let mut target = Session::start(false, Fixture::DelayedReady);
    wait_until(
        || target.samples().contains(&120),
        "delayed ready target writes 120",
    );
    let report = target.contents();
    assert!(
        report.lines().any(|line| line == "INITIAL 60"),
        "startup wrote before fixture initialized: {report}"
    );
    let (before, after) = report
        .split_once("READY ")
        .expect("fixture never became ready");
    let before_values: Vec<_> = sample_values(before).collect();
    assert!(before_values.len() >= 10);
    assert!(
        before_values.iter().all(|&value| value == "-1"),
        "write before ready: {report}"
    );
    let ready_at: u64 = after.lines().next().unwrap().parse().unwrap();
    let first_write_at: u64 = after
        .lines()
        .filter_map(|line| line.strip_prefix("SAMPLE "))
        .find_map(|line| {
            let mut fields = line.split_whitespace();
            let elapsed = fields.next()?;
            (fields.next()? == "120").then(|| elapsed.parse().unwrap())
        })
        .unwrap();
    assert!(
        first_write_at >= ready_at + 500,
        "stability gate too short: {report}"
    );
    target.stop_game();
    assert!(target.finished().success());
}

#[test]
fn initialization_timeout_stops_without_writing() {
    let _serial = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let mut target = Session::start(false, Fixture::Timeout);
    let start = Instant::now();
    assert!(!target.finished_within(Duration::from_secs(63)).success());
    assert!(start.elapsed() >= Duration::from_secs(59));
    let log = fs::read_to_string(&target.log).unwrap();
    assert!(
        log.contains("timeout") || log.contains("initialization deadline"),
        "{log}"
    );
    assert!(!log.contains("WRITING:"), "{log}");
    assert!(
        !target.contents().contains("EXIT "),
        "controller terminated the fake target"
    );
    target.stop_game();
}

#[test]
fn controller_ctrl_c_exit() {
    let _serial = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let output = unsafe { GetStdHandle(STD_OUTPUT_HANDLE) };
    let errors = unsafe { GetStdHandle(STD_ERROR_HANDLE) };
    unsafe {
        FreeConsole();
    }
    assert_ne!(
        unsafe { AllocConsole() },
        0,
        "allocate private test console"
    );
    unsafe {
        SetStdHandle(STD_OUTPUT_HANDLE, output);
        SetStdHandle(STD_ERROR_HANDLE, errors);
    }
    // A handler is process-local, unlike the inheritable ignore-Ctrl+C attribute
    unsafe extern "system" fn keep_test_alive(event: u32) -> i32 {
        i32::from(event == CTRL_C_EVENT)
    }
    assert_ne!(
        unsafe { SetConsoleCtrlHandler(Some(keep_test_alive), 1) },
        0
    );
    let mut controller = Session::with_creation_flags(false, Fixture::Fps, 0);
    wait_until(
        || controller.samples().contains(&120),
        "controller ready for CTRL_C",
    );
    // Ctrl+C cannot target a group, so prove this new console has only our two processes
    let mut members = [0_u32; 3];
    assert_eq!(
        unsafe { GetConsoleProcessList(members.as_mut_ptr(), members.len() as u32) },
        2,
        "refusing Ctrl+C broadcast to an unexpected console"
    );
    members[..2].sort_unstable();
    let mut expected = [std::process::id(), controller.child.id()];
    expected.sort_unstable();
    assert_eq!(members[..2], expected);
    assert_ne!(unsafe { GenerateConsoleCtrlEvent(CTRL_C_EVENT, 0) }, 0);
    let status = controller.finished();
    let log = fs::read_to_string(&controller.log).unwrap();
    assert!(status.success(), "CTRL_C controller status {status}: {log}");
    assert!(log.contains("STOPPED:"), "{log}");
    assert!(!log.contains("FAILED:"), "{log}");
    controller.check_stopped_writing();
    controller.stop_game();
    assert_ne!(
        unsafe { SetConsoleCtrlHandler(Some(keep_test_alive), 0) },
        0
    );
    // Keep the console until the test process exits so its restored output handles stay valid
}

#[test]
#[ignore = "dwproton 11.0-12 maps CTRL_BREAK to SIGQUIT and bypasses Win32 handlers"]
fn controller_break_exit() {
    let _serial = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    // Wine's plain runner may have output pipes but no console for control events
    let output = unsafe { GetStdHandle(STD_OUTPUT_HANDLE) };
    let errors = unsafe { GetStdHandle(STD_ERROR_HANDLE) };
    unsafe {
        FreeConsole();
    }
    assert_ne!(
        unsafe { AllocConsole() },
        0,
        "allocate test console: {}",
        std::io::Error::last_os_error()
    );
    unsafe {
        SetStdHandle(STD_OUTPUT_HANDLE, output);
        SetStdHandle(STD_ERROR_HANDLE, errors);
    }
    let mut controller = Session::start(false, Fixture::Fps);
    wait_until(
        || controller.samples().contains(&120),
        "controller ready for CTRL_BREAK",
    );
    let group = controller.child.id();
    assert_ne!(group, 0);
    assert_ne!(
        unsafe { GenerateConsoleCtrlEvent(CTRL_BREAK_EVENT, group) },
        0,
        "targeted CTRL_BREAK failed: {}",
        std::io::Error::last_os_error()
    );
    let status = controller.finished();
    controller.check_stopped_writing();
    controller.stop_game();
    // The isolated console closes with this test process; detaching closes its output handles
    assert!(
        status.success(),
        "CTRL_BREAK controller status {status}: {}",
        fs::read_to_string(&controller.log).unwrap()
    );
}
