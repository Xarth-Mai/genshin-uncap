use std::{ffi::OsString, path::PathBuf};

pub const USAGE: &str = "genshin-uncap --game <path> [--fps <1..120>] [--probe] [-- <game arguments...>]\nDefault: 120 FPS; exit stops writing without restoring a value\n--probe: launch, locate and report only; never write game memory";

#[derive(Debug, PartialEq, Eq)]
pub struct Options {
    pub game: PathBuf,
    pub fps: i32,
    pub probe: bool,
    pub args: Vec<OsString>,
}

pub fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Option<Options>, String> {
    let mut args = args.into_iter();
    let mut game = None;
    let mut fps = None;
    let mut probe = false;
    let mut forwarded = Vec::new();
    while let Some(arg) = args.next() {
        match arg.to_str() {
            Some("--help" | "-h") => return Ok(None),
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
            Some("--") => {
                forwarded.extend(args);
                break;
            }
            _ => return Err(format!("unknown or repeated option: {arg:?}")),
        }
    }
    let game = game.ok_or("--game is required")?;
    if game.as_os_str().is_empty() {
        return Err("game path is empty".into());
    }
    Ok(Some(Options {
        game,
        fps: fps.unwrap_or(120),
        probe,
        args: forwarded,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn args(a: &[&str]) -> Vec<OsString> {
        a.iter().map(OsString::from).collect()
    }

    #[test]
    fn defaults_unicode_and_exact_argument_forwarding() {
        let options = parse(args(&[
            "--game",
            "C:\\原 神\\game.exe",
            "--",
            "",
            "a b",
            "x\"y",
            "--fps",
            "999",
        ]))
        .unwrap()
        .unwrap();
        assert_eq!(options.game, PathBuf::from("C:\\原 神\\game.exe"));
        assert_eq!(options.fps, 120);
        assert_eq!(options.args, args(&["", "a b", "x\"y", "--fps", "999"]));
        assert!(!options.probe);
    }

    #[test]
    fn invalid_input_and_duplicates_are_rejected() {
        for value in ["1", "120"] {
            assert_eq!(
                parse(args(&["--game", "g.exe", "--fps", value]))
                    .unwrap()
                    .unwrap()
                    .fps,
                value.parse::<i32>().unwrap()
            );
        }
        for value in ["0", "121", "1000", "-1", "+60", "1.5", "99999999999999", ""] {
            assert!(parse(args(&["--game", "g.exe", "--fps", value])).is_err());
        }
        for a in [
            vec![],
            vec!["--game"],
            vec!["--game", ""],
            vec!["--game", "g", "--unknown"],
            vec!["--game", "g", "--fps", "60", "--fps", "120"],
            vec!["--game", "g", "--probe", "--probe"],
        ] {
            assert!(parse(args(&a)).is_err());
        }
        let options = parse(args(&["--game", "g.exe", "--fps", "120", "--probe"]))
            .unwrap()
            .unwrap();
        assert_eq!(options.fps, 120);
        assert!(options.probe);
    }
}
