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

/// Open or close the cheat sheet for Super+C / `--window`.
///
/// If anything is already visible, this press only closes. Otherwise it asks
/// the panel applet to open, or spawns a short-lived `--window` process.
fn run_window() -> cosmic::iced::Result {
    if ipc::anything_open() {
        ipc::close_everything();
        let _ = std::fs::remove_file(pid_path());
        return Ok(());
    }

    if ipc::request_applet_open() {
        return Ok(());
    }

    let pidf = pid_path();
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
