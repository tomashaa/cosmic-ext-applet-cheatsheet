// SPDX-License-Identifier: GPL-3.0-only
// Copyright (c) 2026 Tomas Haaland
//! Lightweight IPC so Super+C / `--window` can toggle a running panel applet
//! instead of spawning a second process.

use std::path::{Path, PathBuf};

fn runtime_dir() -> PathBuf {
    std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/tmp"))
}

fn applet_pid_path() -> PathBuf {
    runtime_dir().join("cosmic-ext-cheatsheet-applet.pid")
}

fn toggle_path() -> PathBuf {
    runtime_dir().join("cosmic-ext-cheatsheet.toggle")
}

fn pid_alive(pid: u32) -> bool {
    Path::new(&format!("/proc/{pid}")).exists()
}

fn cmdline(pid: u32) -> Option<String> {
    let raw = std::fs::read(format!("/proc/{pid}/cmdline")).ok()?;
    Some(String::from_utf8_lossy(&raw).replace('\0', " "))
}

fn is_our_applet(pid: u32) -> bool {
    let Some(cmd) = cmdline(pid) else {
        return false;
    };
    let name = cmd.contains("cosmic-ext-applet-cheatsheet");
    // Panel applet is launched without `--window`.
    name && !cmd.contains("--window")
}

fn is_our_window(pid: u32) -> bool {
    let Some(cmd) = cmdline(pid) else {
        return false;
    };
    cmd.contains("cosmic-ext-applet-cheatsheet") && cmd.contains("--window")
}

/// Record the panel-applet PID so `--window` can signal it.
pub fn register_applet() {
    let _ = std::fs::write(applet_pid_path(), std::process::id().to_string());
}

pub fn unregister_applet() {
    let path = applet_pid_path();
    if let Ok(text) = std::fs::read_to_string(&path) {
        if text.trim().parse::<u32>().ok() == Some(std::process::id()) {
            let _ = std::fs::remove_file(&path);
        }
    }
}

/// If a live panel applet is registered, ask it to toggle and return `true`.
pub fn request_applet_toggle() -> bool {
    let path = applet_pid_path();
    let Ok(text) = std::fs::read_to_string(&path) else {
        return false;
    };
    let Ok(pid) = text.trim().parse::<u32>() else {
        let _ = std::fs::remove_file(&path);
        return false;
    };
    if !pid_alive(pid) || !is_our_applet(pid) {
        let _ = std::fs::remove_file(&path);
        return false;
    }
    if let Err(e) = std::fs::write(toggle_path(), b"1") {
        log::warn!("failed to write cheatsheet toggle: {e}");
        return false;
    }
    true
}

/// Applet-side: consume a pending toggle request (if any).
pub fn take_toggle_request() -> bool {
    let path = toggle_path();
    if path.exists() {
        let _ = std::fs::remove_file(&path);
        true
    } else {
        false
    }
}

/// Kill any live standalone `--window` instances (for reliable Super+C toggle).
pub fn kill_windowed_instances() -> bool {
    let mut killed = false;
    let Ok(dir) = std::fs::read_dir("/proc") else {
        return false;
    };
    let self_pid = std::process::id();
    for ent in dir.flatten() {
        let Ok(pid) = ent.file_name().to_string_lossy().parse::<u32>() else {
            continue;
        };
        if pid == self_pid {
            continue;
        }
        if is_our_window(pid) {
            let _ = std::process::Command::new("kill")
                .arg(pid.to_string())
                .status();
            killed = true;
        }
    }
    killed
}
