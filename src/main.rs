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

fn main() -> cosmic::iced::Result {
    let env = env_logger::Env::default().filter_or("RUST_LOG", "warn");
    env_logger::init_from_env(env);

    // `--window`: open the cheat sheet as a standalone window (for a keybind
    // like Super+C). Otherwise run as a COSMIC panel applet.
    if std::env::args().any(|a| a == "--window") {
        let settings = cosmic::app::Settings::default()
            .size(cosmic::iced::Size::new(480.0, 620.0))
            .is_daemon(false);
        cosmic::app::run::<Window>(settings, true)
    } else {
        cosmic::applet::run::<Window>(false)
    }
}
