use std::{ffi::OsString, path::PathBuf};

pub const USAGE: &str = "Usage: genshin-uncap [--game <path>] [--fps <1..120>] [--hidden] [--probe] [-- <game arguments...>]\n       genshin-uncap --help | --version\n\n--game <path>  Game executable to launch; omit to detect or wait for Genshin\n--fps <1..120> Target FPS (default: 120)\n--hidden      No controller console; ignored when --game is omitted\n--probe       Locate and report only; never write game memory\n-h, --help    Show this help without launching the game\n-V, --version Show version and Rust build information without launching the game\n--            Forward arguments when launching a game; ignored without --game\n\nF10 in the game: pause and restore the initial FPS value, or resume\nExit stops writing without restoring a value";

pub const VERSION: &str = concat!(
    env!("CARGO_PKG_NAME"),
    " ",
    env!("CARGO_PKG_VERSION"),
    "\n",
    env!("BUILD_RUSTC"),
    "\ntarget: ",
    env!("BUILD_TARGET"),
    "\nprofile: ",
    env!("BUILD_PROFILE"),
);

#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    Run(Options),
    Print(&'static str),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Options {
    pub game: Option<PathBuf>,
    pub fps: i32,
    pub probe: bool,
    pub args: Vec<OsString>,
}

pub fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Action, String> {
    let mut args = args.into_iter();
    let mut game = None;
    let mut fps = None;
    let mut probe = false;
    let mut hidden = false;
    let mut forwarded = Vec::new();
    while let Some(arg) = args.next() {
        match arg.to_str() {
            Some("--help" | "-h") => return Ok(Action::Print(USAGE)),
            Some("--version" | "-V") => return Ok(Action::Print(VERSION)),
            Some("--game") if game.is_none() => {
                game = Some(PathBuf::from(args.next().ok_or("--game requires a path")?));
            }
            Some("--fps") if fps.is_none() => {
                let value = args.next().ok_or("--fps requires an integer")?;
                let value = value.to_str().ok_or("FPS must be an integer")?;
                if value.is_empty() || !value.bytes().all(|c| c.is_ascii_digit()) {
                    return Err("FPS must be an integer in 1..120".into());
                }
                let value: i32 = value.parse().map_err(|_| "FPS must be in 1..120")?;
                if !(1..=120).contains(&value) {
                    return Err("FPS must be in 1..120".into());
                }
                fps = Some(value);
            }
            Some("--probe") if !probe => probe = true,
            Some("--hidden") if !hidden => hidden = true,
            Some("--") => {
                forwarded.extend(args);
                break;
            }
            _ => return Err(format!("unknown or repeated option: {arg:?}")),
        }
    }
    if game
        .as_ref()
        .is_some_and(|game| game.as_os_str().is_empty())
    {
        return Err("game path is empty".into());
    }
    Ok(Action::Run(Options {
        game,
        fps: fps.unwrap_or(120),
        probe,
        args: forwarded,
    }))
}

// Choose the output before parsing so hidden help and argument errors never create a console
pub fn hidden_requested(args: &[OsString]) -> bool {
    let mut args = args.iter();
    let mut game = false;
    let mut hidden = false;
    while let Some(arg) = args.next() {
        match arg.to_str() {
            Some("--") => break,
            Some("--game") => {
                game = true;
                args.next();
            }
            Some("--fps") => {
                args.next();
            }
            Some("--hidden") => hidden = true,
            _ => {}
        }
    }
    game && hidden
}

#[cfg(test)]
mod tests {
    use super::*;
    fn args(a: &[&str]) -> Vec<OsString> {
        a.iter().map(OsString::from).collect()
    }

    fn run_options(input: &[&str]) -> Options {
        let Action::Run(options) = parse(args(input)).unwrap() else {
            panic!("expected game options");
        };
        options
    }

    #[test]
    fn informational_flags_do_not_require_a_game_and_respect_forwarding() {
        for (flag, text) in [
            ("--help", USAGE),
            ("-h", USAGE),
            ("--version", VERSION),
            ("-V", VERSION),
        ] {
            assert_eq!(parse(args(&[flag])), Ok(Action::Print(text)));
            assert_eq!(parse(args(&["--hidden", flag])), Ok(Action::Print(text)));
            assert!(!hidden_requested(&args(&[flag, "--hidden"])));
            assert_eq!(
                run_options(&["--game", "g.exe", "--", flag]).args,
                args(&[flag])
            );
            assert_eq!(
                run_options(&["--game", flag]).game,
                Some(PathBuf::from(flag))
            );
        }
        assert!(VERSION.starts_with(concat!(
            env!("CARGO_PKG_NAME"),
            " ",
            env!("CARGO_PKG_VERSION"),
            "\nrustc "
        )));
        assert!(VERSION.contains(concat!("\ntarget: ", env!("BUILD_TARGET"))));
        assert!(VERSION.ends_with(concat!("\nprofile: ", env!("BUILD_PROFILE"))));
    }

    #[test]
    fn defaults_unicode_and_exact_argument_forwarding() {
        let options = run_options(&[
            "--game",
            "C:\\原 神\\game.exe",
            "--",
            "",
            "a b",
            "x\"y",
            "--fps",
            "999",
        ]);
        assert_eq!(options.game, Some(PathBuf::from("C:\\原 神\\game.exe")));
        assert_eq!(options.fps, 120);
        assert_eq!(options.args, args(&["", "a b", "x\"y", "--fps", "999"]));
        assert!(!options.probe);
    }

    #[test]
    fn invalid_input_and_duplicates_are_rejected() {
        for value in ["1", "120"] {
            assert_eq!(
                run_options(&["--game", "g.exe", "--fps", value]).fps,
                value.parse::<i32>().unwrap()
            );
        }
        for value in ["0", "121", "1000", "-1", "+60", "1.5", "99999999999999", ""] {
            assert!(parse(args(&["--game", "g.exe", "--fps", value])).is_err());
        }
        for a in [
            vec!["--game"],
            vec!["--game", ""],
            vec!["--game", "g", "--unknown"],
            vec!["--game", "g", "--fps", "60", "--fps", "120"],
            vec!["--game", "g", "--probe", "--probe"],
            vec!["--game", "g", "--hidden", "--hidden"],
        ] {
            assert!(parse(args(&a)).is_err());
        }
        let options = run_options(&["--game", "g.exe", "--fps", "120", "--probe"]);
        assert_eq!(options.fps, 120);
        assert!(options.probe);
    }

    #[test]
    fn hidden_applies_to_controller_options_only() {
        let options = run_options(&["--hidden", "--game", "g.exe", "--", "--hidden"]);
        assert_eq!(options.args, args(&["--hidden"]));
        let input = vec!["--game", "g.exe", "--hidden"];
        assert!(hidden_requested(&args(&input)));
        for input in [
            vec!["--game", "g.exe", "--", "--hidden"],
            vec!["--game", "--hidden"],
            vec!["--fps", "--hidden"],
        ] {
            assert!(!hidden_requested(&args(&input)));
        }
        assert_eq!(run_options(&[]).game, None);
        let options = run_options(&["--hidden", "--fps", "60", "--probe", "--", "ignored"]);
        assert_eq!(options.fps, 60);
        assert!(options.probe);
        assert_eq!(options.game, None);
        assert_eq!(options.args, args(&["ignored"]));
    }
}
