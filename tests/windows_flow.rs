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
    Foundation::{CloseHandle, WAIT_OBJECT_0},
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
};

const CONTROLLER: &str = env!("CARGO_BIN_EXE_genshin-uncap");
const EXTRA_ARGS: [&str; 5] = ["", "原神 test", "a\"b", "ends\\", "\\\\\"quoted"];
static TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

enum Fixture {
    Fps,
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
            Fixture::Fps | Fixture::DelayedReady | Fixture::MissingText | Fixture::Ambiguous => {
                "fake_game"
            }
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
        self.contents()
            .lines()
            .filter_map(|line| line.strip_prefix("SAMPLE "))
            .filter_map(|line| line.split_whitespace().nth(1)?.parse().ok())
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
        let samples: Vec<_> = report
            .split_once("RESET ")
            .unwrap()
            .1
            .lines()
            .filter_map(|line| line.strip_prefix("SAMPLE "))
            .filter_map(|line| line.split_whitespace().nth(1))
            .collect();
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
    let before_values: Vec<_> = before
        .lines()
        .filter_map(|line| line.strip_prefix("SAMPLE "))
        .filter_map(|line| line.split_whitespace().nth(1))
        .collect();
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
