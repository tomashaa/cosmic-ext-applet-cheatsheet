// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Tomas Haaland
//
// User-defined custom shortcuts, read from
//   ~/.config/cosmic-ext-cheatsheet/custom.toml
//
// Example:
//   [[shortcut]]
//   label = "My screenshot tool"
//   keys = "Super + Shift + M"
//   command = "my-tool --flag"   # optional; if present the row is clickable
//   section = "My tools"          # optional grouping (defaults to "Custom")

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CustomShortcut {
    pub label: String,
    pub keys: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub section: Option<String>,
}

impl CustomShortcut {
    /// The command split into argv, or empty if the row is informational.
    pub fn argv(&self) -> Vec<String> {
        match &self.command {
            Some(c) => c.split_whitespace().map(str::to_string).collect(),
            None => Vec::new(),
        }
    }

    pub fn section_or_default(&self) -> &str {
        self.section.as_deref().unwrap_or("Custom")
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct CustomConfig {
    #[serde(default)]
    shortcut: Vec<CustomShortcut>,
}

/// Write the custom shortcuts back to `custom.toml`, creating the dir if needed.
pub fn save(shortcuts: &[CustomShortcut]) -> std::io::Result<()> {
    let Some(path) = config_path() else {
        return Ok(());
    };
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let cfg = CustomConfig {
        shortcut: shortcuts.to_vec(),
    };
    let text = toml::to_string_pretty(&cfg)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    std::fs::write(path, text)
}

fn config_path() -> Option<std::path::PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(std::path::Path::new(&home).join(".config/cosmic-ext-cheatsheet/custom.toml"))
}

/// Load user shortcuts; returns an empty list if the file is missing or invalid.
pub fn load() -> Vec<CustomShortcut> {
    let Some(path) = config_path() else {
        return Vec::new();
    };
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };
    match toml::from_str::<CustomConfig>(&text) {
        Ok(cfg) => cfg.shortcut,
        Err(e) => {
            log::warn!("invalid {}: {e}", path.display());
            Vec::new()
        }
    }
}

// ---- Settings (persistent) + last-search state (per session) ----

fn config_dir_file(name: &str) -> Option<std::path::PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(std::path::Path::new(&home).join(".config/cosmic-ext-cheatsheet").join(name))
}

fn state_path() -> Option<std::path::PathBuf> {
    let dir = std::env::var_os("XDG_RUNTIME_DIR")?;
    Some(std::path::Path::new(&dir).join("cosmic-ext-cheatsheet-state.toml"))
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    /// Remember the last search + scroll across opens (default true).
    #[serde(default = "default_true")]
    pub remember: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self { remember: true }
    }
}

pub fn load_settings() -> Settings {
    config_dir_file("settings.toml")
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|t| toml::from_str(&t).ok())
        .unwrap_or_default()
}

pub fn save_settings(s: &Settings) {
    if let Some(p) = config_dir_file("settings.toml") {
        if let Some(dir) = p.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(t) = toml::to_string_pretty(s) {
            let _ = std::fs::write(p, t);
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct State {
    #[serde(default)]
    pub search: String,
    #[serde(default)]
    pub scroll: f32,
}

pub fn load_state() -> State {
    state_path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|t| toml::from_str(&t).ok())
        .unwrap_or_default()
}

pub fn save_state(s: &State) {
    if let Some(p) = state_path() {
        if let Ok(t) = toml::to_string(s) {
            let _ = std::fs::write(p, t);
        }
    }
}
