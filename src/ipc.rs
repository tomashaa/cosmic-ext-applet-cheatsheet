// SPDX-License-Identifier: GPL-3.0-only
// Copyright (c) 2026 Tomas Haaland
//! IPC + single-open guard for Super+C / panel toggle.
//!
//! COSMIC may run several applet instances (one per panel/output). A single
//! "toggle" file races: an instance *without* the sheet opens a second one.
//!
//! Protocol:
//! - `*.open` marker — PID of the process that currently owns the sheet
//! - `*.request.open` — one-shot; first applet to claim opens (with marker)
//! - `*.request.close` — sticky; every applet with an open sheet closes

use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

fn runtime_dir() -> PathBuf {
    std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/tmp"))
}

fn applet_pid_path() -> PathBuf {
    runtime_dir().join("cosmic-ext-cheatsheet-applet.pid")
}

fn open_request_path() -> PathBuf {
    runtime_dir().join("cosmic-ext-cheatsheet.request.open")
}

fn close_request_path() -> PathBuf {
    runtime_dir().join("cosmic-ext-cheatsheet.request.close")
}

fn open_marker_path() -> PathBuf {
    runtime_dir().join("cosmic-ext-cheatsheet.open")
}

fn pid_alive(pid: u32) -> bool {
    Path::new(&format!("/proc/{pid}")).exists()
}

fn cmdline(pid: u32) -> Option<String> {
    let raw = std::fs::read(format!("/proc/{pid}/cmdline")).ok()?;
    Some(String::from_utf8_lossy(&raw).replace('\0', " "))
}

fn is_our_binary(pid: u32) -> bool {
    cmdline(pid)
        .map(|c| c.contains("cosmic-ext-applet-cheatsheet"))
        .unwrap_or(false)
}

fn is_our_applet(pid: u32) -> bool {
    let Some(cmd) = cmdline(pid) else {
        return false;
    };
    cmd.contains("cosmic-ext-applet-cheatsheet") && !cmd.contains("--window")
}

fn is_our_window(pid: u32) -> bool {
    let Some(cmd) = cmdline(pid) else {
        return false;
    };
    cmd.contains("cosmic-ext-applet-cheatsheet") && cmd.contains("--window")
}

/// Record that a panel applet exists (best-effort discovery for Super+C).
pub fn register_applet() {
    let _ = std::fs::write(applet_pid_path(), std::process::id().to_string());
}

#[allow(dead_code)]
pub fn unregister_applet() {
    let path = applet_pid_path();
    if let Ok(text) = std::fs::read_to_string(&path) {
        if text.trim().parse::<u32>().ok() == Some(std::process::id()) {
            let _ = std::fs::remove_file(&path);
        }
    }
}

fn applet_is_registered() -> bool {
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
    true
}

/// Ask a live panel applet to open the sheet (one-shot file).
pub fn request_applet_open() -> bool {
    if !applet_is_registered() {
        return false;
    }
    match std::fs::write(open_request_path(), b"1\n") {
        Ok(()) => true,
        Err(e) => {
            log::warn!("failed to write open request: {e}");
            false
        }
    }
}

/// Ask *all* panel applets to close — sticky until cleared.
pub fn request_applet_close() -> bool {
    if !applet_is_registered() {
        // Still write the flag: applets may exist even if pid file is stale.
        let _ = std::fs::write(close_request_path(), b"1\n");
        return close_request_path().exists();
    }
    match std::fs::write(close_request_path(), b"1\n") {
        Ok(()) => true,
        Err(e) => {
            log::warn!("failed to write close request: {e}");
            false
        }
    }
}

pub fn clear_close_request() {
    let _ = std::fs::remove_file(close_request_path());
}

pub fn close_requested() -> bool {
    close_request_path().exists()
}

/// Consume a one-shot open request (only one instance should win).
pub fn take_open_request() -> bool {
    let path = open_request_path();
    if path.exists() {
        let _ = std::fs::remove_file(&path);
        true
    } else {
        false
    }
}

/// True when a live process owns the global sheet marker.
pub fn open_marker_alive() -> bool {
    match open_owner() {
        Some(pid) => pid_alive(pid) && is_our_binary(pid),
        None => false,
    }
}

pub fn open_owner() -> Option<u32> {
    let text = std::fs::read_to_string(open_marker_path()).ok()?;
    let pid = text.trim().parse().ok()?;
    if pid_alive(pid) && is_our_binary(pid) {
        Some(pid)
    } else {
        let _ = std::fs::remove_file(open_marker_path());
        None
    }
}

/// Claim exclusive ownership of the sheet. Fails if another live owner exists.
pub fn claim_open_marker() -> bool {
    if let Some(pid) = open_owner() {
        return pid == std::process::id();
    }
    let path = open_marker_path();
    let _ = std::fs::remove_file(&path);
    match OpenOptions::new().write(true).create_new(true).open(&path) {
        Ok(mut f) => {
            let _ = write!(f, "{}", std::process::id());
            true
        }
        Err(_) => open_owner() == Some(std::process::id()),
    }
}

/// Drop the marker if we own it (or it's stale).
pub fn release_open_marker() {
    let path = open_marker_path();
    match std::fs::read_to_string(&path) {
        Ok(text) => {
            if text.trim().parse::<u32>().ok() == Some(std::process::id()) {
                let _ = std::fs::remove_file(&path);
            } else if open_owner().is_none() {
                let _ = std::fs::remove_file(&path);
            }
        }
        Err(_) => {}
    }
}

/// Kill any live standalone `--window` instances. Returns true if any were signaled.
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

/// True if any standalone `--window` sheet process is running.
pub fn any_windowed_instances() -> bool {
    let Ok(dir) = std::fs::read_dir("/proc") else {
        return false;
    };
    let self_pid = std::process::id();
    dir.flatten().any(|ent| {
        ent.file_name()
            .to_string_lossy()
            .parse::<u32>()
            .ok()
            .filter(|pid| *pid != self_pid)
            .is_some_and(is_our_window)
    })
}

/// Super+C "is anything showing?" — marker or windowed process.
pub fn anything_open() -> bool {
    open_marker_alive() || any_windowed_instances()
}

/// Close every visible sheet: kill `--window` processes and signal applets.
/// Blocks briefly so applet polls can observe the sticky close flag.
pub fn close_everything() {
    let _ = kill_windowed_instances();
    let _ = request_applet_close();
    // Applets poll every ~150ms; wait long enough for begin_close + marker release.
    for _ in 0..20 {
        if !open_marker_alive() && !any_windowed_instances() {
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    clear_close_request();
    if !open_marker_alive() {
        let _ = std::fs::remove_file(open_marker_path());
    }
}
