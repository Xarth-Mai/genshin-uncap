use crate::{cli::Options, scan};
use std::{
    collections::BTreeMap,
    ffi::{OsStr, OsString},
    mem::{size_of, zeroed},
    os::windows::{
        ffi::{OsStrExt, OsStringExt},
        process::CommandExt,
    },
    path::Path,
    process::{Command, Stdio},
    ptr::null,
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};
use windows_sys::Win32::{
    Foundation::*,
    System::{
        Console::*,
        Diagnostics::{Debug::*, ToolHelp::*},
        Memory::*,
        Threading::*,
    },
};

const PERIOD: Duration = Duration::from_millis(500);
const INIT_TIMEOUT: Duration = Duration::from_secs(60);
static STOP: AtomicBool = AtomicBool::new(false);

fn error(stage: &str) -> String {
    format!("{stage}: {}", std::io::Error::last_os_error())
}
fn wide(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(Some(0)).collect()
}

// Each handle has exactly one owner; dropping never terminates its process
struct Handle(HANDLE);
impl Handle {
    fn new(handle: HANDLE, stage: &str) -> Result<Self, String> {
        if handle.is_null() || handle == INVALID_HANDLE_VALUE {
            Err(error(stage))
        } else {
            Ok(Self(handle))
        }
    }
}
impl Drop for Handle {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}

unsafe extern "system" fn on_console(event: u32) -> i32 {
    match event {
        CTRL_C_EVENT | CTRL_BREAK_EVENT | CTRL_CLOSE_EVENT | CTRL_LOGOFF_EVENT
        | CTRL_SHUTDOWN_EVENT => {
            STOP.store(true, Ordering::SeqCst);
            1
        }
        _ => 0,
    }
}
struct ConsoleHandler;
impl ConsoleHandler {
    fn install() -> Result<Self, String> {
        STOP.store(false, Ordering::SeqCst);
        if unsafe { SetConsoleCtrlHandler(Some(on_console), 1) } == 0 {
            return Err(error("console handler"));
        }
        Ok(Self)
    }
}
impl Drop for ConsoleHandler {
    fn drop(&mut self) {
        unsafe {
            SetConsoleCtrlHandler(Some(on_console), 0);
        }
    }
}

fn live(process: &Handle) -> Result<bool, String> {
    match unsafe { WaitForSingleObject(process.0, 0) } {
        WAIT_TIMEOUT => Ok(true),
        WAIT_OBJECT_0 => Ok(false),
        _ => Err(error("process wait")),
    }
}
fn running(process: &Handle) -> Result<bool, String> {
    Ok(!STOP.load(Ordering::SeqCst) && live(process)?)
}
fn initializing(process: &Handle) -> Result<bool, String> {
    if STOP.load(Ordering::SeqCst) {
        return Ok(false);
    }
    if !live(process)? {
        let mut code = 0;
        if unsafe { GetExitCodeProcess(process.0, &mut code) } == 0 {
            return Err(error("game exit code"));
        }
        return Err(format!(
            "game exited before initialization completed (exit code {code})"
        ));
    }
    Ok(true)
}
fn wait(process: &Handle, duration: Duration) -> Result<bool, String> {
    let start = Instant::now();
    while start.elapsed() < duration {
        if !running(process)? {
            return Ok(false);
        }
        let remaining = duration.saturating_sub(start.elapsed());
        let ms = remaining.min(Duration::from_millis(50)).as_millis().max(1) as u32;
        match unsafe { WaitForSingleObject(process.0, ms) } {
            WAIT_OBJECT_0 => return Ok(false),
            WAIT_TIMEOUT => (),
            _ => return Err(error("process wait")),
        }
    }
    running(process)
}

fn existing_game(name: &OsStr) -> Result<bool, String> {
    let snapshot = Handle::new(
        unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) },
        "process snapshot",
    )?;
    let mut entry: PROCESSENTRY32W = unsafe { zeroed() };
    entry.dwSize = size_of::<PROCESSENTRY32W>() as u32;
    let mut result = unsafe { Process32FirstW(snapshot.0, &mut entry) };
    while result != 0 {
        let end = entry
            .szExeFile
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(entry.szExeFile.len());
        if OsString::from_wide(&entry.szExeFile[..end])
            .to_string_lossy()
            .eq_ignore_ascii_case(&name.to_string_lossy())
        {
            return Ok(true);
        }
        result = unsafe { Process32NextW(snapshot.0, &mut entry) };
    }
    if unsafe { GetLastError() } != ERROR_NO_MORE_FILES {
        return Err(error("enumerate processes"));
    }
    Ok(false)
}

fn base_module(pid: u32, expected: &Path) -> Result<Option<(usize, usize)>, String> {
    let raw = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, pid) };
    if raw == INVALID_HANDLE_VALUE && unsafe { GetLastError() } == ERROR_BAD_LENGTH {
        return Ok(None);
    }
    let snapshot = Handle::new(raw, "module snapshot")?;
    let mut entry: MODULEENTRY32W = unsafe { zeroed() };
    entry.dwSize = size_of::<MODULEENTRY32W>() as u32;
    if unsafe { Module32FirstW(snapshot.0, &mut entry) } == 0 {
        if unsafe { GetLastError() } == ERROR_NO_MORE_FILES {
            return Ok(None);
        }
        return Err(error("first module"));
    }
    let end = entry
        .szExePath
        .iter()
        .position(|&c| c == 0)
        .unwrap_or(entry.szExePath.len());
    let actual = std::fs::canonicalize(OsString::from_wide(&entry.szExePath[..end]))
        .map_err(|e| format!("module identity: {e}"))?;
    if !actual
        .as_os_str()
        .to_string_lossy()
        .eq_ignore_ascii_case(&expected.as_os_str().to_string_lossy())
    {
        return Err(format!("module identity mismatch: {actual:?}"));
    }
    Ok(Some((
        entry.modBaseAddr as usize,
        entry.modBaseSize as usize,
    )))
}

fn query(process: &Handle, address: usize) -> Result<MEMORY_BASIC_INFORMATION, String> {
    let mut info = unsafe { zeroed() };
    if unsafe {
        VirtualQueryEx(
            process.0,
            address as _,
            &mut info,
            size_of::<MEMORY_BASIC_INFORMATION>(),
        )
    } != size_of::<MEMORY_BASIC_INFORMATION>()
    {
        return Err(error("VirtualQueryEx"));
    }
    Ok(info)
}
fn readable(info: &MEMORY_BASIC_INFORMATION) -> bool {
    info.State == MEM_COMMIT
        && info.Protect & (PAGE_GUARD | PAGE_NOACCESS) == 0
        && matches!(
            info.Protect & 0xff,
            PAGE_READONLY
                | PAGE_READWRITE
                | PAGE_WRITECOPY
                | PAGE_EXECUTE_READ
                | PAGE_EXECUTE_READWRITE
                | PAGE_EXECUTE_WRITECOPY
        )
}
fn read(process: &Handle, address: usize, size: usize) -> Result<Vec<u8>, String> {
    if !running(process)? {
        return Err("read cancelled or game exited".into());
    }
    address.checked_add(size).ok_or("read address overflow")?;
    let mut bytes = vec![0; size];
    let mut count = 0;
    if unsafe {
        ReadProcessMemory(
            process.0,
            address as _,
            bytes.as_mut_ptr().cast(),
            size,
            &mut count,
        )
    } == 0
    {
        return Err(error(&format!(
            "ReadProcessMemory at {address:#x}, {count}/{size} bytes"
        )));
    }
    if count != size {
        return Err(format!("partial read at {address:#x}: {count}/{size}"));
    }
    Ok(bytes)
}

fn write(process: &Handle, address: usize, bytes: &[u8]) -> Result<(), String> {
    if !running(process)? {
        return Err("write cancelled or game exited".into());
    }
    address
        .checked_add(bytes.len())
        .ok_or("write address overflow")?;
    let mut count = 0;
    if unsafe {
        WriteProcessMemory(
            process.0,
            address as _,
            bytes.as_ptr().cast(),
            bytes.len(),
            &mut count,
        )
    } == 0
    {
        return Err(error(&format!(
            "WriteProcessMemory at {address:#x}, {count}/{} bytes",
            bytes.len()
        )));
    }
    if count != bytes.len() {
        return Err(format!("partial write: {count}/{}; stopping", bytes.len()));
    }
    Ok(())
}

struct Candidate {
    address: usize,
    instruction: usize,
    evidence: Vec<u8>,
}

fn validate_page(process: &Handle, base: usize, address: usize) -> Result<(), String> {
    let info = query(process, address)?;
    let end = address.checked_add(4).ok_or("FPS address overflow")?;
    let region_end = (info.BaseAddress as usize)
        .checked_add(info.RegionSize)
        .ok_or("region overflow")?;
    if !readable(&info)
        || end > region_end
        || address < info.BaseAddress as usize
        || info.AllocationBase as usize != base
        || info.Type != MEM_IMAGE
        || !matches!(info.Protect & 0xff, PAGE_READWRITE | PAGE_WRITECOPY)
    {
        return Err(format!(
            "FPS range is not writable non-executable image data at {address:#x}"
        ));
    }
    Ok(())
}

fn locate(
    process: &Handle,
    base: usize,
    module_size: usize,
    deadline: Instant,
) -> Result<Option<Candidate>, String> {
    let dos = read(process, base, 64)?;
    let prefix = read(process, base, scan::pe_coff_header_len(&dos)?)?;
    let headers = read(process, base, scan::pe_header_len(&prefix)?)?;
    let image = scan::parse_pe(&headers)?;
    if image.size as usize != module_size {
        return Err("PE/module image sizes disagree".into());
    }
    base.checked_add(module_size)
        .ok_or("module address overflow")?;
    let sections: Vec<_> = image
        .sections
        .iter()
        .filter(|s| s.name == *b".text\0\0\0" && s.characteristics & 0x60000000 == 0x60000000)
        .collect();
    if sections.len() != 1 {
        return Err("expected exactly one readable executable .text section".into());
    }
    let section = sections[0];
    let mut cursor = base
        .checked_add(section.rva as usize)
        .ok_or("section overflow")?;
    let end = cursor
        .checked_add(section.size as usize)
        .ok_or("section end overflow")?;
    let mut scanner = scan::PatternScanner::default();
    let mut candidates = BTreeMap::new();
    while cursor < end {
        if Instant::now() >= deadline {
            return Err("scan initialization deadline exceeded".into());
        }
        let info = query(process, cursor)?;
        // A hole prevents proving uniqueness; never scan uninitialized local bytes
        if !readable(&info) || info.AllocationBase as usize != base || info.Type != MEM_IMAGE {
            return Err(format!(
                "unreadable or replaced .text region at {cursor:#x}"
            ));
        }
        let region_end = (info.BaseAddress as usize)
            .checked_add(info.RegionSize)
            .ok_or("region overflow")?;
        let chunk_end = region_end.min(end).min(cursor.saturating_add(64 * 1024));
        if chunk_end <= cursor {
            return Err("non-progressing memory region".into());
        }
        let bytes = read(process, cursor, chunk_end - cursor)?;
        for instruction in scanner.feed(cursor, &bytes)? {
            let evidence = read(process, instruction, scan::PATTERN_LEN)?;
            let address = scan::fps_address(instruction, &evidence)?;
            let rva = address.checked_sub(base).ok_or("candidate before module")?;
            let candidate_end = rva.checked_add(4).ok_or("candidate overflow")?;
            if !image.sections.iter().any(|s| {
                s.characteristics & 0x80000000 != 0
                    && s.characteristics & 0x20000000 == 0
                    && rva >= s.rva as usize
                    && candidate_end <= s.rva as usize + s.size as usize
            }) {
                return Err(format!(
                    "pattern points outside writable image data: {address:#x}"
                ));
            }
            validate_page(process, base, address)?;
            println!(
                "Candidate: instruction={instruction:#x}, FPS={address:#x}, bytes={evidence:02x?}"
            );
            candidates.entry(address).or_insert(Candidate {
                address,
                instruction,
                evidence,
            });
            if candidates.len() > 1 {
                return Err("ambiguous FPS candidates; refusing writes".into());
            }
        }
        cursor = chunk_end;
    }
    Ok(candidates.into_values().next())
}

fn value(process: &Handle, candidate: &Candidate) -> Result<i32, String> {
    let bytes = read(process, candidate.address, 4)?;
    Ok(i32::from_le_bytes(
        bytes.try_into().map_err(|_| "invalid FPS read length")?,
    ))
}

fn update_fps(process: &Handle, candidate: &Candidate, target: i32) -> Result<bool, String> {
    let current = value(process, candidate)?;
    if !(1..=120).contains(&current) {
        return Err(format!("unverified FPS value {current}; stopping"));
    }
    if current == target || !running(process)? {
        return Ok(false);
    }
    write(process, candidate.address, &target.to_le_bytes())?;
    Ok(true)
}

pub fn run(options: Options) -> Result<(), String> {
    let _handler = ConsoleHandler::install()?;
    let result = run_controller(options);
    if STOP.load(Ordering::SeqCst) {
        println!("STOPPED: user requested exit; no FPS value restored");
        Ok(())
    } else {
        result
    }
}

fn run_controller(options: Options) -> Result<(), String> {
    let name = wide(OsStr::new("Local\\genshin-uncap-v1-controller"));
    let mutex = unsafe { CreateMutexW(null(), 0, name.as_ptr()) };
    let mutex_error = unsafe { GetLastError() };
    let _mutex = Handle::new(mutex, "controller mutex")?;
    if mutex_error == ERROR_ALREADY_EXISTS {
        return Err("another controller is running in this session".into());
    }
    let game = std::fs::canonicalize(&options.game).map_err(|e| format!("game path: {e}"))?;
    if !game.is_file() {
        return Err("game path is not a file".into());
    }
    let name = game.file_name().ok_or("game path has no filename")?;
    if existing_game(name)? {
        return Err("game is already running; close it before starting this tool".into());
    }
    if STOP.load(Ordering::SeqCst) {
        println!("Cancelled before launch");
        return Ok(());
    }
    println!(
        "Launching: {game:?}; target={} FPS; probe={}",
        options.fps, options.probe
    );
    // The child must not share our console; closing this window must leave it alive
    let child = Command::new(&game)
        .args(&options.args)
        .current_dir(game.parent().ok_or("game directory missing")?)
        .creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("launch: {e}"))?;
    let rights = PROCESS_QUERY_INFORMATION
        | PROCESS_VM_READ
        | PROCESS_SYNCHRONIZE
        | if options.probe {
            0
        } else {
            PROCESS_VM_WRITE | PROCESS_VM_OPERATION
        };
    let process = Handle::new(
        unsafe { OpenProcess(rights, 0, child.id()) },
        "open child process",
    )?;
    let pid = child.id();
    drop(child);
    println!("Launched Win32 PID={pid}; waiting for module");
    let deadline = Instant::now() + INIT_TIMEOUT;
    let (base, size) = loop {
        if !initializing(&process)? {
            println!("Cancelled during initialization");
            return Ok(());
        }
        if let Some(module) = base_module(pid, &game)? {
            break module;
        }
        if Instant::now() >= deadline {
            return Err("module initialization timeout".into());
        }
        wait(&process, Duration::from_millis(100))?;
    };
    println!("Module: base={base:#x}, image_size={size:#x}");
    let mut pending_logged = false;
    let candidate = loop {
        if !initializing(&process)? {
            println!("Cancelled during initialization");
            return Ok(());
        }
        if let Some(candidate) = locate(&process, base, size, deadline)? {
            break candidate;
        }
        if Instant::now() >= deadline {
            return Err("locate timeout: no FPS candidate; unsupported game build".into());
        }
        if !pending_logged {
            println!("Waiting for FPS signature in initialized image");
            pending_logged = true;
        }
        wait(&process, Duration::from_secs(1))?;
    };
    let mut last = None;
    let mut valid_since = None;
    let initial = loop {
        if !initializing(&process)? {
            return Ok(());
        }
        validate_page(&process, base, candidate.address)?;
        if read(&process, candidate.instruction, candidate.evidence.len())? != candidate.evidence {
            return Err("locator instruction changed during initialization".into());
        }
        let observed = value(&process, &candidate)?;
        if last != Some(observed) {
            println!("Read-only initialization: observed FPS value={observed}");
            valid_since = if (1..=120).contains(&observed) {
                Some(Instant::now())
            } else {
                None
            };
            last = Some(observed);
        }
        if valid_since.is_some_and(|since| since.elapsed() >= PERIOD) {
            break observed;
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "FPS initialization timeout: unverified or unstable value {observed}; zero writes"
            ));
        }
        wait(&process, Duration::from_millis(100))?;
    };
    println!(
        "Located: FPS address={:#x}, observed={initial}; candidate is not a measured frame rate",
        candidate.address
    );
    if options.probe {
        println!("PROBE COMPLETE: zero writes; game left running");
        return Ok(());
    }
    println!(
        "CHECKING: target {} FPS every {} ms; writing only when different",
        options.fps,
        PERIOD.as_millis()
    );
    let mut wrote = false;
    let outcome = (|| -> Result<(), String> {
        while running(&process)? {
            validate_page(&process, base, candidate.address)?;
            if read(&process, candidate.instruction, candidate.evidence.len())?
                != candidate.evidence
            {
                return Err("locator instruction changed; stopping".into());
            }
            if update_fps(&process, &candidate, options.fps)? && !wrote {
                println!(
                    "WRITING: set {} FPS; actual frame rate requires measurement",
                    options.fps
                );
                wrote = true;
            }
            if !wait(&process, PERIOD)? {
                break;
            }
        }
        Ok(())
    })();
    // A process exit during a Win32 operation is a normal runtime stop, not a write failure
    if let Err(failure) = outcome {
        if live(&process)? {
            return Err(failure);
        }
    }
    println!("STOPPED: no further writes; no FPS value restored");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matching_fps_needs_no_write_access_but_mismatch_does() {
        let process = Handle::new(
            unsafe {
                OpenProcess(
                    PROCESS_VM_READ | PROCESS_SYNCHRONIZE,
                    0,
                    GetCurrentProcessId(),
                )
            },
            "read-only self process",
        )
        .unwrap();
        let fps = std::sync::atomic::AtomicI32::new(120);
        let candidate = Candidate {
            address: fps.as_ptr() as usize,
            instruction: 0,
            evidence: vec![],
        };
        // No write permission: success proves matching values skip WriteProcessMemory
        assert!(write(&process, candidate.address, &120_i32.to_le_bytes()).is_err());
        assert!(!update_fps(&process, &candidate, 120).unwrap());
        fps.store(60, Ordering::SeqCst);
        assert!(
            update_fps(&process, &candidate, 120)
                .unwrap_err()
                .contains("WriteProcessMemory")
        );
        assert_eq!(fps.load(Ordering::SeqCst), 60);
        let writable = Handle::new(
            unsafe {
                OpenProcess(
                    PROCESS_VM_READ | PROCESS_VM_WRITE | PROCESS_VM_OPERATION | PROCESS_SYNCHRONIZE,
                    0,
                    GetCurrentProcessId(),
                )
            },
            "writable self process",
        )
        .unwrap();
        assert!(update_fps(&writable, &candidate, 120).unwrap());
        assert_eq!(fps.load(Ordering::SeqCst), 120);
        assert!(update_fps(&writable, &candidate, 60).unwrap());
        assert_eq!(fps.load(Ordering::SeqCst), 60);
        assert!(!update_fps(&process, &candidate, 60).unwrap());
        for invalid in [-1, 0, 121] {
            fps.store(invalid, Ordering::SeqCst);
            assert!(
                update_fps(&writable, &candidate, 120)
                    .unwrap_err()
                    .contains("unverified FPS value")
            );
            assert_eq!(fps.load(Ordering::SeqCst), invalid);
        }
        let unreadable = Candidate {
            address: 0,
            ..candidate
        };
        assert!(
            update_fps(&writable, &unreadable, 120)
                .unwrap_err()
                .contains("ReadProcessMemory")
        );
    }

    #[test]
    fn inaccessible_ranges_are_errors_not_complete_reads_or_writes() {
        let process = Handle::new(
            unsafe {
                OpenProcess(
                    PROCESS_QUERY_INFORMATION
                        | PROCESS_VM_READ
                        | PROCESS_VM_WRITE
                        | PROCESS_VM_OPERATION
                        | PROCESS_SYNCHRONIZE,
                    0,
                    GetCurrentProcessId(),
                )
            },
            "self process",
        )
        .unwrap();
        let allocation =
            unsafe { VirtualAlloc(null(), 8192, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE) };
        assert!(!allocation.is_null());
        struct Allocation(*mut std::ffi::c_void);
        impl Drop for Allocation {
            fn drop(&mut self) {
                unsafe {
                    VirtualFree(self.0, 0, MEM_RELEASE);
                }
            }
        }
        let _allocation = Allocation(allocation);
        let address = allocation as usize;
        write(&process, address, &120_i32.to_le_bytes()).unwrap();
        assert_eq!(read(&process, address, 4).unwrap(), 120_i32.to_le_bytes());
        // Writable private memory is still not an accepted game image candidate
        assert!(validate_page(&process, address, address).is_err());
        let mut old = 0;
        assert_ne!(
            unsafe { VirtualProtect((address + 4096) as _, 4096, PAGE_NOACCESS, &mut old) },
            0
        );
        assert!(read(&process, address + 4094, 4).is_err());
        assert!(write(&process, address + 4094, &120_i32.to_le_bytes()).is_err());
        assert!(read(&process, usize::MAX, 4).is_err());
        assert!(write(&process, usize::MAX, &[1, 2]).is_err());
        assert!(!readable(&query(&process, address + 4096).unwrap()));
    }
}
