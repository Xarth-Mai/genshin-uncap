#[cfg(all(windows, target_arch = "x86_64"))]
use std::sync::atomic::{AtomicI32, Ordering};

#[cfg(all(windows, target_arch = "x86_64"))]
#[used]
#[unsafe(no_mangle)]
static fake_fps: AtomicI32 = AtomicI32::new(60);

#[cfg(all(windows, target_arch = "x86_64"))]
std::arch::global_asm!(
    ".text",
    ".global fake_pattern",
    "fake_pattern:",
    "mov ecx, dword ptr [rip + fake_fps]",
    ".byte 0xeb, 0x00, 0x33, 0xc0",
    "ret",
);

#[cfg(all(windows, target_arch = "x86_64"))]
unsafe extern "C" {
    fn fake_pattern();
}

#[cfg(all(windows, target_arch = "x86_64"))]
struct FixtureWindow(windows_sys::Win32::Foundation::HWND);

#[cfg(all(windows, target_arch = "x86_64"))]
impl FixtureWindow {
    fn new() -> std::io::Result<Self> {
        use windows_sys::{
            Win32::UI::WindowsAndMessaging::{CreateWindowExW, WS_OVERLAPPEDWINDOW, WS_VISIBLE},
            core::w,
        };
        let title: Vec<_> = format!("genshin-uncap fixture {}", std::process::id())
            .encode_utf16()
            .chain(Some(0))
            .collect();
        let handle = unsafe {
            CreateWindowExW(
                0,
                w!("STATIC"),
                title.as_ptr(),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                100,
                100,
                640,
                240,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null(),
            )
        };
        if handle.is_null() {
            return Err(std::io::Error::last_os_error());
        }
        Ok(Self(handle))
    }

    fn pump(&self) -> bool {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            DispatchMessageW, IsWindow, PM_REMOVE, PeekMessageW, TranslateMessage,
        };
        let mut message = unsafe { std::mem::zeroed() };
        unsafe {
            while PeekMessageW(&mut message, std::ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
            IsWindow(self.0) != 0
        }
    }
}

#[cfg(all(windows, target_arch = "x86_64"))]
impl Drop for FixtureWindow {
    fn drop(&mut self) {
        unsafe {
            windows_sys::Win32::UI::WindowsAndMessaging::DestroyWindow(self.0);
        }
    }
}

#[cfg(all(windows, target_arch = "x86_64"))]
fn main() -> std::io::Result<()> {
    use std::{
        fs::File,
        io::Write,
        os::windows::ffi::OsStrExt,
        path::Path,
        time::{Duration, Instant},
    };

    std::hint::black_box(fake_pattern as unsafe extern "C" fn());
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() < 3 {
        return Err(std::io::Error::other(
            "usage: fake_game <report> <reset-signal> <exit-signal> [arguments...]",
        ));
    }
    let delayed_ready = args
        .iter()
        .any(|argument| argument == "--fixture-delayed-ready");
    let initial_fps = args
        .iter()
        .position(|argument| argument == "--fixture-initial-fps")
        .map(|index| {
            args.get(index + 1)
                .and_then(|value| value.to_str()?.parse::<i32>().ok())
                .filter(|value| [30, 60, 120].contains(value))
                .ok_or_else(|| std::io::Error::other("fixture FPS must be 30, 60, or 120"))
        })
        .transpose()?
        .unwrap_or(60);
    fake_fps.store(initial_fps, Ordering::SeqCst);
    let initial = if delayed_ready {
        fake_fps.swap(-1, Ordering::SeqCst)
    } else {
        fake_fps.load(Ordering::SeqCst)
    };
    let start = Instant::now();
    let mut report = File::create(&args[0])?;
    writeln!(report, "PID {}", std::process::id())?;
    writeln!(report, "INITIAL {initial}")?;
    let window = args
        .iter()
        .any(|argument| argument == "--fixture-window")
        .then(FixtureWindow::new)
        .transpose()?;
    if let Some(window) = &window {
        writeln!(report, "HWND {:x}", window.0 as usize)?;
    }
    for (index, argument) in args.iter().enumerate() {
        write!(report, "ARG {index} ")?;
        for word in argument.encode_wide() {
            write!(report, "{word:04x}")?;
        }
        writeln!(report)?;
    }
    write!(report, "CWD ")?;
    for word in std::fs::canonicalize(std::env::current_dir()?)?
        .as_os_str()
        .encode_wide()
    {
        write!(report, "{word:04x}")?;
    }
    writeln!(report)?;
    let mut reset = false;
    let mut ready = !delayed_ready;
    let lifetime = Duration::from_secs(if window.is_some() { 120 } else { 10 });
    while start.elapsed() < lifetime && !Path::new(&args[2]).exists() {
        if window.as_ref().is_some_and(|window| !window.pump()) {
            break;
        }
        if !ready && start.elapsed() >= Duration::from_millis(700) {
            fake_fps.store(initial_fps, Ordering::SeqCst);
            ready = true;
            writeln!(report, "READY {}", start.elapsed().as_millis())?;
        }
        if !reset && Path::new(&args[1]).exists() {
            let value = std::fs::read_to_string(&args[1])?
                .trim()
                .parse()
                .unwrap_or(30);
            fake_fps.store(value, Ordering::SeqCst);
            reset = true;
            writeln!(report, "RESET {}", start.elapsed().as_millis())?;
        }
        writeln!(
            report,
            "SAMPLE {} {}",
            start.elapsed().as_millis(),
            fake_fps.load(Ordering::SeqCst)
        )?;
        std::thread::sleep(Duration::from_millis(50));
    }
    writeln!(report, "EXIT {}", start.elapsed().as_millis())?;
    Ok(())
}

#[cfg(not(all(windows, target_arch = "x86_64")))]
fn main() {
    eprintln!("The fake target requires x86-64 Windows APIs");
}
