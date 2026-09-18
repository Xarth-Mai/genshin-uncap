fn main() {
    let options = match genshin_uncap::cli::parse(std::env::args_os().skip(1)) {
        Ok(Some(options)) => options,
        Ok(None) => {
            println!("{}", genshin_uncap::cli::USAGE);
            return;
        }
        Err(error) => {
            eprintln!("Arguments: {error}\n{}", genshin_uncap::cli::USAGE);
            std::process::exit(2);
        }
    };
    #[cfg(windows)]
    if let Err(error) = genshin_uncap::windows::run(options) {
        eprintln!("FAILED: {error}\nNo further writes; no FPS value restored");
        std::process::exit(1);
    }
    #[cfg(not(windows))]
    {
        let _ = options;
        eprintln!("This executable requires Windows APIs; build for x86_64-pc-windows-gnu");
        std::process::exit(1);
    }
}
