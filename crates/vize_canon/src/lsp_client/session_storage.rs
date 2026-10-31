//! Session-owned writable mirrors, separate from authored projects and packages.

use super::paths::find_node_modules_with_vue;
use std::{
    fs,
    io::{ErrorKind, Write},
    path::{Component, Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};
use vize_l0::{String, cstr};

pub(super) struct SessionStorage {
    root: PathBuf,
    directory: Option<tempfile::TempDir>,
    retain_after_failure: AtomicBool,
}

impl SessionStorage {
    pub(super) fn new(project_root: &Path) -> Result<Self, String> {
        let base = std::env::temp_dir()
            .canonicalize()
            .map_err(|error| cstr!("Failed to resolve private Corsa storage base: {error}"))?;
        let authored = project_root
            .canonicalize()
            .unwrap_or_else(|_| project_root.to_path_buf());
        if base.starts_with(&authored)
            || base.components().any(|part| {
                part.as_os_str()
                    .to_string_lossy()
                    .eq_ignore_ascii_case("node_modules")
            })
        {
            return Err(cstr!(
                "Private Corsa storage cannot be inside the authored or package tree"
            ));
        }
        let directory = tempfile::Builder::new()
            .prefix("vize-corsa-overlay-")
            .tempdir_in(base)
            .map_err(|error| cstr!("Failed to create private Corsa storage: {error}"))?;
        let root = directory.path().join("project");
        // Document URIs use the normal Windows spelling, without a verbatim
        // prefix. Keep the owned boundary in that same physical spelling so
        // strict component checks accept only its exact URI-derived members.
        #[cfg(windows)]
        let root = crate::file_uri::file_uri_to_path(&crate::file_uri::path_to_file_uri(&root))
            .ok_or_else(|| cstr!("Failed to derive private Corsa mirror path"))?;
        fs::create_dir(&root)
            .map_err(|error| cstr!("Failed to create private Corsa mirror: {error}"))?;
        if let Some(source) = find_node_modules_with_vue(project_root) {
            // Package lookup may ascend out of the writable mirror. No mapped
            // document can target this sibling or write through its link.
            let target = directory.path().join("node_modules");
            link_dependencies(&source, &target)
                .map_err(|error| cstr!("Failed to link private Corsa dependencies: {error}"))?;
        }
        Ok(Self {
            root,
            directory: Some(directory),
            retain_after_failure: AtomicBool::new(false),
        })
    }

    pub(super) fn root(&self) -> &Path {
        &self.root
    }

    pub(super) fn observe_retirement(&self, result: &Result<(), String>) {
        if result.is_err() {
            self.retain_after_failed_shutdown();
        }
    }

    pub(super) fn retain_after_failed_shutdown(&self) {
        self.retain_after_failure.store(true, Ordering::Release);
    }
}

impl Drop for SessionStorage {
    fn drop(&mut self) {
        if self.retain_after_failure.load(Ordering::Acquire)
            && let Some(directory) = self.directory.take()
        {
            let retained = directory.keep();
            tracing::warn!(path = %retained.display(), "retaining private Corsa storage after incomplete shutdown");
        }
    }
}

fn link_dependencies(source: &Path, target: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(source, target)
    }
    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_dir(source, target)
            .or_else(|_| junction::create(source, target))
    }
}

pub(super) fn check_owned_destination(project_root: &Path, path: &Path) -> Result<bool, String> {
    // Constructors retain a canonical root. Refuse a later replacement of
    // that boundary before inspecting any child through a foreign tree.
    let root = fs::symlink_metadata(project_root)
        .map_err(|error| cstr!("Failed to inspect closed Corsa project root: {error}"))?;
    if !root.is_dir()
        || root.file_type().is_symlink()
        || (cfg!(windows) && fs::read_link(project_root).is_ok())
    {
        return Err(cstr!(
            "Closed Corsa project root is not an owned real directory"
        ));
    }
    let relative = path
        .strip_prefix(project_root)
        .map_err(|_| cstr!("Closed Corsa document is outside its project"))?;
    let mut current = project_root.to_path_buf();
    for component in relative.components() {
        if !matches!(component, Component::Normal(_)) {
            return Err(cstr!("Invalid closed Corsa document path"));
        }
        current.push(component);
        let metadata = match fs::symlink_metadata(&current) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(false),
            Err(error) => return Err(cstr!("Failed to inspect closed Corsa document: {error}")),
        };
        if metadata.file_type().is_symlink() || (cfg!(windows) && fs::read_link(&current).is_ok()) {
            return Err(cstr!("Closed Corsa document crosses a linked mirror path"));
        }
        if current == path {
            if !metadata.is_file() {
                return Err(cstr!("Closed Corsa document mirror is not a file"));
            }
            return Ok(true);
        }
        if !metadata.is_dir() {
            return Err(cstr!(
                "Closed Corsa document mirror parent is not a directory"
            ));
        }
    }
    Err(cstr!(
        "Closed Corsa document cannot replace its project root"
    ))
}

pub(super) fn check_project_root(root: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(root)
        .map_err(|error| cstr!("Failed to inspect authored Corsa project: {error}"))?;
    if !metadata.is_dir()
        || metadata.file_type().is_symlink()
        || (cfg!(windows) && fs::read_link(root).is_ok())
    {
        return Err(cstr!("Authored Corsa project is not a real directory"));
    }
    Ok(())
}

pub(super) fn write_owned_file(root: &Path, path: &Path, content: &[u8]) -> Result<bool, String> {
    if !path.strip_prefix(root).is_ok_and(|relative| {
        !relative.as_os_str().is_empty()
            && relative
                .components()
                .all(|part| matches!(part, Component::Normal(_)))
    }) {
        return Err(cstr!("Corsa document is outside its private mirror"));
    }
    let owner = root
        .parent()
        .ok_or_else(|| cstr!("Missing private Corsa storage owner"))?;
    let existed = check_owned_destination(owner, path)?;
    let parent = path
        .parent()
        .ok_or_else(|| cstr!("Missing private Corsa document parent"))?;
    fs::create_dir_all(parent)
        .map_err(|error| cstr!("Failed to prepare private Corsa document: {error}"))?;
    check_owned_destination(owner, path)?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent)
        .map_err(|error| cstr!("Failed to prepare private Corsa document: {error}"))?;
    temporary
        .write_all(content)
        .and_then(|()| {
            temporary
                .persist(path)
                .map(|_| ())
                .map_err(|error| error.error)
        })
        .map_err(|error| cstr!("Failed to write private Corsa document: {error}"))?;
    Ok(existed)
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod native_tests;
