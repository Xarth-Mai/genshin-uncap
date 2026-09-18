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
    let initial = if delayed_ready {
        fake_fps.swap(-1, Ordering::SeqCst)
    } else {
        fake_fps.load(Ordering::SeqCst)
    };
    let start = Instant::now();
    let mut report = File::create(&args[0])?;
    writeln!(report, "PID {}", std::process::id())?;
    writeln!(report, "INITIAL {initial}")?;
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
    while start.elapsed() < Duration::from_secs(10) && !Path::new(&args[2]).exists() {
        if !ready && start.elapsed() >= Duration::from_millis(700) {
            fake_fps.store(60, Ordering::SeqCst);
            ready = true;
            writeln!(report, "READY {}", start.elapsed().as_millis())?;
        }
        if !reset && Path::new(&args[1]).exists() {
            fake_fps.store(30, Ordering::SeqCst);
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
