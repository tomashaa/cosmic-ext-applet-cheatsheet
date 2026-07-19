// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Tomas Haaland
//
// Built-in cheat-sheet data. Labels are i18n keys resolved via `crate::i18n`;
// shortcut strings are language-neutral. Clickable actions have a non-empty
// `command`; informational rows do not.

pub struct Action {
    pub icon: &'static str,
    pub label_key: &'static str,
    pub keys: &'static str,
    pub command: &'static [&'static str],
}

pub struct Section {
    pub title_key: &'static str,
    /// (label i18n key, shortcut string)
    pub rows: &'static [(&'static str, &'static str)],
}

/// Clickable actions — clicking spawns `command`.
pub const ACTIONS: &[Action] = &[
    Action { icon: "🖥", label_key: "act.terminal", keys: "Super + T", command: &["cosmic-term"] },
    Action { icon: "📁", label_key: "act.files", keys: "Super + F", command: &["cosmic-files"] },
    Action { icon: "📷", label_key: "act.shot_file", keys: "Print", command: &["cosmic-screenshot"] },
    Action { icon: "✂", label_key: "act.shot_clip", keys: "Super + Shift + S", command: &["cosmic-shot-clip"] },
    Action { icon: "🔡", label_key: "act.ocr", keys: "Super + Shift + T", command: &["cosmic-ocr-clip"] },
    Action { icon: "🌐", label_key: "act.ocr_lang", keys: "—", command: &["cosmic-ocr-settings"] },
    Action { icon: "⚙", label_key: "act.settings", keys: "—", command: &["cosmic-settings"] },
    Action { icon: "🔤", label_key: "act.app_lib", keys: "Super + A", command: &["cosmic-app-library"] },
    Action { icon: "🗂", label_key: "act.workspaces", keys: "Super + W", command: &["cosmic-workspaces"] },
    Action { icon: "🌐", label_key: "act.browser", keys: "Super + B", command: &["xdg-open", "http://"] },
    Action { icon: "📋", label_key: "act.clip_history", keys: "Super + V", command: &["cosmic-clip-history"] },
    Action { icon: "🔄", label_key: "act.panel_reset", keys: "Super + Shift + D", command: &["cosmic-panel-reset"] },
    Action { icon: "🔒", label_key: "act.lock", keys: "Super + Esc", command: &["loginctl", "lock-session"] },
];

/// Informational shortcut reference (not clickable).
pub const SECTIONS: &[Section] = &[
    Section {
        title_key: "sec.shot",
        rows: &[
            ("d.area_file", "Print"),
            ("d.area_clip", "Super + Shift + S"),
            ("d.ocr_clip", "Super + Shift + T"),
            ("d.drag_hint", "—"),
        ],
    },
    Section {
        title_key: "sec.tiling",
        rows: &[
            ("d.toggle_tiling", "Super + Y"),
            ("d.move_window", "Super + Shift + ←↓↑→"),
            ("d.move_focus", "Super + ←↓↑→"),
            ("d.corner", "Super + Shift + ←↓↑→"),
            ("d.stack_tabs", "Super + S"),
            ("d.split_dir", "Super + O"),
            ("d.swap", "Super + X"),
            ("d.floating", "Super + G"),
        ],
    },
    Section {
        title_key: "sec.window",
        rows: &[
            ("d.close", "Super + Q  /  Alt + F4"),
            ("d.maximize", "Super + M"),
            ("d.fullscreen", "Super + F11"),
            ("d.resize", "Super + R  /  Super + Shift + R"),
            ("d.in_out_stack", "Super + I  /  Super + U"),
        ],
    },
    Section {
        title_key: "sec.move",
        rows: &[
            ("d.to_monitor", "Super + Shift + Alt + ←↓↑→"),
            ("d.goto_ws", "Super + 1 … 9"),
            ("d.prev_next_ws", "Super + Ctrl + ←↓↑→"),
            ("d.move_win_num", "Super + Shift + 1 … 9"),
            ("d.move_prev_next", "Super + Shift + Ctrl + ←↓↑→"),
        ],
    },
    Section {
        title_key: "sec.zoom",
        rows: &[("d.zoom", "Super + +  /  Super + −")],
    },
    Section {
        title_key: "sec.system",
        rows: &[
            ("d.launcher", "Super  /  Super + /"),
            ("d.switch_window", "Alt + Tab  /  Super + Tab"),
            ("d.logout", "Super + Shift + Esc"),
            ("d.screenreader", "Super + Alt + S"),
            ("d.kb_lang", "Super + Space"),
            ("d.panel_reset", "Super + Shift + D"),
        ],
    },
    Section {
        title_key: "sec.clip",
        rows: &[
            ("d.history", "Super + V"),
            ("d.save_slot", "Super + Ctrl + 1 … 9"),
            ("d.paste_slot", "Super + Alt + 1 … 9"),
            ("d.remember_hint", "—"),
        ],
    },
    Section {
        title_key: "sec.panel",
        rows: &[
            ("d.toggle_panel", "Super + P"),
            ("d.close_now", "Esc"),
        ],
    },
];
