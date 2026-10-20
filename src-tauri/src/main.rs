use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use std::{
    collections::HashMap,
    io::{Read, Write},
    process::Command,
    sync::Mutex,
};
use tauri::{AppHandle, Emitter, State};

struct Pty {
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    child: Box<dyn portable_pty::Child + Send + Sync>,
}

#[derive(Default)]
struct Ptys(Mutex<HashMap<String, Pty>>);

fn workspace() -> String {
    std::env::var("TAKARA_WORKSPACE").unwrap_or_else(|_| {
        format!("{}/Code/takara-space", std::env::var("HOME").unwrap_or_default())
    })
}

fn shell() -> String {
    std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".into())
}

#[tauri::command]
fn get_workspace() -> String {
    workspace()
}

#[tauri::command]
fn git_branch() -> Option<String> {
    let out = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(workspace())
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

#[tauri::command]
fn version(cmd: String) -> Option<String> {
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
) -> Result<u32, String> {
    let pair = native_pty_system()
        .openpty(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 })
        .map_err(|e| e.to_string())?;
    let sh = shell();
    let mut b = CommandBuilder::new(&sh);
    if cmd.is_empty() {
        b.arg("-l");
    } else {
        b.args(["-lic", &format!("{cmd}; exec {sh} -l")]);
    }
    b.cwd(workspace());
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

    state.0.lock().unwrap().insert(id, Pty { master: pair.master, writer, child });
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
    tauri::Builder::default()
        .manage(Ptys::default())
        .invoke_handler(tauri::generate_handler![
            get_workspace, git_branch, version, pty_spawn, pty_write, pty_resize, pty_kill
        ])
        .run(tauri::generate_context!())
        .expect("error running Takara Space");
}
