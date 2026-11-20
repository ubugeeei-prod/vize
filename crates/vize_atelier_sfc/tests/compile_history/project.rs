//! Temporary projects for multi-file compile-history cases.

use std::path::{Path, PathBuf};

use vize_l0::String;

use crate::Case;

/// A fixture project nested below a controlled ancestor config.
pub(crate) struct ProjectDir {
    _ancestor: tempfile::TempDir,
    path: PathBuf,
}

impl ProjectDir {
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

/// Write the case below a controlled ancestor config. Ordinary projects get a
/// root config; `noProjectConfig` cases exercise explicit absent authority.
pub(crate) fn write_project(case: &Case) -> ProjectDir {
    let ancestor = tempfile::tempdir().expect("project ancestor");
    // This would change no-config cases if they performed ordinary discovery.
    // Their explicit absent-project authority must make the result independent
    // of configs owned by the host above the fixture root.
    std::fs::write(ancestor.path().join("tsconfig.json"), "{}\n").expect("ancestor project config");
    let project = ancestor.path().join("project");
    std::fs::create_dir(&project).expect("project directory");
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
        let path = project.join(relative.as_str());
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("project directory tree");
        }
        std::fs::write(&path, text.as_bytes()).expect("project file");
    }
    ProjectDir {
        _ancestor: ancestor,
        path: project,
    }
}

/// Replace the temporary project root, in both its given and canonical
/// spellings, with `$PROJECT`.
pub(crate) fn relocate(bytes: std::string::String, root: Option<&Path>) -> std::string::String {
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
