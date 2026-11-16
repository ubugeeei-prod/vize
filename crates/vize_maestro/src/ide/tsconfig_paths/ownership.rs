//! The referenced project that owns a source (#13090).
//!
//! A solution-style `tsconfig.json` gives each source the options of the
//! referenced project whose effective `files` / `include` / `exclude` admits
//! it, exactly as the compiler, linter and checker select it through the
//! shared `vize_l0::typescript_ownership` authority.

use std::path::{Path, PathBuf};

use serde_json::Value;
use vize_l0::typescript_ownership::{ProjectReferenceGraph, TsconfigSourceKind, normalize_path};
use vize_l0::typescript_project::{CompilerConfigHost, parse_jsonc_value};

/// Direct reads of the user's configs; an unreadable config owns nothing.
struct FileHost;

impl CompilerConfigHost for FileHost {
    type Error = ();

    fn read_jsonc(&self, path: &Path) -> Result<Value, ()> {
        let source = std::fs::read_to_string(path).map_err(|_| ())?;
        parse_jsonc_value(&source).map_err(|_| ())
    }

    fn configuration_error(&self, _message: vize_l0::String) {}
}

/// The unique referenced project of `shell` that owns `source`, spelled under
/// the shell's directory as the editor spells it. `None` when `shell` has no
/// references, or when no project or several projects admit the source.
pub(super) fn owning_reference(shell: &Path, source: &Path) -> Option<PathBuf> {
    let graph = ProjectReferenceGraph::load(shell, &FileHost);
    if !graph.has_references() {
        return None;
    }
    let source = normalize_path(source);
    let owner = graph.select_normalized(&source, TsconfigSourceKind::for_path(&source))?;
    if graph.projects().first() == Some(&owner) {
        return None;
    }
    let shell_dir = shell.parent()?;
    Some(
        owner
            .strip_prefix(normalize_path(shell_dir))
            .map_or_else(|_| owner.clone(), |relative| shell_dir.join(relative)),
    )
}
