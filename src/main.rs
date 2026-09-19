#![cfg_attr(windows, windows_subsystem = "windows")]

#[cfg(windows)]
fn main() {
    use genshin_uncap::{cli, output::Output};

    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let mut output = match Output::new(cli::hidden_requested(&args)) {
        Ok(output) => output,
        Err(error) => {
            genshin_uncap::output::show_error(&error);
            std::process::exit(1);
        }
    };
    let options = match cli::parse(args) {
        Ok(cli::Action::Run(options)) => options,
        Ok(cli::Action::Print(message)) => {
            if let Err(error) = output.line(format_args!("{message}")) {
                output.failure(&error);
                std::process::exit(1);
            }
            return;
        }
        Err(error) => {
            output.failure(&format!("Arguments: {error}\n{}", cli::USAGE));
            std::process::exit(2);
        }
    };
    if let Err(error) = genshin_uncap::windows::run(options, &mut output) {
        output.failure(&format!(
            "FAILED: {error}\nNo further writes; exit does not restore an FPS value"
        ));
        std::process::exit(1);
    }
}

#[cfg(not(windows))]
fn main() {
    eprintln!("This executable requires Windows APIs; build on Windows for x86_64-pc-windows-msvc");
    std::process::exit(1);
}
