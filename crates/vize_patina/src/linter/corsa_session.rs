use corsa::api::ProjectSession;
use std::path::PathBuf;
use vize_l0::String;

mod boolean;
mod errors;
mod paths;
mod probe;
mod session;
mod virtual_documents;
mod vue_dependencies;

pub(in crate::linter) use paths::{
    effective_tsconfig_for_source, resolve_project_root, validate_tsconfig,
};

#[cfg(test)]
mod tests;

pub(super) type TypeProbe = corsa::api::TypeProbe;

/// Keep a rewritten module path inside its authored string delimiter.
pub(in crate::linter) fn escape_specifier(value: &str, quote: u8) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            '\u{2028}' => escaped.push_str("\\u2028"),
            '\u{2029}' => escaped.push_str("\\u2029"),
            character if character == char::from(quote) => {
                escaped.push('\\');
                escaped.push(character);
            }
            character if character < '\u{20}' => {
                escaped.push_str(&vize_l0::cstr!("\\u{:04x}", character as u32));
            }
            character => escaped.push(character),
        }
    }
    escaped
}

pub(crate) struct CorsaTypeAwareSession {
    session: ProjectSession,
    project_root: PathBuf,
    tsconfig_path: Option<PathBuf>,
    session_root: PathBuf,
    virtual_file_path: PathBuf,
    virtual_file_wire: String,
    supports_overlay_updates: bool,
    overlay_version: i32,
    dependency_paths: vize_l0::FxHashSet<PathBuf>,
    dependency_cache: vize_l0::FxHashMap<PathBuf, vue_dependencies::CachedDocument>,
    vue_path_aliases: vue_dependencies::PathAliases,
    active_filename: Option<String>,
    opened_documents: vize_l0::FxHashMap<String, virtual_documents::OpenedDocument>,
    published: virtual_documents::PublishedDocuments,
    #[cfg(test)]
    last_refresh: virtual_documents::RefreshCounts,
    closed: bool,
}

impl Drop for CorsaTypeAwareSession {
    fn drop(&mut self) {
        self.close();
    }
}

pub(super) fn remove_finished_process_sessions(files: &[std::path::PathBuf]) {
    let mut seen = vize_l0::FxHashSet::default();
    for file in files {
        let root = paths::resolve_project_root(file.to_string_lossy().as_ref());
        if seen.insert(root.clone()) {
            paths::remove_finished_process_sessions(&root);
        }
    }
}
