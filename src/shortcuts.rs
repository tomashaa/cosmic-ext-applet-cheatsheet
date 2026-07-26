// SPDX-License-Identifier: GPL-3.0-only
// Copyright (c) 2026 Tomas Haaland
//
// Loads the user's actual COSMIC keyboard shortcuts via cosmic-settings-config
// (same types Cosmic Settings / the compositor use).

use std::collections::BTreeMap;
use std::path::Path;

use cosmic_settings_config::shortcuts::action::{
    Action, Direction, FocusDirection, ResizeDirection, System,
};
use cosmic_settings_config::shortcuts::{self, Binding};

use crate::i18n;

#[derive(Clone)]
pub struct Shortcut {
    /// Display string, e.g. "Super + Shift + Q".
    pub keys: String,
    /// Human label for the action (already localized).
    pub label: String,
    /// Optional freedesktop symbolic icon name (Cosmic theme).
    pub icon: Option<&'static str>,
    /// Spawn command split into argv, when the action launches something.
    pub command: Option<Vec<String>>,
    /// Section heading key (grouping) — look up via i18n.
    pub section: &'static str,
    /// Rows with the same key are merged in compact mode.
    compact_key: Option<&'static str>,
    /// Hidden entirely when compact mode is on (e.g. media keys).
    compact_hide: bool,
}

/// Pretty-print a single key name (uppercase letters, arrows, common names).
fn key_name(k: &str) -> String {
    match k {
        "Left" => "←".into(),
        "Right" => "→".into(),
        "Up" => "↑".into(),
        "Down" => "↓".into(),
        "Escape" => "Esc".into(),
        "Return" => "Enter".into(),
        "grave" | "Grave" => "`".into(),
        "space" | "Space" => "Space".into(),
        "plus" | "Plus" => "+".into(),
        "minus" | "Minus" => "−".into(),
        _ if k.chars().count() == 1 => k.to_uppercase(),
        _ => k.to_string(),
    }
}

fn pretty_keys(binding: &Binding) -> String {
    let raw = binding.to_string();
    if raw.is_empty() {
        return String::new();
    }
    raw.split('+')
        .map(|p| key_name(p.trim()))
        .collect::<Vec<_>>()
        .join(" + ")
}

fn tr_or(lang: &str, key: &str, en: &str) -> String {
    let t = i18n::tr(lang, key);
    if t.is_empty() {
        en.to_string()
    } else {
        t.to_string()
    }
}

fn tr_arg(lang: &str, key: &str, en_fmt: &str, arg: &str) -> String {
    let t = i18n::tr(lang, key);
    if t.is_empty() {
        en_fmt.replace("{}", arg)
    } else {
        t.replace("{}", arg)
    }
}

fn dir_word(lang: &str, d: Direction) -> String {
    match d {
        Direction::Left => tr_or(lang, "dir.left", "left"),
        Direction::Right => tr_or(lang, "dir.right", "right"),
        Direction::Up => tr_or(lang, "dir.up", "up"),
        Direction::Down => tr_or(lang, "dir.down", "down"),
    }
}

fn focus_word(lang: &str, d: FocusDirection) -> String {
    match d {
        FocusDirection::Left => tr_or(lang, "dir.left", "left"),
        FocusDirection::Right => tr_or(lang, "dir.right", "right"),
        FocusDirection::Up => tr_or(lang, "dir.up", "up"),
        FocusDirection::Down => tr_or(lang, "dir.down", "down"),
        FocusDirection::In => tr_or(lang, "dir.in", "in"),
        FocusDirection::Out => tr_or(lang, "dir.out", "out"),
    }
}

/// Split a Spawn command into argv, respecting quotes.
pub fn split_command(cmd: &str) -> Vec<String> {
    shlex::split(cmd).unwrap_or_else(|| vec![cmd.to_string()])
}

fn is_self_cheatsheet(cmd: &str) -> bool {
    cmd.to_ascii_lowercase()
        .contains("cosmic-ext-applet-cheatsheet")
}

fn spawn_bin(cmd: &str) -> String {
    split_command(cmd)
        .into_iter()
        .next()
        .map(|p| {
            Path::new(&p)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(&p)
                .to_string()
        })
        .unwrap_or_else(|| cmd.to_string())
}

fn spawn_icon(cmd: &str) -> Option<&'static str> {
    let bin = spawn_bin(cmd);
    let args = split_command(cmd);
    Some(match bin.as_str() {
        "cosmic-term" => "utilities-terminal-symbolic",
        "cosmic-files" => "system-file-manager-symbolic",
        "cosmic-edit" => "accessories-text-editor-symbolic",
        "cosmic-screenshot" => "accessories-screenshot-symbolic",
        "cosmic-shot-clip" => "edit-cut-symbolic",
        "cosmic-ocr-clip" => "edit-select-all-symbolic",
        "cosmic-ocr-settings" => "preferences-desktop-locale-symbolic",
        "cosmic-settings" => "preferences-system-symbolic",
        "cosmic-app-library" => "preferences-applications-symbolic",
        "cosmic-workspaces" => "preferences-workspaces-symbolic",
        "cosmic-clip-history" => "edit-paste-symbolic",
        "cosmic-clip-slot" => {
            if args.iter().any(|a| a == "save") {
                "edit-copy-symbolic"
            } else if args.iter().any(|a| a == "paste") {
                "edit-paste-symbolic"
            } else {
                "edit-copy-symbolic"
            }
        }
        "cosmic-panel-reset" => "preferences-panel-symbolic",
        "cosmic-annotate" => "edit-symbolic",
        "cosmic-cheatsheet-toggle" | "cosmic-cheatsheet" => "input-keyboard-symbolic",
        _ if bin.contains("firefox") || bin.contains("chrome") || bin.contains("browser") => {
            "web-browser-symbolic"
        }
        _ => return None,
    })
}

fn spawn_label(lang: &str, cmd: &str, description: Option<&str>) -> String {
    if let Some(desc) = description.filter(|d| !d.trim().is_empty()) {
        return desc.to_string();
    }
    let bin = spawn_bin(cmd);
    match bin.as_str() {
        "cosmic-files" => tr_or(lang, "act.files", "Files"),
        "cosmic-term" => tr_or(lang, "act.terminal", "Terminal"),
        "cosmic-edit" => tr_or(lang, "act.editor", "Editor"),
        "cosmic-settings" => tr_or(lang, "act.settings", "Settings"),
        "cosmic-app-library" => tr_or(lang, "act.app_lib", "App library"),
        "cosmic-workspaces" => tr_or(lang, "act.workspaces", "Workspaces"),
        "cosmic-screenshot" => tr_or(lang, "act.shot_file", "Screenshot → file"),
        "cosmic-shot-clip" => tr_or(lang, "act.shot_clip", "Screenshot → clipboard"),
        "cosmic-ocr-clip" => tr_or(lang, "act.ocr", "Text from image (OCR)"),
        "cosmic-ocr-settings" => tr_or(lang, "act.ocr_lang", "OCR language (setting)"),
        "cosmic-clip-history" => tr_or(lang, "act.clip_history", "Clipboard history"),
        "cosmic-panel-reset" => tr_or(lang, "act.panel_reset", "Reset panel/dock"),
        "cosmic-annotate" => tr_or(lang, "act.annotate", "Annotate"),
        "cosmic-cheatsheet-toggle" | "cosmic-cheatsheet" => {
            tr_or(lang, "act.cheat_gtk", "Cheat sheet (GTK)")
        }
        "cosmic-clip-slot" => {
            let args = split_command(cmd);
            if args.iter().any(|a| a == "save") {
                tr_or(lang, "d.save_slot", "Save to slot 1–9")
            } else if args.iter().any(|a| a == "paste") {
                tr_or(lang, "d.paste_slot", "Paste from slot 1–9")
            } else {
                tr_or(lang, "act.clip_history", "Clipboard history")
            }
        }
        other => other.to_string(),
    }
}

fn spawn_compact(cmd: &str) -> (Option<&'static str>, bool) {
    let lower = cmd.to_ascii_lowercase();
    if lower.contains("cosmic-clip-slot") && lower.contains("save") {
        return (Some("clip_save"), false);
    }
    if lower.contains("cosmic-clip-slot") && lower.contains("paste") {
        return (Some("clip_paste"), false);
    }
    (None, false)
}

fn system_icon(sys: &System) -> Option<&'static str> {
    Some(match sys {
        System::Terminal => "utilities-terminal-symbolic",
        System::HomeFolder => "user-home-symbolic",
        System::WebBrowser => "web-browser-symbolic",
        System::Screenshot => "accessories-screenshot-symbolic",
        System::AppLibrary => "preferences-applications-symbolic",
        System::WorkspaceOverview => "preferences-workspaces-symbolic",
        System::LockScreen => "system-lock-screen-symbolic",
        System::LogOut => "system-log-out-symbolic",
        System::Launcher => "system-search-symbolic",
        System::WindowSwitcher | System::WindowSwitcherPrevious => "focus-windows-symbolic",
        System::ScreenReader => "preferences-desktop-accessibility-symbolic",
        System::InputSourceSwitch => "input-keyboard-symbolic",
        System::VolumeRaise => "audio-volume-high-symbolic",
        System::VolumeLower => "audio-volume-low-symbolic",
        System::Mute => "audio-volume-muted-symbolic",
        System::MuteMic => "microphone-sensitivity-muted-symbolic",
        System::PlayPause => "media-playback-start-symbolic",
        System::PlayNext => "media-skip-forward-symbolic",
        System::PlayPrev => "media-skip-backward-symbolic",
        System::BrightnessUp => "display-brightness-high-symbolic",
        System::BrightnessDown => "display-brightness-low-symbolic",
        System::KeyboardBrightnessUp | System::KeyboardBrightnessDown => {
            "keyboard-brightness-symbolic"
        }
        System::DisplayToggle => "video-display-symbolic",
        System::TouchpadToggle => "input-touchpad-symbolic",
        System::Suspend => "system-suspend-symbolic",
        System::PowerOff => "system-shutdown-symbolic",
    })
}

/// Icon for a compact-mode merge group.
fn compact_icon(key: &str, fallback: Option<&'static str>) -> Option<&'static str> {
    Some(match key {
        "focus_cardinal" => "focus-windows-symbolic",
        "focus_stack" => "window-stack-symbolic",
        "move_cardinal" => "preferences-window-management-symbolic",
        "workspace_num" | "ws_next_prev" | "migrate_ws" => "preferences-workspaces-symbolic",
        "move_to_ws_num" | "move_ws_next_prev" => "go-jump-symbolic",
        "switch_output" | "move_to_output" => "video-display-symbolic",
        "resize" => "view-fullscreen-symbolic",
        "zoom" => "zoom-in-symbolic",
        "close" => "window-close-symbolic",
        "clip_save" => "edit-copy-symbolic",
        "clip_paste" => "edit-paste-symbolic",
        _ => return fallback,
    })
}

fn is_media_system(sys: &System) -> bool {
    matches!(
        sys,
        System::VolumeRaise
            | System::VolumeLower
            | System::Mute
            | System::MuteMic
            | System::PlayPause
            | System::PlayNext
            | System::PlayPrev
            | System::BrightnessUp
            | System::BrightnessDown
            | System::KeyboardBrightnessUp
            | System::KeyboardBrightnessDown
    )
}

struct Described {
    label: String,
    section: &'static str,
    command: Option<Vec<String>>,
    icon: Option<&'static str>,
    compact_key: Option<&'static str>,
    compact_hide: bool,
}

/// Map an action to display metadata. None to hide it.
fn describe(lang: &str, action: &Action, binding: &Binding) -> Option<Described> {
    match action {
        Action::Disable | Action::Debug => None,
        Action::Spawn(cmd) => {
            if is_self_cheatsheet(cmd) {
                return None;
            }
            let (compact_key, compact_hide) = spawn_compact(cmd);
            Some(Described {
                label: spawn_label(lang, cmd, binding.description.as_deref()),
                section: "sec.apps",
                command: Some(split_command(cmd)),
                icon: spawn_icon(cmd),
                compact_key,
                compact_hide,
            })
        }
        Action::Close => Some(Described {
            label: tr_or(lang, "d.close", "Close window"),
            section: "sec.window",
            command: None,
            icon: Some("window-close-symbolic"),
            compact_key: Some("close"),
            compact_hide: false,
        }),
        Action::Terminate => Some(Described {
            label: tr_or(lang, "act.terminate", "Force quit (kill window)"),
            section: "sec.window",
            command: None,
            icon: Some("process-stop-symbolic"),
            compact_key: None,
            compact_hide: false,
        }),
        Action::Maximize => Some(Described {
            label: tr_or(lang, "d.maximize", "Maximize"),
            section: "sec.window",
            command: None,
            icon: Some("window-maximize-symbolic"),
            compact_key: None,
            compact_hide: false,
        }),
        Action::Fullscreen => Some(Described {
            label: tr_or(lang, "d.fullscreen", "Fullscreen"),
            section: "sec.window",
            command: None,
            icon: Some("view-fullscreen-symbolic"),
            compact_key: None,
            compact_hide: false,
        }),
        Action::Minimize => Some(Described {
            label: tr_or(lang, "act.minimize", "Minimize"),
            section: "sec.window",
            command: None,
            icon: Some("window-minimize-symbolic"),
            compact_key: None,
            compact_hide: false,
        }),
        Action::ToggleWindowFloating => Some(Described {
            label: tr_or(lang, "d.floating", "Floating (exempt from tiling)"),
            section: "sec.window",
            command: None,
            icon: Some("window-pop-out-symbolic"),
            compact_key: None,
            compact_hide: false,
        }),
        Action::ToggleTiling => Some(Described {
            label: tr_or(lang, "d.toggle_tiling", "Toggle tiling on/off"),
            section: "sec.window",
            command: None,
            icon: Some("com.system76.CosmicAppletTiling-symbolic"),
            compact_key: None,
            compact_hide: false,
        }),
        Action::ToggleStacking => Some(Described {
            label: tr_or(lang, "d.stack_tabs", "Stack windows (tabs) on/off"),
            section: "sec.window",
            command: None,
            icon: Some("window-stack-symbolic"),
            compact_key: None,
            compact_hide: false,
        }),
        Action::ToggleOrientation => Some(Described {
            label: tr_or(lang, "d.split_dir", "Switch split direction"),
            section: "sec.window",
            command: None,
            icon: Some("object-flip-horizontal-symbolic"),
            compact_key: None,
            compact_hide: false,
        }),
        Action::ToggleSticky => Some(Described {
            label: tr_or(lang, "act.toggle_sticky", "Toggle sticky"),
            section: "sec.window",
            command: None,
            icon: Some("emblem-important-symbolic"),
            compact_key: None,
            compact_hide: false,
        }),
        Action::SwapWindow => Some(Described {
            label: tr_or(lang, "d.swap", "Swap two windows"),
            section: "sec.window",
            command: None,
            icon: Some("window-swap-symbolic"),
            compact_key: None,
            compact_hide: false,
        }),
        Action::Resizing(ResizeDirection::Inwards) => Some(Described {
            label: tr_or(lang, "act.resize_in", "Resize in"),
            section: "sec.window",
            command: None,
            icon: Some("zoom-out-symbolic"),
            compact_key: Some("resize"),
            compact_hide: false,
        }),
        Action::Resizing(ResizeDirection::Outwards) => Some(Described {
            label: tr_or(lang, "act.resize_out", "Resize out"),
            section: "sec.window",
            command: None,
            icon: Some("zoom-in-symbolic"),
            compact_key: Some("resize"),
            compact_hide: false,
        }),
        Action::Focus(d) => {
            let (compact_key, icon) = match d {
                FocusDirection::Left => (Some("focus_cardinal"), Some("go-previous-symbolic")),
                FocusDirection::Right => (Some("focus_cardinal"), Some("go-next-symbolic")),
                FocusDirection::Up => (Some("focus_cardinal"), Some("go-up-symbolic")),
                FocusDirection::Down => (Some("focus_cardinal"), Some("go-down-symbolic")),
                FocusDirection::In => (Some("focus_stack"), Some("go-jump-symbolic")),
                FocusDirection::Out => (Some("focus_stack"), Some("window-pop-out-symbolic")),
            };
            Some(Described {
                label: tr_arg(lang, "act.focus", "Focus {}", &focus_word(lang, *d)),
                section: "sec.focus",
                command: None,
                icon,
                compact_key,
                compact_hide: false,
            })
        }
        Action::Move(d) => {
            let icon = match d {
                Direction::Left => "go-previous-symbolic",
                Direction::Right => "go-next-symbolic",
                Direction::Up => "go-up-symbolic",
                Direction::Down => "go-down-symbolic",
            };
            Some(Described {
                label: tr_arg(lang, "act.move_win", "Move window {}", &dir_word(lang, *d)),
                section: "sec.focus",
                command: None,
                icon: Some(icon),
                compact_key: Some("move_cardinal"),
                compact_hide: false,
            })
        }
        Action::Workspace(_) => Some(Described {
            label: tr_or(lang, "d.goto_ws", "Go to workspace 1–9"),
            section: "sec.ws",
            command: None,
            icon: Some("preferences-workspaces-symbolic"),
            compact_key: Some("workspace_num"),
            compact_hide: false,
        }),
        Action::MoveToWorkspace(_) | Action::SendToWorkspace(_) => Some(Described {
            label: tr_or(lang, "d.move_win_num", "Move window to no."),
            section: "sec.ws",
            command: None,
            icon: Some("go-jump-symbolic"),
            compact_key: Some("move_to_ws_num"),
            compact_hide: false,
        }),
        Action::LastWorkspace => Some(Described {
            label: tr_or(lang, "act.last_ws", "Last workspace"),
            section: "sec.ws",
            command: None,
            icon: Some("go-last-symbolic"),
            compact_key: None,
            compact_hide: false,
        }),
        Action::MoveToLastWorkspace | Action::SendToLastWorkspace => Some(Described {
            label: tr_or(lang, "act.move_to_last_ws", "Window to last workspace"),
            section: "sec.ws",
            command: None,
            icon: Some("go-last-symbolic"),
            compact_key: None,
            compact_hide: false,
        }),
        Action::NextWorkspace => Some(Described {
            label: tr_or(lang, "act.next_ws", "Next workspace"),
            section: "sec.ws",
            command: None,
            icon: Some("go-next-symbolic"),
            compact_key: Some("ws_next_prev"),
            compact_hide: false,
        }),
        Action::PreviousWorkspace => Some(Described {
            label: tr_or(lang, "act.prev_ws", "Previous workspace"),
            section: "sec.ws",
            command: None,
            icon: Some("go-previous-symbolic"),
            compact_key: Some("ws_next_prev"),
            compact_hide: false,
        }),
        Action::MoveToNextWorkspace | Action::SendToNextWorkspace => Some(Described {
            label: tr_or(lang, "act.move_to_next_ws", "Window to next workspace"),
            section: "sec.ws",
            command: None,
            icon: Some("go-next-symbolic"),
            compact_key: Some("move_ws_next_prev"),
            compact_hide: false,
        }),
        Action::MoveToPreviousWorkspace | Action::SendToPreviousWorkspace => Some(Described {
            label: tr_or(lang, "act.move_to_prev_ws", "Window to previous workspace"),
            section: "sec.ws",
            command: None,
            icon: Some("go-previous-symbolic"),
            compact_key: Some("move_ws_next_prev"),
            compact_hide: false,
        }),
        Action::SwitchOutput(_) => Some(Described {
            label: tr_or(lang, "act.switch_output_any", "Switch monitor"),
            section: "sec.monitor",
            command: None,
            icon: Some("video-display-symbolic"),
            compact_key: Some("switch_output"),
            compact_hide: false,
        }),
        Action::MoveToOutput(_) | Action::SendToOutput(_) => Some(Described {
            label: tr_or(lang, "d.to_monitor", "To another monitor"),
            section: "sec.monitor",
            command: None,
            icon: Some("display-symbolic"),
            compact_key: Some("move_to_output"),
            compact_hide: false,
        }),
        Action::MigrateWorkspaceToOutput(_) => Some(Described {
            label: tr_or(lang, "act.migrate_ws_any", "Workspace to another monitor"),
            section: "sec.monitor",
            command: None,
            icon: Some("preferences-workspaces-symbolic"),
            compact_key: Some("migrate_ws"),
            compact_hide: false,
        }),
        Action::ZoomIn => Some(Described {
            label: tr_or(lang, "act.zoom_in", "Zoom in"),
            section: "sec.zoom",
            command: None,
            icon: Some("zoom-in-symbolic"),
            compact_key: Some("zoom"),
            compact_hide: false,
        }),
        Action::ZoomOut => Some(Described {
            label: tr_or(lang, "act.zoom_out", "Zoom out"),
            section: "sec.zoom",
            command: None,
            icon: Some("zoom-out-symbolic"),
            compact_key: Some("zoom"),
            compact_hide: false,
        }),
        Action::System(sys) => {
            let (key, en, section) = match sys {
                System::Launcher => ("d.launcher", "Launcher (search / start app)", "sec.system"),
                System::LockScreen => ("act.lock", "Lock screen", "sec.system"),
                System::LogOut => ("d.logout", "Log out", "sec.system"),
                System::Terminal => ("act.terminal", "Terminal", "sec.system"),
                System::WebBrowser => ("act.browser", "Browser", "sec.system"),
                System::HomeFolder => ("act.files", "Files", "sec.system"),
                System::Screenshot => ("act.shot_file", "Screenshot → file", "sec.system"),
                System::AppLibrary => ("act.app_lib", "App library", "sec.system"),
                System::WorkspaceOverview => ("act.workspaces", "Workspaces", "sec.system"),
                System::WindowSwitcher => ("d.switch_window", "Switch window", "sec.system"),
                System::WindowSwitcherPrevious => {
                    ("act.switch_window_prev", "Switch window (previous)", "sec.system")
                }
                System::VolumeRaise => ("act.vol_up", "Volume up", "sec.system"),
                System::VolumeLower => ("act.vol_down", "Volume down", "sec.system"),
                System::Mute => ("act.mute", "Mute", "sec.system"),
                System::MuteMic => ("act.mute_mic", "Mute microphone", "sec.system"),
                System::PlayPause => ("act.play_pause", "Play / pause", "sec.system"),
                System::PlayNext => ("act.play_next", "Next track", "sec.system"),
                System::PlayPrev => ("act.play_prev", "Previous track", "sec.system"),
                System::BrightnessUp => ("act.brightness_up", "Brightness up", "sec.system"),
                System::BrightnessDown => ("act.brightness_down", "Brightness down", "sec.system"),
                System::KeyboardBrightnessUp => {
                    ("act.kb_brightness_up", "Keyboard brightness up", "sec.system")
                }
                System::KeyboardBrightnessDown => (
                    "act.kb_brightness_down",
                    "Keyboard brightness down",
                    "sec.system",
                ),
                System::ScreenReader => ("d.screenreader", "Screen reader", "sec.system"),
                System::InputSourceSwitch => ("d.kb_lang", "Switch keyboard language", "sec.system"),
                System::DisplayToggle => ("act.display_toggle", "Toggle display", "sec.system"),
                System::TouchpadToggle => ("act.touchpad_toggle", "Toggle touchpad", "sec.system"),
                System::Suspend => ("act.suspend", "Suspend", "sec.system"),
                System::PowerOff => ("act.power_off", "Power off", "sec.system"),
            };
            Some(Described {
                label: tr_or(lang, key, en),
                section,
                command: None,
                icon: system_icon(sys),
                compact_key: None,
                compact_hide: is_media_system(sys),
            })
        }
        // Deprecated / less common — still show under Other.
        other => Some(Described {
            label: format!("{other:?}"),
            section: "sec.other",
            command: None,
            icon: Some("preferences-other-symbolic"),
            compact_key: None,
            compact_hide: true,
        }),
    }
}

/// Load the actual COSMIC shortcuts (defaults + custom merge), localized.
pub fn load(lang: &str) -> Vec<Shortcut> {
    let Ok(ctx) = shortcuts::context() else {
        log::warn!("could not open COSMIC shortcuts config context");
        return Vec::new();
    };
    let map = shortcuts::shortcuts(&ctx);

    let mut out = Vec::new();
    for (binding, action) in map.iter() {
        let Some(d) = describe(lang, action, binding) else {
            continue;
        };
        let keys = pretty_keys(binding);
        if keys.is_empty() {
            continue;
        }
        out.push(Shortcut {
            keys,
            label: d.label,
            icon: d.icon,
            command: d.command,
            section: d.section,
            compact_key: d.compact_key,
            compact_hide: d.compact_hide,
        });
    }

    sort_shortcuts(&mut out);
    out
}

/// View of shortcuts for the UI: full list, or Super+P-style compact.
pub fn for_display(full: &[Shortcut], lang: &str, compact: bool) -> Vec<Shortcut> {
    if !compact {
        return full.to_vec();
    }
    compactify(full, lang)
}

fn sort_shortcuts(out: &mut [Shortcut]) {
    out.sort_by(|a, b| {
        section_index(a.section)
            .cmp(&section_index(b.section))
            .then_with(|| a.keys.cmp(&b.keys))
            .then_with(|| a.label.cmp(&b.label))
    });
}

fn last_key_token(keys: &str) -> &str {
    keys.rsplit(" + ").next().unwrap_or(keys).trim()
}

fn mods_prefix(keys: &str) -> String {
    let parts: Vec<&str> = keys.split(" + ").map(str::trim).collect();
    if parts.len() <= 1 {
        String::new()
    } else {
        parts[..parts.len() - 1].join(" + ")
    }
}

fn is_arrow_token(t: &str) -> bool {
    matches!(t, "←" | "→" | "↑" | "↓")
}

fn is_letter_token(t: &str) -> bool {
    t.chars().count() == 1 && t.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
}

/// Build GTK-style "Super + ←↓↑→" (and optional " / Super + HJKL").
fn merge_cardinal_keys(entries: &[&Shortcut]) -> String {
    let mut arrow_mods: Option<String> = None;
    let mut letter_mods: Option<String> = None;
    let mut letters: Vec<char> = Vec::new();

    for e in entries {
        let tok = last_key_token(&e.keys);
        let mods = mods_prefix(&e.keys);
        if is_arrow_token(tok) {
            if arrow_mods.is_none() {
                arrow_mods = Some(mods);
            }
        } else if is_letter_token(tok) {
            if letter_mods.is_none() {
                letter_mods = Some(mods);
            }
            if let Some(c) = tok.chars().next() {
                if !letters.contains(&c) {
                    letters.push(c);
                }
            }
        }
    }

    // Stable letter order H J K L when present, else alpha.
    let preferred = ['H', 'J', 'K', 'L'];
    letters.sort_by_key(|c| {
        preferred
            .iter()
            .position(|p| p == c)
            .unwrap_or(100 + *c as usize)
    });

    let mut parts = Vec::new();
    if let Some(mods) = arrow_mods {
        let fan = if mods.is_empty() {
            "←↓↑→".to_string()
        } else {
            format!("{mods} + ←↓↑→")
        };
        parts.push(fan);
    }
    if let (Some(mods), true) = (letter_mods, !letters.is_empty()) {
        let body: String = letters.into_iter().collect();
        let spaced: String = body.chars().map(|c| c.to_string()).collect::<Vec<_>>().join(" ");
        let fan = if mods.is_empty() {
            spaced
        } else {
            format!("{mods} + {spaced}")
        };
        // Only add letter fan if we didn't already show arrows with same idea,
        // or always as secondary — GTK omits letters; we keep arrows-only when both exist.
        if parts.is_empty() {
            parts.push(fan);
        }
    }

    if parts.is_empty() {
        // Fallback: join unique key combos.
        let mut uniq = Vec::new();
        for e in entries {
            if !uniq.contains(&e.keys) {
                uniq.push(e.keys.clone());
            }
        }
        return uniq.join("  /  ");
    }
    parts.join("  /  ")
}

fn merge_simple_keys(entries: &[&Shortcut]) -> String {
    let mut uniq = Vec::new();
    for e in entries {
        if !uniq.contains(&e.keys) {
            uniq.push(e.keys.clone());
        }
    }
    // Prefer a short combined form for numbered series.
    if let Some(merged) = try_merge_number_series(&uniq) {
        return merged;
    }
    uniq.join("  /  ")
}

fn try_merge_number_series(keys: &[String]) -> Option<String> {
    if keys.len() < 2 {
        return None;
    }
    let mut mods: Option<String> = None;
    let mut nums: Vec<u32> = Vec::new();
    for k in keys {
        let tok = last_key_token(k);
        let n: u32 = tok.parse().ok()?;
        let m = mods_prefix(k);
        match &mods {
            None => mods = Some(m),
            Some(existing) if *existing == m => {}
            _ => return None,
        }
        if !nums.contains(&n) {
            nums.push(n);
        }
    }
    nums.sort_unstable();
    let mods = mods?;
    let (lo, hi) = (*nums.first()?, *nums.last()?);
    if nums.len() as u32 != hi - lo + 1 {
        // Not a contiguous run — still show range if mostly 1–9.
        if lo == 1 && hi == 9 {
            // ok
        } else {
            return None;
        }
    }
    let body = format!("{lo} … {hi}");
    if mods.is_empty() {
        Some(body)
    } else {
        Some(format!("{mods} + {body}"))
    }
}

fn compact_label(lang: &str, key: &str, fallback: &str) -> String {
    match key {
        "focus_cardinal" => tr_or(lang, "d.move_focus", "Move focus between windows"),
        "focus_stack" => tr_or(lang, "d.in_out_stack", "Into / out of stack"),
        "move_cardinal" => tr_or(lang, "d.move_window", "Move window (builds corners)"),
        "workspace_num" => tr_or(lang, "d.goto_ws", "Go to workspace 1–9"),
        "move_to_ws_num" => tr_or(lang, "d.move_win_num", "Move window to no."),
        "ws_next_prev" => tr_or(lang, "d.prev_next_ws", "Previous / next workspace"),
        "move_ws_next_prev" => tr_or(lang, "d.move_prev_next", "Move to previous/next"),
        "switch_output" => tr_or(lang, "act.switch_output_any", "Switch monitor"),
        "move_to_output" => tr_or(lang, "d.to_monitor", "To another monitor"),
        "migrate_ws" => tr_or(lang, "act.migrate_ws_any", "Workspace to another monitor"),
        "resize" => tr_or(lang, "d.resize", "Resize out / in"),
        "zoom" => tr_or(lang, "d.zoom", "Zoom in / out"),
        "close" => tr_or(lang, "d.close", "Close window"),
        "clip_save" => tr_or(lang, "d.save_slot", "Save to slot 1–9"),
        "clip_paste" => tr_or(lang, "d.paste_slot", "Paste from slot 1–9"),
        _ => fallback.to_string(),
    }
}

fn compactify(full: &[Shortcut], lang: &str) -> Vec<Shortcut> {
    let mut groups: BTreeMap<&'static str, Vec<&Shortcut>> = BTreeMap::new();
    let mut singles: Vec<Shortcut> = Vec::new();

    for s in full {
        if s.compact_hide {
            continue;
        }
        // Hardware media / special keys clutter the overview — hide in compact.
        if s.keys.contains("XF86") {
            continue;
        }
        match s.compact_key {
            Some(key) => groups.entry(key).or_default().push(s),
            None => singles.push(s.clone()),
        }
    }

    let mut out = singles;
    for (key, entries) in groups {
        if entries.is_empty() {
            continue;
        }
        let keys = match key {
            // Direction families → GTK-style "Super + ←↓↑→" (never list every HJKL combo).
            "focus_cardinal"
            | "move_cardinal"
            | "switch_output"
            | "move_to_output"
            | "migrate_ws"
            | "ws_next_prev"
            | "move_ws_next_prev" => merge_cardinal_keys(&entries),
            _ => merge_simple_keys(&entries),
        };
        let first = entries[0];
        // Prefer a clickable command if any member has one (clip slots).
        let command = entries.iter().find_map(|e| e.command.clone());
        out.push(Shortcut {
            keys,
            label: compact_label(lang, key, &first.label),
            icon: compact_icon(key, first.icon),
            command,
            section: first.section,
            compact_key: Some(key),
            compact_hide: false,
        });
    }

    sort_shortcuts(&mut out);
    out
}

fn section_index(sec: &str) -> usize {
    SECTION_ORDER
        .iter()
        .position(|s| *s == sec)
        .unwrap_or(SECTION_ORDER.len())
}

/// Section heading keys in display order.
pub const SECTION_ORDER: &[&str] = &[
    "sec.apps",
    "sec.system",
    "sec.window",
    "sec.focus",
    "sec.ws",
    "sec.monitor",
    "sec.zoom",
    "sec.other",
];
