// SPDX-License-Identifier: GPL-3.0-only
// Copyright (c) 2026 Tomas Haaland
//! Fluent-based UI translations (Pop/COSMIC style).
//!
//! Message ids in `.ftl` use hyphens (`ui-title`); callers may pass either
//! `ui-title` or the legacy dotted form `ui.title`.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use i18n_embed::{
    DefaultLocalizer, LanguageLoader, Localizer,
    fluent::{FluentLanguageLoader, fluent_language_loader},
};
use rust_embed::RustEmbed;
use unic_langid::LanguageIdentifier;

#[derive(RustEmbed)]
#[folder = "i18n/"]
struct Localizations;

static LANGUAGE_LOADER: OnceLock<FluentLanguageLoader> = OnceLock::new();
static CURRENT: Mutex<&'static str> = Mutex::new("eng");

fn loader() -> &'static FluentLanguageLoader {
    LANGUAGE_LOADER.get_or_init(|| {
        let loader: FluentLanguageLoader = fluent_language_loader!();
        loader
            .load_fallback_language(&Localizations)
            .expect("failed to load fallback language (en)");
        loader
    })
}

/// Language codes used in settings / the language picker (legacy clip-suite ids).
pub const LANGS: &[&str] = &["nor", "eng", "nld", "fra", "spa", "deu", "swe", "dan", "ita"];

/// Native display names for the language picker.
pub const LANG_NAMES: &[&str] = &[
    "Norsk",
    "English",
    "Nederlands",
    "Français",
    "Español",
    "Deutsch",
    "Svenska",
    "Dansk",
    "Italiano",
];

fn our_to_langid(code: &str) -> Option<LanguageIdentifier> {
    let tag = match code {
        "eng" => "en",
        "nor" => "nb",
        "nld" => "nl",
        "fra" => "fr",
        "spa" => "es",
        "deu" => "de",
        "swe" => "sv",
        "dan" => "da",
        "ita" => "it",
        _ => return None,
    };
    tag.parse().ok()
}

fn locale_to_our(code: &str) -> Option<&'static str> {
    Some(match code {
        "nb" | "nn" | "no" => "nor",
        "en" => "eng",
        "nl" => "nld",
        "fr" => "fra",
        "es" => "spa",
        "de" => "deu",
        "sv" => "swe",
        "da" => "dan",
        "it" => "ita",
        _ => return None,
    })
}

fn parse_our(raw: &str) -> Option<&'static str> {
    let code = raw.trim();
    LANGS.iter().copied().find(|l| *l == code)
}

fn lang_from_locale() -> &'static str {
    for var in ["LC_ALL", "LC_MESSAGES", "LANG", "LANGUAGE"] {
        if let Ok(v) = std::env::var(var) {
            if v.is_empty() {
                continue;
            }
            let code = v
                .split(':')
                .next()
                .unwrap_or("")
                .split('.')
                .next()
                .unwrap_or("")
                .split('_')
                .next()
                .unwrap_or("")
                .to_lowercase();
            if let Some(l) = locale_to_our(&code) {
                return l;
            }
        }
    }
    "eng"
}

/// Legacy path shared with the GTK cheat sheet (read as fallback only).
fn legacy_ui_lang_path() -> Option<std::path::PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(std::path::Path::new(&home).join(".config/cosmic-clip/ui-lang"))
}

fn apply_lang(code: &'static str) {
    if let Some(id) = our_to_langid(code) {
        let localizer = DefaultLocalizer::new(loader(), &Localizations);
        if let Err(e) = localizer.select(&[id]) {
            log::warn!("failed to select UI language {code}: {e}");
        }
    }
    if let Ok(mut g) = CURRENT.lock() {
        *g = code;
    }
}

/// Initialise Fluent from settings / legacy file / system locale.
pub fn init(preferred: Option<&str>) {
    let _ = loader();
    let code = preferred
        .and_then(parse_our)
        .or_else(|| {
            legacy_ui_lang_path()
                .and_then(|p| std::fs::read_to_string(p).ok())
                .and_then(|t| parse_our(&t).map(|s| s))
        })
        .unwrap_or_else(lang_from_locale);
    apply_lang(code);
}

/// Current UI language code (`nor`, `eng`, …).
pub fn current_lang() -> &'static str {
    CURRENT.lock().map(|g| *g).unwrap_or("eng")
}

pub fn lang_index(lang: &str) -> Option<usize> {
    LANGS.iter().position(|l| *l == lang)
}

/// Switch UI language at runtime (Fluent select). Caller persists settings.
pub fn set_lang(code: &str) -> bool {
    let Some(code) = parse_our(code) else {
        return false;
    };
    apply_lang(code);
    true
}

fn fluent_id(key: &str) -> String {
    key.replace('.', "-")
}

/// Resolve a UI string; falls back to the key itself if missing.
pub fn tr(key: &str) -> String {
    let id = fluent_id(key);
    let loader = loader();
    if loader.has(&id) {
        loader.get(&id)
    } else if key != id && loader.has(key) {
        loader.get(key)
    } else {
        key.to_string()
    }
}

/// Resolve a string with a single `{ $arg }` placeholder.
pub fn tr_arg(key: &str, arg: &str) -> String {
    let id = fluent_id(key);
    let loader = loader();
    if !loader.has(&id) {
        return tr(key).replace("{}", arg);
    }
    let mut args = HashMap::new();
    args.insert("arg", arg);
    loader.get_args(&id, args)
}
