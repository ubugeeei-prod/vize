//! File discovery shared by formatting and feature-independent story migration.

#[path = "fmt/files.rs"]
pub(super) mod files;
#[path = "fmt/ignores.rs"]
pub(super) mod ignores;
#[path = "fmt/patterns.rs"]
pub(super) mod patterns;

pub(super) use files::collect_files;
