// SPDX-License-Identifier: GPL-3.0-only
// Copyright (c) 2026 Tomas Haaland
//! IPC + single-open guard for Super+C / panel toggle.
//!
//! COSMIC may run several applet instances (one per panel/output). Protocol:
//! - `*.open` marker — PID that currently owns the sheet
//! - `*.request.open` — one-shot; first applet to claim opens
//! - `*.request.close` — sticky while closing; every instance with a sheet closes

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

fn cmdline_args(pid: u32) -> Option<Vec<String>> {
    let raw = std::fs::read(format!("/proc/{pid}/cmdline")).ok()?;
    let args: Vec<String> = raw
        .split(|b| *b == 0)
        .filter(|a| !a.is_empty())
        .map(|a| String::from_utf8_lossy(a).into_owned())
        .collect();
    if args.is_empty() {
        None
    } else {
        Some(args)
    }
}

/// True only when `/proc/pid/exe` is our binary — never match shells whose
/// cmdline merely *mentions* `cosmic-ext-applet-cheatsheet --window` (Cursor
/// agent wrappers previously made Super+C stuck in permanent close mode).
fn exe_is_ours(pid: u32) -> bool {
    let Ok(link) = std::fs::read_link(format!("/proc/{pid}/exe")) else {
        return false;
    };
    let name = link
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    // Deleted binaries show as "name (deleted)".
    name.starts_with("cosmic-ext-applet-cheatsheet")
}

fn is_our_binary(pid: u32) -> bool {
    exe_is_ours(pid)
}

fn is_our_applet(pid: u32) -> bool {
    exe_is_ours(pid)
        && cmdline_args(pid).is_some_and(|args| args.iter().all(|a| a != "--window"))
}

fn is_our_window(pid: u32) -> bool {
    exe_is_ours(pid)
        && cmdline_args(pid).is_some_and(|args| args.iter().any(|a| a == "--window"))
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
    // Never let a leftover close flag kill the sheet we are about to open.
    clear_close_request();
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
    // Drop any pending open so we don't reopen while closing.
    let _ = std::fs::remove_file(open_request_path());
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

pub fn open_marker_alive() -> bool {
    open_owner().is_some()
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

pub fn anything_open() -> bool {
    open_marker_alive() || any_windowed_instances()
}

/// Non-blocking close signal for use on the UI thread (panel button).
pub fn signal_close() {
    let _ = kill_windowed_instances();
    let _ = request_applet_close();
}

/// Blocking close for the Super+C helper process (not the applet UI thread).
pub fn close_everything() {
    signal_close();
    for _ in 0..20 {
        if !open_marker_alive() && !any_windowed_instances() {
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    clear_close_request();
    let _ = std::fs::remove_file(open_request_path());
    // If an applet ignored close or the marker is stale, don't stay stuck in
    // permanent "close-only" mode — drop the marker so the next press can open.
    if open_marker_alive() {
        log::warn!("cheatsheet open marker still present after close; forcing clear");
        let _ = std::fs::remove_file(open_marker_path());
    }
}

/// Wait until an applet claims the open marker (or timeout).
pub fn wait_until_open(timeout: Duration) -> bool {
    let steps = (timeout.as_millis() / 50).max(1) as usize;
    for _ in 0..steps {
        if open_marker_alive() {
            return true;
        }
        // A stale close flag must not cancel a fresh open.
        clear_close_request();
        std::thread::sleep(Duration::from_millis(50));
    }
    false
}
