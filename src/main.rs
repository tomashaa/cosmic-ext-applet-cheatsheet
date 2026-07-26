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

use std::time::Duration;

use window::Window;

fn pid_path() -> std::path::PathBuf {
    let dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    std::path::Path::new(&dir).join("cosmic-ext-cheatsheet.pid")
}

/// Open or close the cheat sheet for Super+C / `--window`.
fn run_window() -> cosmic::iced::Result {
    // CLOSE — anything already visible.
    if ipc::anything_open() {
        ipc::close_everything();
        let _ = std::fs::remove_file(pid_path());
        return Ok(());
    }

    // OPEN — prefer resident panel applet; fall back to standalone if it
    // doesn't claim the sheet quickly (avoids "Super+C does nothing").
    ipc::clear_close_request();
    if ipc::request_applet_open() {
        if ipc::wait_until_open(Duration::from_millis(750)) {
            return Ok(());
        }
        log::warn!("panel applet did not open cheatsheet; falling back to --window");
        let _ = std::fs::remove_file(
            std::path::Path::new(&std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".into()))
                .join("cosmic-ext-cheatsheet.request.open"),
        );
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
