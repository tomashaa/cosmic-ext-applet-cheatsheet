// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Tomas Haaland
//
// Reads the user's ACTUAL COSMIC keyboard shortcuts by parsing the RON config:
//   /usr/share/cosmic/com.system76.CosmicSettings.Shortcuts/v1/defaults  (system)
//   ~/.config/cosmic/com.system76.CosmicSettings.Shortcuts/v1/custom      (user)
// The user's custom entries override/disable the defaults for the same binding.
//
// Each line looks like:
//   (modifiers: [Super, Shift], key: "q"): Close,
//   (modifiers: [Super], key: "c"): Spawn("/home/.../cmd --window"),

pub struct Shortcut {
    /// Display string, e.g. "Super + Shift + Q".
    pub keys: String,
    /// Human label for the action.
    pub label: String,
    /// Spawn command split into argv, when the action launches something.
    pub command: Option<Vec<String>>,
    /// Section heading key (grouping).
    pub section: &'static str,
}

/// Slice of `s` between the first `start` and the following `end`.
fn between<'a>(s: &'a str, start: &str, end: &str) -> Option<&'a str> {
    let i = s.find(start)? + start.len();
    let rest = &s[i..];
    let j = rest.find(end)?;
    Some(&rest[..j])
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
        "grave" => "`".into(),
        _ if k.chars().count() == 1 => k.to_uppercase(),
        _ => k.to_string(),
    }
}

/// "[Super, Shift]" + key -> "Super + Shift + Q".
fn key_combo(mods: &str, key: &str) -> String {
    let mut parts: Vec<String> = mods
        .split(',')
        .map(|m| m.trim())
        .filter(|m| !m.is_empty())
        .map(|m| m.to_string())
        .collect();
    parts.push(key_name(key));
    parts.join(" + ")
}

/// Simple whitespace argv split for a Spawn command.
fn argv(cmd: &str) -> Vec<String> {
    cmd.split_whitespace().map(str::to_string).collect()
}

fn dir(d: &str) -> &str {
    match d {
        "Left" => "left",
        "Right" => "right",
        "Up" => "up",
        "Down" => "down",
        "In" => "in",
        "Out" => "out",
        other => other,
    }
}

/// Map an action string to (label, section, optional command). None to hide it.
fn describe(action: &str) -> Option<(String, &'static str, Option<Vec<String>>)> {
    // Spawn("...") -> launch
    if let Some(cmd) = between(action, "Spawn(\"", "\")").or_else(|| between(action, "Spawn(\"", "\"")) {
        let label = cmd
            .split_whitespace()
            .next()
            .and_then(|p| p.rsplit('/').next())
            .unwrap_or(cmd)
            .to_string();
        return Some((label, "sec.apps", Some(argv(cmd))));
    }
    // Action with a payload, e.g. Focus(Left), Workspace(1), System(Launcher).
    let (name, arg) = match action.split_once('(') {
        Some((n, rest)) => (n, rest.trim_end_matches(')')),
        None => (action, ""),
    };
    let (label, section): (String, &'static str) = match name {
        "Close" => ("Close window".into(), "sec.window"),
        "Terminate" => ("Force quit (kill window)".into(), "sec.window"),
        "Maximize" => ("Maximize".into(), "sec.window"),
        "Fullscreen" => ("Fullscreen".into(), "sec.window"),
        "ToggleWindowFloating" => ("Toggle floating".into(), "sec.window"),
        "ToggleTiling" => ("Toggle tiling".into(), "sec.window"),
        "ToggleStacking" => ("Toggle stacking".into(), "sec.window"),
        "ToggleOrientation" => ("Toggle split orientation".into(), "sec.window"),
        "SwapWindow" => ("Swap window".into(), "sec.window"),
        "Resizing" => (format!("Resize ({})", dir(arg)), "sec.window"),
        "Focus" => (format!("Focus {}", dir(arg)), "sec.focus"),
        "Move" => (format!("Move window {}", dir(arg)), "sec.focus"),
        "Workspace" => (format!("Workspace {arg}"), "sec.ws"),
        "MoveToWorkspace" => (format!("Window to workspace {arg}"), "sec.ws"),
        "LastWorkspace" => ("Last workspace".into(), "sec.ws"),
        "MoveToLastWorkspace" => ("Window to last workspace".into(), "sec.ws"),
        "NextWorkspace" => ("Next workspace".into(), "sec.ws"),
        "PreviousWorkspace" => ("Previous workspace".into(), "sec.ws"),
        "MoveToNextWorkspace" => ("Window to next workspace".into(), "sec.ws"),
        "MoveToPreviousWorkspace" => ("Window to previous workspace".into(), "sec.ws"),
        "SwitchOutput" => (format!("Switch to {} monitor", dir(arg)), "sec.monitor"),
        "MoveToOutput" => (format!("Window to {} monitor", dir(arg)), "sec.monitor"),
        "ZoomIn" => ("Zoom in".into(), "sec.zoom"),
        "ZoomOut" => ("Zoom out".into(), "sec.zoom"),
        "System" => {
            let l = match arg {
                "Launcher" => "Launcher",
                "LockScreen" => "Lock screen",
                "LogOut" => "Log out",
                "Terminal" => "Terminal",
                "WebBrowser" => "Web browser",
                "HomeFolder" => "Files",
                "Screenshot" => "Screenshot",
                "AppLibrary" => "App library",
                "WorkspaceOverview" => "Workspace overview",
                "WindowSwitcher" => "Switch window",
                "VolumeRaise" => "Volume up",
                "VolumeLower" => "Volume down",
                "Mute" => "Mute",
                other => return Some((other.to_string(), "sec.system", None)),
            };
            (l.to_string(), "sec.system")
        }
        // Internal / not user-facing.
        "Debug" | "Disable" => return None,
        other => (other.to_string(), "sec.other"),
    };
    Some((label, section, None))
}

fn parse_file(text: &str, out: &mut Vec<(String, String, String)>) {
    // Both compact (defaults) and pretty (custom) RON: flatten whitespace, then
    // split into bindings at each "(modifiers: [".
    let flat: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
    for seg in flat.split("modifiers: [").skip(1) {
        let mods = seg.split(']').next().unwrap_or("").trim();
        let Some(key) = between(seg, "key: \"", "\"") else {
            continue;
        };
        let Some((_, after)) = seg.split_once("): ") else {
            continue;
        };
        let action = after
            .split(',')
            .next()
            .unwrap_or("")
            .trim()
            .trim_end_matches('}')
            .trim()
            .to_string();
        if action.is_empty() {
            continue;
        }
        // Dedup/override key: modifier set (order-independent) + key.
        let mut mset: Vec<&str> = mods
            .split(',')
            .map(|m| m.trim())
            .filter(|m| !m.is_empty())
            .collect();
        mset.sort_unstable();
        let id = format!("{}|{}", mset.join("+"), key);
        out.push((id, format!("{mods}\u{0}{key}"), action));
    }
}

/// Load the actual COSMIC shortcuts, custom overriding defaults.
pub fn load() -> Vec<Shortcut> {
    let mut raw: Vec<(String, String, String)> = Vec::new();
    if let Ok(t) =
        std::fs::read_to_string("/usr/share/cosmic/com.system76.CosmicSettings.Shortcuts/v1/defaults")
    {
        parse_file(&t, &mut raw);
    }
    if let Some(home) = std::env::var_os("HOME") {
        let p = std::path::Path::new(&home)
            .join(".config/cosmic/com.system76.CosmicSettings.Shortcuts/v1/custom");
        if let Ok(t) = std::fs::read_to_string(p) {
            parse_file(&t, &mut raw);
        }
    }

    // Merge: later entries (custom) override earlier (defaults) by id.
    // Preserve first-seen order of ids; Disable removes.
    let mut order: Vec<String> = Vec::new();
    let mut map: std::collections::HashMap<String, (String, String)> = std::collections::HashMap::new();
    for (id, combo, action) in raw {
        if !map.contains_key(&id) {
            order.push(id.clone());
        }
        map.insert(id, (combo, action));
    }

    let mut out = Vec::new();
    for id in order {
        let Some((combo, action)) = map.get(&id) else {
            continue;
        };
        if action == "Disable" {
            continue;
        }
        let Some((label, section, command)) = describe(action) else {
            continue;
        };
        let (mods, key) = combo.split_once('\u{0}').unwrap_or((combo.as_str(), ""));
        out.push(Shortcut {
            keys: key_combo(mods, key),
            label,
            command,
            section,
        });
    }
    out
}

/// Section headings in display order.
pub const SECTION_ORDER: &[(&str, &str)] = &[
    ("sec.window", "Windows"),
    ("sec.focus", "Focus & move"),
    ("sec.ws", "Workspaces"),
    ("sec.monitor", "Monitors"),
    ("sec.zoom", "Zoom"),
    ("sec.system", "System"),
    ("sec.apps", "Applications"),
    ("sec.other", "Other"),
];
