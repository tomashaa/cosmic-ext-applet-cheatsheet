// SPDX-License-Identifier: GPL-3.0-only
// Copyright (c) 2026 Tomas Haaland
//
// COSMIC panel applet: a searchable keyboard-shortcut cheat sheet.
// Native libcosmic port of the GTK `cosmic-cheatsheet` from clip-suite.

mod config;
mod i18n;
mod ipc;
mod shortcuts;
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
///
/// Prefer toggling a live panel applet via IPC when one is registered.
fn run_window() -> cosmic::iced::Result {
    // 1) Panel applet running → ask it to open/close in-process.
    if ipc::request_applet_toggle() {
        return Ok(());
    }

    // 2) Any orphaned `--window` sheets → kill them (acts as "close").
    if ipc::kill_windowed_instances() {
        let _ = std::fs::remove_file(pid_path());
        return Ok(());
    }

    // 3) PID-file toggle (legacy / single-instance).
    let pidf = pid_path();
    if let Some(pid) = read_live_pid(&pidf) {
        let _ = std::process::Command::new("kill")
            .arg(pid.to_string())
            .status();
        let _ = std::fs::remove_file(&pidf);
        return Ok(());
    }

    let _ = std::fs::write(&pidf, std::process::id().to_string());
    let settings = cosmic::app::Settings::default().no_main_window(true);
    let result = cosmic::app::run::<Window>(settings, true);
    let _ = std::fs::remove_file(&pidf);
    result
}

fn main() -> cosmic::iced::Result {
    let env = env_logger::Env::default().filter_or("RUST_LOG", "warn");
    env_logger::init_from_env(env);

    if std::env::args().any(|a| a == "--dump-shortcuts") {
        let settings = config::load_settings();
        i18n::init(settings.lang.as_deref());
        let compact = !std::env::args().any(|a| a == "--all");
        let full = shortcuts::load();
        let list = shortcuts::for_display(&full, compact);
        println!(
            "# {} rows ({})",
            list.len(),
            if compact { "compact" } else { "all" }
        );
        for s in list {
            let icon = s.icon.unwrap_or("-");
            let cmd = s.command.as_ref().map(|c| c.join(" ")).unwrap_or_default();
            println!(
                "[{}] {:40} {:28} {:32} {}",
                s.section, icon, s.keys, s.label, cmd
            );
        }
        return Ok(());
    }

    if std::env::args().any(|a| a == "--window") {
        run_window()
    } else {
        cosmic::applet::run::<Window>(false)
    }
}
