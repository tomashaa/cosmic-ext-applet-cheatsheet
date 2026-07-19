// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Tomas Haaland
//
// COSMIC panel applet: a searchable keyboard-shortcut cheat sheet.
// Native libcosmic port of the GTK `cosmic-cheatsheet` from clip-suite.

mod config;
mod data;
mod i18n;
mod window;

use window::Window;

fn pid_path() -> std::path::PathBuf {
    let dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    std::path::Path::new(&dir).join("cosmic-ext-cheatsheet.pid")
}

/// The PID in the file, but only if that process is still alive.
fn read_live_pid(p: &std::path::Path) -> Option<u32> {
    let pid: u32 = std::fs::read_to_string(p).ok()?.trim().parse().ok()?;
    std::path::Path::new(&format!("/proc/{pid}"))
        .exists()
        .then_some(pid)
}

/// Open the cheat sheet as a standalone window; a second invocation while one
/// is already open closes it instead (toggle), so a keybind toggles it.
fn run_window() -> cosmic::iced::Result {
    let pidf = pid_path();
    if let Some(pid) = read_live_pid(&pidf) {
        let _ = std::process::Command::new("kill")
            .arg(pid.to_string())
            .status();
        let _ = std::fs::remove_file(&pidf);
        return Ok(());
    }
    let _ = std::fs::write(&pidf, std::process::id().to_string());
    // No main window: the cheat sheet lives in a top-anchored layer surface
    // created in Window::init (see window.rs).
    let settings = cosmic::app::Settings::default().no_main_window(true);
    let result = cosmic::app::run::<Window>(settings, true);
    let _ = std::fs::remove_file(&pidf);
    result
}

fn main() -> cosmic::iced::Result {
    let env = env_logger::Env::default().filter_or("RUST_LOG", "warn");
    env_logger::init_from_env(env);

    // `--window`: open the cheat sheet as a standalone window (for a keybind
    // like Super+C). Otherwise run as a COSMIC panel applet.
    if std::env::args().any(|a| a == "--window") {
        run_window()
    } else {
        cosmic::applet::run::<Window>(false)
    }
}
