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

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct CustomShortcut {
    pub label: String,
    pub keys: String,
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
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

#[derive(Debug, Default, Deserialize)]
struct CustomConfig {
    #[serde(default)]
    shortcut: Vec<CustomShortcut>,
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
