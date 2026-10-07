use corsa::api::ProjectSession;
use std::path::PathBuf;
use vize_l0::String;

mod boolean;
mod errors;
mod paths;
mod probe;
mod session;
mod vue_dependencies;

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
    session_root: PathBuf,
    virtual_file_path: PathBuf,
    virtual_file_wire: String,
    supports_overlay_updates: bool,
    overlay_version: i32,
    rewritten_source: Option<String>,
    import_source_map: vize_canon::ImportSourceMap,
    dependency_paths: Vec<PathBuf>,
    dependency_cache: vize_l0::FxHashMap<PathBuf, vue_dependencies::CachedDocument>,
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
