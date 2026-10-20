use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use serde::Serialize;
use std::{
    collections::HashMap,
    fs,
    io::{Read, Write},
    path::Path,
    process::Command,
    sync::Mutex,
    time::UNIX_EPOCH,
};
use tauri::{AppHandle, Emitter, State};

struct Pty {
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    child: Box<dyn portable_pty::Child + Send + Sync>,
}

#[derive(Default)]
struct Ptys(Mutex<HashMap<String, Pty>>);

fn expand_path(p: &str) -> String {
    let trimmed = p.trim();
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| if cfg!(windows) { "C:\\".into() } else { "/".into() });
    if trimmed.is_empty() || trimmed == "~" {
        return home;
    }
    if let Some(rest) = trimmed.strip_prefix("~/") {
        return format!("{home}/{rest}");
    }
    if let Some(rest) = trimmed.strip_prefix("~\\") {
        return format!("{home}\\{rest}");
    }
    trimmed.to_string()
}

#[cfg(target_os = "macos")]
fn fix_macos_path() {
    if let Ok(current_path) = std::env::var("PATH") {
        let home = std::env::var("HOME").unwrap_or_default();
        let extra_paths = [
            "/opt/homebrew/bin".to_string(),
            "/usr/local/bin".to_string(),
            format!("{home}/.local/bin"),
            format!("{home}/.kimi-code/bin"),
            format!("{home}/.cargo/bin"),
        ];
        let mut new_path = current_path;
        for p in extra_paths {
            if !new_path.split(':').any(|x| x == p) {
                new_path = format!("{p}:{new_path}");
            }
        }
        std::env::set_var("PATH", new_path);
    }
}

#[cfg(not(target_os = "macos"))]
fn fix_macos_path() {}

fn shell() -> String {
    if cfg!(target_os = "windows") {
        std::env::var("COMSPEC").unwrap_or_else(|_| "powershell.exe".into())
    } else {
        std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".into())
    }
}

#[tauri::command]
fn get_workspace() -> String {
    "~".to_string()
}

#[tauri::command]
fn verify_path(path: String) -> bool {
    let expanded = expand_path(&path);
    std::path::Path::new(&expanded).is_dir()
}

#[derive(Serialize)]
struct FileItem {
    name: String,
    path: String,
    is_dir: bool,
    is_symlink: bool,
    size: u64,
    modified: u64,
    ext: String,
}

#[derive(Serialize)]
struct DirContent {
    current_path: String,
    parent_path: Option<String>,
    entries: Vec<FileItem>,
}

#[tauri::command]
fn read_dir(path: Option<String>) -> Result<DirContent, String> {
    let target = expand_path(path.as_deref().unwrap_or("~"));
    let p = Path::new(&target);
    if !p.exists() {
        return Err(format!("Direktori tidak ditemukan: {target}"));
    }
    if !p.is_dir() {
        return Err(format!("Bukan direktori: {target}"));
    }

    let parent_path = p.parent().map(|par| par.to_string_lossy().to_string());

    let mut entries = Vec::new();
    let read = fs::read_dir(p).map_err(|e| e.to_string())?;

    for entry in read.flatten() {
        let file_name = entry.file_name().to_string_lossy().to_string();
        let full_path = entry.path().to_string_lossy().to_string();
        let ft = entry.file_type().ok();
        let is_dir = ft.as_ref().map(|f| f.is_dir()).unwrap_or(false);
        let is_symlink = ft.as_ref().map(|f| f.is_symlink()).unwrap_or(false);
        let meta = entry.metadata().ok();
        let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
        let modified = meta
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let ext = entry
            .path()
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        entries.push(FileItem {
            name: file_name,
            path: full_path,
            is_dir,
            is_symlink,
            size,
            modified,
            ext,
        });
    }

    entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    Ok(DirContent {
        current_path: target,
        parent_path,
        entries,
    })
}

#[tauri::command]
fn read_file_preview(path: String) -> Result<String, String> {
    let target = expand_path(&path);
    let p = Path::new(&target);
    if !p.exists() || p.is_dir() {
        return Err("File tidak ditemukan atau berupa direktori".into());
    }
    let mut file = fs::File::open(p).map_err(|e| e.to_string())?;
    let mut buffer = [0u8; 49152];
    let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
    Ok(String::from_utf8_lossy(&buffer[..n]).to_string())
}

#[tauri::command]
fn reveal_in_finder(path: String) -> Result<(), String> {
    let target = expand_path(&path);
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .args(["-R", &target])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .args(["/select,", &target])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    {
        let p = Path::new(&target);
        let dir = if p.is_dir() { p } else { p.parent().unwrap_or(p) };
        Command::new("xdg-open")
            .arg(dir)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn git_branch(path: Option<String>) -> Option<String> {
    let target = expand_path(path.as_deref().unwrap_or("~"));
    let out = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(&target)
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

#[tauri::command]
fn version(cmd: String) -> Option<String> {
    #[cfg(target_os = "windows")]
    let out = Command::new("cmd")
        .args(["/C", &format!("{cmd} --version")])
        .output()
        .ok()?;
    #[cfg(not(target_os = "windows"))]
    let out = Command::new(shell())
        .args(["-lic", &format!("{cmd} --version")])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    text.split_whitespace()
        .find(|w| {
            let mut p = w.trim_start_matches('v').split('.');
            (0..3).all(|_| p.next().is_some_and(|x| !x.is_empty() && x.chars().all(|c| c.is_ascii_digit())))
        })
        .map(|w| w.trim_start_matches('v').to_string())
}

#[tauri::command]
fn pty_spawn(
    app: AppHandle,
    state: State<Ptys>,
    id: String,
    cmd: String,
    cols: u16,
    rows: u16,
    cwd: Option<String>,
) -> Result<u32, String> {
    let pair = native_pty_system()
        .openpty(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 })
        .map_err(|e| e.to_string())?;
    let sh = shell();
    let mut b = CommandBuilder::new(&sh);
    #[cfg(target_os = "windows")]
    {
        if !cmd.is_empty() {
            if sh.to_lowercase().contains("powershell") {
                b.args(["-NoExit", "-Command", &cmd]);
            } else {
                b.args(["/K", &cmd]);
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        if cmd.is_empty() {
            b.arg("-l");
        } else {
            b.args(["-lic", &format!("{cmd}; exec {sh} -l")]);
        }
    }
    let target_dir = expand_path(cwd.as_deref().unwrap_or("~"));
    b.cwd(&target_dir);
    b.env("TERM", "xterm-256color");
    let child = pair.slave.spawn_command(b).map_err(|e| e.to_string())?;
    let pid = child.process_id().unwrap_or(0);
    let mut reader = pair.master.try_clone_reader().map_err(|e| e.to_string())?;
    let writer = pair.master.take_writer().map_err(|e| e.to_string())?;

    let eid = id.clone();
    std::thread::spawn(move || {
        let mut buf = [0u8; 8192];
        let mut pending: Vec<u8> = Vec::new();
        loop {
            match reader.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    pending.extend_from_slice(&buf[..n]);
                    // jangan potong karakter UTF-8 di tengah
                    let valid = match std::str::from_utf8(&pending) {
                        Ok(s) => s.len(),
                        Err(e) => e.valid_up_to(),
                    };
                    if valid > 0 {
                        let s = String::from_utf8_lossy(&pending[..valid]).into_owned();
                        let _ = app.emit("pty:data", (eid.clone(), s));
                        pending.drain(..valid);
                    }
                }
            }
        }
        let _ = app.emit("pty:exit", (eid, 0i32));
    });

    {
        let mut map = state.0.lock().unwrap();
        if let Some(mut old) = map.remove(&id) {
            let _ = old.child.kill();
        }
        map.insert(id, Pty { master: pair.master, writer, child });
    }
    Ok(pid)
}

#[tauri::command]
fn pty_write(state: State<Ptys>, id: String, data: String) {
    if let Some(p) = state.0.lock().unwrap().get_mut(&id) {
        let _ = p.writer.write_all(data.as_bytes());
    }
}

#[tauri::command]
fn pty_resize(state: State<Ptys>, id: String, cols: u16, rows: u16) {
    if let Some(p) = state.0.lock().unwrap().get(&id) {
        let _ = p.master.resize(PtySize {
            rows: rows.max(2),
            cols: cols.max(2),
            pixel_width: 0,
            pixel_height: 0,
        });
    }
}

#[tauri::command]
fn pty_kill(state: State<Ptys>, id: String) {
    if let Some(mut p) = state.0.lock().unwrap().remove(&id) {
        let _ = p.child.kill();
    }
}

fn main() {
    fix_macos_path();
    tauri::Builder::default()
        .manage(Ptys::default())
        .invoke_handler(tauri::generate_handler![
            get_workspace, git_branch, version, pty_spawn, pty_write, pty_resize, pty_kill, verify_path,
            read_dir, read_file_preview, reveal_in_finder
        ])
        .run(tauri::generate_context!())
        .expect("error running VibeDeck");
}
