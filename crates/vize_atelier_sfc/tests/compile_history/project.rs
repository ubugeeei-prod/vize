//! Temporary projects for multi-file compile-history cases.

use std::path::Path;

use vize_l0::String;

use super::Case;

/// Write the case's project files and its own source into a fresh directory.
/// A project without its own `tsconfig.json` gets an empty one, so config
/// discovery stops at the project root instead of probing the machine's
/// temporary-directory ancestors.
pub(super) fn write_project(case: &Case) -> tempfile::TempDir {
    let project = tempfile::tempdir().expect("project directory");
    let tsconfig: (String, String) = ("tsconfig.json".into(), "{}\n".into());
    let default_config = (!case.files.contains_key("tsconfig.json")
        && !case.options.no_project_config.unwrap_or(false))
    .then_some((&tsconfig.0, &tsconfig.1));
    let files = case
        .files
        .iter()
        .chain(default_config)
        .chain([(&case.filename, &case.source)]);
    for (relative, text) in files {
        let path = project.path().join(relative.as_str());
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("project directory tree");
        }
        std::fs::write(&path, text.as_bytes()).expect("project file");
    }
    project
}

/// Replace the temporary project root, in both its given and canonical
/// spellings, with `$PROJECT`.
pub(super) fn relocate(bytes: std::string::String, root: Option<&Path>) -> std::string::String {
    let Some(root) = root else {
        return bytes;
    };
    let canonical = root.canonicalize().expect("canonical project root");
    let mut bytes = bytes;
    for spelling in [canonical.as_path(), root] {
        bytes = bytes.replace(spelling.to_string_lossy().as_ref(), "$PROJECT");
    }
    // Dependency probes outside the project (absolute paths that are not
    // `$PROJECT/...`) name the machine's directories, never the case's.
    bytes
        .split_inclusive('\n')
        .filter(|line| {
            let line = line.trim();
            !(line.starts_with("\"/") && line.ends_with("\","))
        })
        .collect()
}
