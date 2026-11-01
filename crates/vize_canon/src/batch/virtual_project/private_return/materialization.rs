//! Filesystem checks for one owner-selected private declaration, not resolver authority.

use std::collections::hash_map::Entry;
use std::fs::{self, Metadata, OpenOptions};
use std::io::{self, ErrorKind, Write};
use std::path::{Component, Path, PathBuf};

use vize_carton::{FxHashMap, String, profiler::global_profiler};

use crate::batch::materialize_fs::ensure_dir;

use super::HelperFile;

#[cfg(test)]
mod tests;

/// The owner already holds its storage/materialization locks. Never repair an
/// unknown existing leaf: a changed content-addressed body needs a new target.
pub(super) fn write_owned(helper: &HelperFile) -> io::Result<()> {
    let mut tree = CheckedTree::new(&helper.root)?;
    let stem = helper_stem(helper)?;
    let parent = tree.parent(&helper.path, true)?;
    tree.refuse_alternatives(&parent, stem)?;
    match fs::symlink_metadata(&helper.path) {
        Ok(_) => {
            verify_owned(helper, &[])?;
            let profiler = global_profiler();
            profiler.record_counter("io.write.skipped.calls", 1);
            profiler.record_counter("io.write.skipped.bytes", helper.body.len() as u64);
            return Ok(());
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }

    // Unlike write_if_changed, create_new cannot replace an unexpected leaf or
    // unlink a symlink. A failed write stays a refused, incomplete artifact.
    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&helper.path)?;
        file.write_all(helper.body.as_bytes())?;
        file.sync_all()
    })();
    let profiler = global_profiler();
    match &result {
        Ok(()) => profiler.record_fs_write(helper.body.len()),
        Err(_) => profiler.record_fs_write_failure(helper.body.len()),
    }
    result?;
    verify_owned(helper, &[])
}

/// Check materialized bytes and the finite `.js` extension candidate family.
/// The caller separately seals final import literals, configuration and inputs.
pub(super) fn verify_owned(helper: &HelperFile, importers: &[PathBuf]) -> io::Result<()> {
    let mut tree = CheckedTree::new(&helper.root)?;
    let stem = helper_stem(helper)?;
    let parent = tree.parent(&helper.path, false)?;
    tree.refuse_alternatives(&parent, stem)?;
    let metadata = regular_file(&helper.path)?;
    if metadata.len() != helper.body.len() as u64 {
        return Err(refuse("private return helper byte length changed"));
    }
    let bytes = fs::read(&helper.path);
    let profiler = global_profiler();
    profiler.record_counter("io.read.calls", 1);
    profiler.record_counter("syscall.fs.read.calls", 1);
    match &bytes {
        Ok(bytes) => profiler.record_counter("io.read.bytes", bytes.len() as u64),
        Err(_) => {
            profiler.record_counter("io.read.failures", 1);
            profiler.record_counter("syscall.fs.read.failures", 1);
        }
    }
    if bytes? != helper.body.as_bytes() {
        return Err(refuse("private return helper body changed"));
    }
    for importer in importers {
        tree.parent(importer, false)?;
        regular_file(importer)?;
    }
    Ok(())
}

fn refuse(message: &'static str) -> io::Error {
    io::Error::new(ErrorKind::InvalidData, message)
}

fn is_link(path: &Path, metadata: &Metadata) -> bool {
    metadata.file_type().is_symlink() || (cfg!(windows) && fs::read_link(path).is_ok())
}

fn regular_file(path: &Path) -> io::Result<Metadata> {
    let metadata = fs::symlink_metadata(path)?;
    if is_link(path, &metadata) || !metadata.is_file() {
        return Err(refuse("private return path is not a regular owned file"));
    }
    Ok(metadata)
}

fn relative_path<'a>(root: &Path, path: &'a Path) -> io::Result<&'a Path> {
    let relative = path
        .strip_prefix(root)
        .map_err(|_| refuse("private return path is outside its owned root"))?;
    if relative.as_os_str().is_empty()
        || relative.components().any(|component| match component {
            Component::Normal(name) => name.to_str().is_none_or(|name| !name.is_ascii()),
            _ => true,
        })
    {
        return Err(refuse("unsupported private return relative path"));
    }
    Ok(relative)
}

fn helper_stem(helper: &HelperFile) -> io::Result<&str> {
    relative_path(&helper.root, &helper.path)?;
    let stem = helper
        .path
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| name.strip_suffix(".d.ts"))
        .filter(|stem| !stem.is_empty() && !matches!(*stem, "." | ".."));
    stem.ok_or_else(|| refuse("private return helper needs an ASCII .d.ts basename"))
}

/// Inventories live only for this call. Shared importer directories are read
/// once; no directory observation survives into a later owner revision.
struct CheckedTree<'a> {
    root: &'a Path,
    names: FxHashMap<PathBuf, FxHashMap<String, String>>,
}

impl<'a> CheckedTree<'a> {
    fn new(root: &'a Path) -> io::Result<Self> {
        if !root.is_absolute() || root.parent().is_none() {
            return Err(refuse("private return needs an absolute owned directory"));
        }
        let mut current = PathBuf::new();
        for component in root.components() {
            match component {
                Component::RootDir | Component::Normal(_) => current.push(component),
                _ => return Err(refuse("unsupported private return root spelling")),
            }
            let metadata = fs::symlink_metadata(&current)?;
            if is_link(&current, &metadata) || !metadata.is_dir() {
                return Err(refuse(
                    "private return root has a non-directory or linked ancestor",
                ));
            }
        }
        Ok(Self {
            root,
            names: FxHashMap::default(),
        })
    }

    fn names(&mut self, directory: &Path) -> io::Result<&FxHashMap<String, String>> {
        match self.names.entry(directory.to_path_buf()) {
            Entry::Occupied(entry) => Ok(entry.into_mut()),
            Entry::Vacant(entry) => {
                let mut names = FxHashMap::default();
                for child in fs::read_dir(directory)? {
                    let name = child?.file_name();
                    let name = name
                        .to_str()
                        .filter(|name| name.is_ascii())
                        .ok_or_else(|| refuse("unsupported private return sibling spelling"))?;
                    let folded = String::from(name.to_ascii_lowercase());
                    if names.insert(folded, String::from(name)).is_some() {
                        return Err(refuse(
                            "private return directory has a case-folding collision",
                        ));
                    }
                }
                Ok(entry.insert(names))
            }
        }
    }

    fn exact_spelling(&mut self, parent: &Path, wanted: &str) -> io::Result<()> {
        let folded = String::from(wanted.to_ascii_lowercase());
        if self
            .names(parent)?
            .get(&folded)
            .is_some_and(|name| name.as_str() != wanted)
        {
            return Err(refuse("private return path has a case-folding collision"));
        }
        Ok(())
    }

    fn parent(&mut self, path: &Path, create: bool) -> io::Result<PathBuf> {
        let relative = relative_path(self.root, path)?;
        let mut components = relative.components().peekable();
        let mut current = self.root.to_path_buf();
        while let Some(Component::Normal(name)) = components.next() {
            let spelling = name
                .to_str()
                .ok_or_else(|| refuse("private return relative spelling changed"))?;
            self.exact_spelling(&current, spelling)?;
            if components.peek().is_none() {
                return Ok(current);
            }
            let parent = current.clone();
            current.push(name);
            match fs::symlink_metadata(&current) {
                Ok(metadata) if !is_link(&current, &metadata) && metadata.is_dir() => {}
                Ok(_) => return Err(refuse("private return parent is not an owned directory")),
                Err(error) if create && error.kind() == ErrorKind::NotFound => {
                    ensure_dir(&current)?;
                    let metadata = fs::symlink_metadata(&current)?;
                    if is_link(&current, &metadata) || !metadata.is_dir() {
                        return Err(refuse("private return created parent changed"));
                    }
                    self.names.remove(&parent);
                }
                Err(error) => return Err(error),
            }
        }
        Err(refuse("private return path has no file component"))
    }

    fn refuse_alternatives(&mut self, parent: &Path, stem: &str) -> io::Result<()> {
        let folded = String::from(stem.to_ascii_lowercase());
        let names = self.names(parent)?;
        for suffix in ["", ".ts", ".tsx", ".js", ".jsx"] {
            let name = vize_carton::cstr!("{folded}{suffix}");
            if names.contains_key(&name) {
                return Err(refuse(
                    "private return target has a competing file or directory",
                ));
            }
        }
        Ok(())
    }
}
