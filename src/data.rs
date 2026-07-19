// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Tomas Haaland
//
// Built-in cheat-sheet data, ported from the GTK cosmic-cheatsheet.
// Clickable actions have a non-empty `command`; informational rows do not.

pub struct Action {
    pub icon: &'static str,
    pub label: &'static str,
    pub keys: &'static str,
    pub command: &'static [&'static str],
}

pub struct Section {
    pub title: &'static str,
    pub rows: &'static [(&'static str, &'static str)],
}

/// Clickable actions — clicking spawns `command`.
pub const ACTIONS: &[Action] = &[
    Action { icon: "🖥", label: "Terminal", keys: "Super + T", command: &["cosmic-term"] },
    Action { icon: "📁", label: "Files", keys: "Super + F", command: &["cosmic-files"] },
    Action { icon: "📷", label: "Screenshot → file", keys: "Print", command: &["cosmic-screenshot"] },
    Action { icon: "✂", label: "Screenshot → clipboard", keys: "Super + Shift + S", command: &["cosmic-shot-clip"] },
    Action { icon: "🔡", label: "OCR text from screenshot", keys: "Super + Shift + T", command: &["cosmic-ocr-clip"] },
    Action { icon: "🌐", label: "OCR languages", keys: "—", command: &["cosmic-ocr-settings"] },
    Action { icon: "⚙", label: "Settings", keys: "—", command: &["cosmic-settings"] },
    Action { icon: "🔤", label: "App library", keys: "Super + A", command: &["cosmic-app-library"] },
    Action { icon: "🗂", label: "Workspaces", keys: "Super + W", command: &["cosmic-workspaces"] },
    Action { icon: "🌐", label: "Browser", keys: "Super + B", command: &["xdg-open", "http://"] },
    Action { icon: "📋", label: "Clipboard history", keys: "Super + V", command: &["cosmic-clip-history"] },
    Action { icon: "🔄", label: "Reset panel / dock", keys: "Super + Shift + D", command: &["cosmic-panel-reset"] },
    Action { icon: "🔒", label: "Lock screen", keys: "Super + Esc", command: &["loginctl", "lock-session"] },
];

/// Informational shortcut reference (not clickable).
pub const SECTIONS: &[Section] = &[
    Section {
        title: "Screenshots",
        rows: &[
            ("Area → file", "Print"),
            ("Area → clipboard", "Super + Shift + S"),
            ("OCR → clipboard", "Super + Shift + T"),
            ("Drag to select a region", "—"),
        ],
    },
    Section {
        title: "Tiling",
        rows: &[
            ("Toggle tiling", "Super + Y"),
            ("Move window", "Super + Shift + ←↓↑→"),
            ("Move focus", "Super + ←↓↑→"),
            ("Move to corner", "Super + Shift + ←↓↑→"),
            ("Stack / tabs", "Super + S"),
            ("Split direction", "Super + O"),
            ("Swap window", "Super + X"),
            ("Toggle floating", "Super + G"),
        ],
    },
    Section {
        title: "Windows",
        rows: &[
            ("Close", "Super + Q  /  Alt + F4"),
            ("Maximize", "Super + M"),
            ("Fullscreen", "Super + F11"),
            ("Resize", "Super + R  /  Super + Shift + R"),
            ("Into / out of stack", "Super + I  /  Super + U"),
        ],
    },
    Section {
        title: "Move & workspaces",
        rows: &[
            ("To monitor", "Super + Shift + Alt + ←↓↑→"),
            ("Go to workspace", "Super + 1 … 9"),
            ("Prev / next workspace", "Super + Ctrl + ←↓↑→"),
            ("Move window to workspace", "Super + Shift + 1 … 9"),
            ("Move to prev / next workspace", "Super + Shift + Ctrl + ←↓↑→"),
        ],
    },
    Section {
        title: "Zoom",
        rows: &[("Zoom in / out", "Super + +  /  Super + −")],
    },
    Section {
        title: "System",
        rows: &[
            ("Launcher", "Super  /  Super + /"),
            ("Switch window", "Alt + Tab  /  Super + Tab"),
            ("Log out", "Super + Shift + Esc"),
            ("Screen reader", "Super + Alt + S"),
            ("Keyboard language", "Super + Space"),
            ("Reset panel / dock", "Super + Shift + D"),
        ],
    },
    Section {
        title: "Clipboard",
        rows: &[
            ("History", "Super + V"),
            ("Save to slot", "Super + Ctrl + 1 … 9"),
            ("Paste from slot", "Super + Alt + 1 … 9"),
            ("Slots are remembered across reboots", "—"),
        ],
    },
    Section {
        title: "This panel",
        rows: &[
            ("Toggle cheat sheet", "Super + P"),
            ("Close", "Esc"),
        ],
    },
];
