fn main() -> std::io::Result<()> {
    use std::{
        fs::File,
        io::Write,
        path::Path,
        time::{Duration, Instant},
    };
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() < 3 {
        return Err(std::io::Error::other(
            "usage: no_pattern <report> <reset-signal> <exit-signal> [--fixture-timeout]",
        ));
    }
    let mut report = File::create(&args[0])?;
    writeln!(report, "PID {}", std::process::id())?;
    let timeout = args.iter().any(|argument| argument == "--fixture-timeout");
    let lifetime = if timeout {
        Duration::from_secs(65)
    } else {
        Duration::from_millis(300)
    };
    let start = Instant::now();
    while start.elapsed() < lifetime && !Path::new(&args[2]).exists() {
        std::thread::sleep(Duration::from_millis(25));
    }
    writeln!(report, "EXIT {}", start.elapsed().as_millis())?;
    std::process::exit(7);
}
