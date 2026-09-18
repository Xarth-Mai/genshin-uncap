use std::{
    fmt,
    fs::{self, File},
    io::{self, Write},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use windows_sys::Win32::{
    Foundation::{ERROR_ACCESS_DENIED, GetHandleInformation, GetLastError, INVALID_HANDLE_VALUE},
    System::Console::{
        ATTACH_PARENT_PROCESS, AllocConsole, AttachConsole, GetConsoleCP, GetStdHandle,
        STD_ERROR_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE, SetStdHandle,
    },
    UI::WindowsAndMessaging::{MB_ICONERROR, MB_OK, MessageBoxW},
};

pub struct Output {
    log: Option<(File, PathBuf)>,
}

impl Output {
    pub fn new(hidden: bool) -> Result<Self, String> {
        if !hidden {
            prepare_console()?;
            return Ok(Self { log: None });
        }
        let directory = PathBuf::from(
            std::env::var_os("LOCALAPPDATA").ok_or("LOCALAPPDATA is missing; cannot create log")?,
        )
        .join("genshin-uncap");
        fs::create_dir_all(&directory)
            .map_err(|error| format!("create log directory {}: {error}", directory.display()))?;
        cleanup_logs(&directory);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| format!("log timestamp: {error}"))?
            .as_nanos();
        let path = directory.join(format!("{timestamp}-{}.log", std::process::id()));
        let log = File::create_new(&path)
            .map_err(|error| format!("create log {}: {error}", path.display()))?;
        let mut output = Self {
            log: Some((log, path)),
        };
        output.line(format_args!("genshin-uncap: hidden startup"))?;
        Ok(output)
    }

    pub fn line(&mut self, message: fmt::Arguments<'_>) -> Result<(), String> {
        match self.log.as_mut() {
            Some((file, path)) => write_line(file, message)
                .map_err(|error| format!("write log {}: {error}", path.display())),
            None => write_line(&mut io::stdout().lock(), message)
                .map_err(|error| format!("write stdout: {error}")),
        }
    }

    // Called only after the controller has returned and all memory writes have stopped
    pub fn failure(&mut self, message: &str) {
        if self.log.is_some() {
            let log_error = self.line(format_args!("{message}")).err();
            let path = &self.log.as_ref().unwrap().1;
            let mut description = format!("{message}\n\nLog: {}", path.display());
            if let Some(error) = log_error {
                description.push_str(&format!("\n{error}"));
            }
            show_error(&description);
        } else {
            let _ = write_line(&mut io::stderr().lock(), format_args!("{message}"));
        }
    }
}

fn cleanup_logs(directory: &std::path::Path) {
    let mut logs: Vec<_> = fs::read_dir(directory)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            if !entry.file_type().ok()?.is_file() || path.extension()?.to_str()? != "log" {
                return None;
            }
            let (timestamp, pid) = path.file_stem()?.to_str()?.rsplit_once('-')?;
            let timestamp = timestamp.parse::<u128>().ok()?;
            let pid = pid.parse::<u32>().ok()?;
            Some((timestamp, pid, path))
        })
        .collect();
    logs.sort_unstable();
    let excess = logs.len().saturating_sub(9);
    for (_, _, path) in logs.into_iter().take(excess) {
        let _ = fs::remove_file(path);
    }
}

fn write_line(output: &mut impl Write, message: fmt::Arguments<'_>) -> io::Result<()> {
    writeln!(output, "{message}")?;
    output.flush()
}

fn prepare_console() -> Result<(), String> {
    if unsafe { GetConsoleCP() } != 0 {
        return Ok(());
    }
    // Attaching initializes std handles; retain any valid inherited pipe/file redirection
    let inherited = [STD_INPUT_HANDLE, STD_OUTPUT_HANDLE, STD_ERROR_HANDLE].map(|id| {
        let handle = unsafe { GetStdHandle(id) };
        let mut flags = 0;
        let valid = !handle.is_null()
            && handle != INVALID_HANDLE_VALUE
            && unsafe { GetHandleInformation(handle, &mut flags) } != 0;
        (id, handle, valid)
    });
    if unsafe { AttachConsole(ATTACH_PARENT_PROCESS) } == 0
        && unsafe { GetLastError() } != ERROR_ACCESS_DENIED
        && unsafe { AllocConsole() } == 0
    {
        return Err(format!(
            "attach or create console: {}",
            io::Error::last_os_error()
        ));
    }
    for (id, handle, valid) in inherited {
        if valid && unsafe { SetStdHandle(id, handle) } == 0 {
            return Err(format!(
                "restore inherited stream: {}",
                io::Error::last_os_error()
            ));
        }
    }
    Ok(())
}

pub fn show_error(message: &str) {
    let text: Vec<_> = message.encode_utf16().chain(Some(0)).collect();
    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            text.as_ptr(),
            windows_sys::core::w!("genshin-uncap"),
            MB_OK | MB_ICONERROR,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::cleanup_logs;
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn cleanup_keeps_recent_logs_and_ignores_other_files() {
        let directory = std::env::temp_dir().join(format!(
            "genshin-uncap-log-test-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&directory).unwrap();
        for timestamp in 1..=11 {
            fs::write(directory.join(format!("{timestamp}-1.log")), []).unwrap();
        }
        fs::write(directory.join("keep.txt"), []).unwrap();

        cleanup_logs(&directory);

        let logs = fs::read_dir(&directory)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| {
                entry.path().extension().and_then(|value| value.to_str()) == Some("log")
            })
            .count();
        assert_eq!(logs, 9);
        assert!(!directory.join("1-1.log").exists());
        assert!(!directory.join("2-1.log").exists());
        assert!(directory.join("10-1.log").exists());
        assert!(directory.join("11-1.log").exists());
        assert!(directory.join("keep.txt").exists());
        fs::remove_dir_all(directory).unwrap();
    }
}
