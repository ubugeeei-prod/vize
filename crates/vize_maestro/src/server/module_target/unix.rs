//! Each original directory/file stays open through fresh path traversal.
use super::{ModuleTargetError, TargetOperation, TargetPolicy, candidate};
use std::{
    ffi::{CString, OsStr},
    fs::File,
    os::{
        fd::{AsRawFd, FromRawFd, RawFd},
        unix::{ffi::OsStrExt, fs::MetadataExt},
    },
    path::{Component, Path, PathBuf},
};
use tower_lsp::lsp_types::Url;

#[derive(Clone, Copy, PartialEq, Eq)]
struct Identity {
    device: u64,
    inode: u64,
}
struct Handle {
    path: PathBuf,
    file: File,
    identity: Identity,
}
impl Handle {
    fn open(
        parent: RawFd,
        name: &OsStr,
        path: PathBuf,
        directory: bool,
    ) -> Result<Self, ModuleTargetError> {
        let name = CString::new(name.as_bytes())
            .map_err(|_| ModuleTargetError::Policy(TargetPolicy::Candidate))?;
        let flags = libc::O_RDONLY
            | libc::O_NOFOLLOW
            | libc::O_CLOEXEC
            | libc::O_NONBLOCK
            | if directory { libc::O_DIRECTORY } else { 0 };
        // SAFETY: the live parent FD and NUL-terminated name are retained for
        // this synchronous syscall. No descriptor is closed or reused here.
        let fd = unsafe { libc::openat(parent, name.as_ptr(), flags) };
        if fd < 0 {
            return Err(ModuleTargetError::Io {
                operation: TargetOperation::Open,
                path,
                error: std::io::Error::last_os_error(),
            });
        }
        // SAFETY: openat returned a fresh owned descriptor >= 0. This File
        // takes its sole ownership and closes it on every subsequent failure.
        let file = unsafe { File::from_raw_fd(fd) };
        let metadata = file.metadata().map_err(|error| ModuleTargetError::Io {
            operation: TargetOperation::Metadata,
            path: path.clone(),
            error,
        })?;
        if directory && !metadata.is_dir() {
            return Err(ModuleTargetError::Policy(TargetPolicy::NonDirectory));
        }
        if !directory && !metadata.is_file() {
            return Err(ModuleTargetError::Policy(TargetPolicy::NonRegular));
        }
        Ok(Self {
            path,
            file,
            identity: Identity {
                device: metadata.dev(),
                inode: metadata.ino(),
            },
        })
    }
}

pub(crate) struct PhysicalTargets {
    root: Vec<Handle>,
    chains: Vec<Vec<Handle>>,
    source: Url,
    candidates: Vec<vize_l0::String>,
    uris: Vec<Url>,
}
impl PhysicalTargets {
    pub(crate) fn observe(
        root: &Path,
        source: &Url,
        values: &[&str],
    ) -> Result<Self, ModuleTargetError> {
        if values.is_empty() {
            return Err(ModuleTargetError::Policy(TargetPolicy::EmptyBatch));
        }
        let root = root_chain(root)?;
        let Some(anchor) = root.last() else {
            return Err(ModuleTargetError::Policy(TargetPolicy::Root));
        };
        if source.scheme() != "file"
            || source.query().is_some()
            || source.fragment().is_some()
            || !matches!(source.host_str(), None | Some("localhost"))
        {
            return Err(ModuleTargetError::Policy(TargetPolicy::SourceUri));
        }
        let path = source
            .to_file_path()
            .map_err(|_| ModuleTargetError::Policy(TargetPolicy::SourceUri))?;
        let parent = path
            .parent()
            .ok_or(ModuleTargetError::Policy(TargetPolicy::SourceParent))?;
        let parent = parent
            .strip_prefix(&anchor.path)
            .map_err(|_| ModuleTargetError::Policy(TargetPolicy::SourceParent))?;
        let mut chains = Vec::with_capacity(values.len());
        let mut uris = Vec::with_capacity(values.len());
        let mut candidates = Vec::with_capacity(values.len());
        for value in values {
            let relative = candidate(value)?;
            let mut chain = Vec::new();
            let mut directory = anchor;
            for part in parent.components() {
                let Component::Normal(name) = part else {
                    return Err(ModuleTargetError::Policy(TargetPolicy::SourceParent));
                };
                let handle = Handle::open(
                    directory.file.as_raw_fd(),
                    name,
                    directory.path.join(name),
                    true,
                )?;
                chain.push(handle);
                directory = chain
                    .last()
                    .ok_or(ModuleTargetError::Policy(TargetPolicy::SourceParent))?;
            }
            let mut parts = Path::new(relative).components().peekable();
            while let Some(part) = parts.next() {
                let Component::Normal(name) = part else {
                    return Err(ModuleTargetError::Policy(TargetPolicy::Candidate));
                };
                let is_directory = parts.peek().is_some();
                let handle = Handle::open(
                    directory.file.as_raw_fd(),
                    name,
                    directory.path.join(name),
                    is_directory,
                )?;
                chain.push(handle);
                directory = chain
                    .last()
                    .ok_or(ModuleTargetError::Policy(TargetPolicy::Candidate))?;
            }
            uris.push(
                Url::from_file_path(&directory.path)
                    .map_err(|_| ModuleTargetError::Policy(TargetPolicy::Uri))?,
            );
            chains.push(chain);
            candidates.push((*value).into());
        }
        Ok(Self {
            root,
            chains,
            source: source.clone(),
            candidates,
            uris,
        })
    }

    /// Freshly reach every pathname component. Old handles prevent inode reuse.
    /// No promise concerns mutations after the final successful observation.
    pub(crate) fn recheck(&self) -> Result<Self, ModuleTargetError> {
        let root = self
            .root
            .last()
            .ok_or(ModuleTargetError::Policy(TargetPolicy::Root))?;
        let values: Vec<&str> = self.candidates.iter().map(|value| value.as_str()).collect();
        let fresh = Self::observe(&root.path, &self.source, &values)?;
        same_chain(&self.root, &fresh.root)?;
        if self.chains.len() != fresh.chains.len() {
            return Err(ModuleTargetError::Policy(TargetPolicy::Candidate));
        }
        for (original, current) in self.chains.iter().zip(&fresh.chains) {
            same_chain(original, current)?;
        }
        Ok(fresh)
    }
    pub(crate) fn uris(&self) -> &[Url] {
        &self.uris
    }
}

fn root_chain(path: &Path) -> Result<Vec<Handle>, ModuleTargetError> {
    if !path.is_absolute() {
        return Err(ModuleTargetError::Policy(TargetPolicy::Root));
    }
    let root = Handle::open(libc::AT_FDCWD, OsStr::new("/"), PathBuf::from("/"), true)?;
    let mut chain = vec![root];
    for component in path.components() {
        let name = match component {
            Component::RootDir => continue,
            Component::Normal(name) => name,
            _ => return Err(ModuleTargetError::Policy(TargetPolicy::Root)),
        };
        let parent = chain
            .last()
            .ok_or(ModuleTargetError::Policy(TargetPolicy::Root))?;
        let handle = Handle::open(parent.file.as_raw_fd(), name, parent.path.join(name), true)?;
        chain.push(handle);
    }
    Ok(chain)
}
fn same_chain(original: &[Handle], current: &[Handle]) -> Result<(), ModuleTargetError> {
    if original.len() != current.len() {
        return Err(ModuleTargetError::Policy(TargetPolicy::Candidate));
    }
    for (original, current) in original.iter().zip(current) {
        if original.path != current.path || original.identity != current.identity {
            return Err(ModuleTargetError::Changed(original.path.clone()));
        }
    }
    Ok(())
}
