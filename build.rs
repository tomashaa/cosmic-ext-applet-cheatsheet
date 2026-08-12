// SPDX-License-Identifier: GPL-3.0-only
// Copyright (c) 2026 Tomas Haaland

//! Make cargo notice translation changes.
//!
//! `rust_embed` pulls `i18n/` into the binary through a proc macro, and a proc
//! macro cannot tell cargo which files it read. Without this, editing a `.ftl`
//! leaves every input cargo *does* track unchanged, so the build is a no-op and
//! you silently ship the previous strings — the binary looks fresh, and the old
//! text is still in it. Only touching a `.rs` file would rebuild.
//!
//! Cargo scans a directory path recursively, so this covers every locale.

fn main() {
    println!("cargo:rerun-if-changed=i18n");
    // Without this, adding the line above would itself be the only tracked
    // change and cargo would fall back to rerunning on *any* file change.
    println!("cargo:rerun-if-changed=build.rs");
}
