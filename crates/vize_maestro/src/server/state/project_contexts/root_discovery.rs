//! Fresh project-marker discovery. Source operation counts compile out of
//! ordinary builds; profiling retains the exact traversal and marker policy.

use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

#[cfg(target_os = "linux")]
use dashmap::DashMap;
#[cfg(target_os = "linux")]
use std::{fs::Metadata, time::SystemTime};

#[cfg(target_os = "linux")]
const MARKER_CACHE_CAPACITY: usize = 65_536;

/// Linux directory entries are rescanned only after the kernel reports a name
/// change. Platforms or paths without a reliable kernel generation retain the
/// uncached scan, independently of client watcher support.
#[derive(Default)]
pub(super) struct DirectoryMarkerCache {
    #[cfg(target_os = "linux")]
    entries: DashMap<PathBuf, CachedDirectoryMarkers>,
    #[cfg(target_os = "linux")]
    changes: linux::DirectoryChangeTracker,
    #[cfg(all(test, target_os = "linux"))]
    scans: std::sync::atomic::AtomicUsize,
}

#[cfg(target_os = "linux")]
#[derive(Clone)]
struct CachedDirectoryMarkers {
    identity: DirectoryIdentity,
    generation: linux::DirectoryGeneration,
    markers: DirectoryMarkers,
}

#[cfg(target_os = "linux")]
#[derive(Clone, Debug, Eq, PartialEq)]
struct DirectoryIdentity {
    len: u64,
    modified: Option<SystemTime>,
    #[cfg(unix)]
    device: u64,
    #[cfg(unix)]
    inode: u64,
    #[cfg(unix)]
    changed_seconds: i64,
    #[cfg(unix)]
    changed_nanoseconds: i64,
    #[cfg(windows)]
    attributes: u32,
    #[cfg(windows)]
    created: u64,
    #[cfg(windows)]
    written: u64,
}

#[cfg(target_os = "linux")]
impl DirectoryIdentity {
    fn capture(metadata: &Metadata) -> Self {
        #[cfg(unix)]
        use std::os::unix::fs::MetadataExt;
        #[cfg(windows)]
        use std::os::windows::fs::MetadataExt;

        Self {
            len: metadata.len(),
            modified: metadata.modified().ok(),
            #[cfg(unix)]
            device: metadata.dev(),
            #[cfg(unix)]
            inode: metadata.ino(),
            #[cfg(unix)]
            changed_seconds: metadata.ctime(),
            #[cfg(unix)]
            changed_nanoseconds: metadata.ctime_nsec(),
            #[cfg(windows)]
            attributes: metadata.file_attributes(),
            #[cfg(windows)]
            created: metadata.creation_time(),
            #[cfg(windows)]
            written: metadata.last_write_time(),
        }
    }
}

impl DirectoryMarkerCache {
    fn markers(&self, directory: &Path) -> DirectoryMarkers {
        #[cfg(not(target_os = "linux"))]
        {
            self.record_scan();
            uncached_directory_markers(directory).unwrap_or_default()
        }

        #[cfg(target_os = "linux")]
        {
            self.linux_markers(directory)
        }
    }

    #[cfg(target_os = "linux")]
    fn linux_markers(&self, directory: &Path) -> DirectoryMarkers {
        let Some(mut generation) = self.changes.generation(directory) else {
            self.record_scan();
            return uncached_directory_markers(directory).unwrap_or_default();
        };
        let Ok(metadata) = directory.metadata() else {
            return uncached_directory_markers(directory).unwrap_or_default();
        };
        vize_l0::micro_count!("fs.directory_metadata.calls", 1);
        let mut identity = DirectoryIdentity::capture(&metadata);
        if let Some(cached) = self.entries.get(directory)
            && cached.identity == identity
            && cached.generation == generation
        {
            vize_l0::micro_count!("fs.directory_marker_cache.hits", 1);
            return cached.markers;
        }

        vize_l0::micro_count!("fs.directory_marker_cache.misses", 1);
        // A scan racing a directory-entry mutation cannot certify either
        // generation. Retry the new generation once, then return a fresh
        // uncached observation rather than retaining an ambiguous result.
        for _ in 0..2 {
            self.record_scan();
            let Ok(markers) = uncached_directory_markers(directory) else {
                return DirectoryMarkers::default();
            };
            let Ok(after) = directory.metadata() else {
                return DirectoryMarkers::default();
            };
            vize_l0::micro_count!("fs.directory_metadata.calls", 1);
            let after = DirectoryIdentity::capture(&after);
            let Some(after_generation) = self.changes.generation(directory) else {
                return markers;
            };
            if identity == after && generation == after_generation {
                if self.entries.len() >= MARKER_CACHE_CAPACITY {
                    self.entries.clear();
                    self.changes.reset();
                    return markers;
                }
                self.entries.insert(
                    directory.to_path_buf(),
                    CachedDirectoryMarkers {
                        identity,
                        generation,
                        markers,
                    },
                );
                return markers;
            }
            identity = after;
            generation = after_generation;
        }
        self.record_scan();
        uncached_directory_markers(directory).unwrap_or_default()
    }

    fn record_scan(&self) {
        #[cfg(all(test, target_os = "linux"))]
        self.scans
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    #[cfg(all(test, target_os = "linux"))]
    fn scans(&self) -> usize {
        self.scans.load(std::sync::atomic::Ordering::Relaxed)
    }
}

pub(super) fn document_root(path: &Path, workspace: &Path) -> PathBuf {
    document_root_with_markers(path, workspace, None).to_path_buf()
}

pub(super) fn cached_document_root(
    path: &Path,
    workspace: &Path,
    markers: &DirectoryMarkerCache,
) -> PathBuf {
    document_root_with_markers(path, workspace, Some(markers)).to_path_buf()
}

fn document_root_with_markers<'a>(
    path: &'a Path,
    workspace: &'a Path,
    cache: Option<&DirectoryMarkerCache>,
) -> &'a Path {
    vize_l0::micro_span!("maestro.project.document_root", {
        let mut directory = path.parent().unwrap_or(workspace);
        let mut package_root = None;
        let mut vite_root = None;
        while directory.starts_with(workspace) {
            vize_l0::micro_count!("fs.directory_scan.calls", 1);
            let markers = cache.map_or_else(
                || uncached_directory_markers(directory).unwrap_or_default(),
                |cache| cache.markers(directory),
            );
            if markers.typescript {
                if package_root.is_some()
                    && let Some(vite_root) = vite_root
                {
                    return vite_root;
                }
                return directory;
            }
            if vite_root.is_none() && package_root.is_none() && markers.vite {
                vite_root = Some(directory);
            }
            if package_root.is_none() && markers.package {
                package_root = Some(directory);
            }
            if directory == workspace {
                break;
            }
            let Some(parent) = directory.parent() else {
                break;
            };
            directory = parent;
        }
        vite_root.or(package_root).unwrap_or(workspace)
    })
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct DirectoryMarkers {
    package: bool,
    vite: bool,
    typescript: bool,
}

impl DirectoryMarkers {
    // A TypeScript marker decides this directory's ownership independently of
    // its other markers. Normalize those irrelevant flags for either reader.
    fn observe(&mut self, name: &OsStr) -> bool {
        match config_marker_name(name) {
            Some(ConfigMarker::Package) => self.package = true,
            Some(ConfigMarker::Vite) => self.vite = true,
            Some(ConfigMarker::TypeScript) => {
                *self = Self {
                    typescript: true,
                    ..Self::default()
                };
                return true;
            }
            None => {}
        }
        false
    }
}

fn uncached_directory_markers(directory: &Path) -> std::io::Result<DirectoryMarkers> {
    #[cfg(target_os = "linux")]
    if let Ok(markers) = linux::directory_markers(directory) {
        return Ok(markers);
    }
    standard_directory_markers(directory)
}

fn standard_directory_markers(directory: &Path) -> std::io::Result<DirectoryMarkers> {
    vize_l0::micro_count!("fs.read_dir.calls", 1);
    let mut markers = DirectoryMarkers::default();
    for entry in std::fs::read_dir(directory)?.flatten() {
        vize_l0::micro_count!("fs.read_dir.entries", 1);
        if markers.observe(&entry.file_name()) {
            break;
        }
    }
    Ok(markers)
}

#[cfg(target_os = "linux")]
mod linux;

#[derive(Debug, Eq, PartialEq)]
enum ConfigMarker {
    Package,
    Vite,
    TypeScript,
}

// Most entries are source files. Reject them using borrowed OS bytes, without
// constructing a full path or validating their names as UTF8. Candidate marker
// names retain the previous UTF8 requirement, including non-UTF8 suffixes.
fn config_marker_name(name: &OsStr) -> Option<ConfigMarker> {
    let bytes = name.as_encoded_bytes();
    if bytes == b"package.json" {
        return Some(ConfigMarker::Package);
    }
    if bytes == b"jsconfig.json" {
        return Some(ConfigMarker::TypeScript);
    }
    let vite = bytes.starts_with(b"vite.config.");
    let typescript = bytes.starts_with(b"tsconfig") && bytes.ends_with(b".json");
    if !(vite || typescript) || name.to_str().is_none() {
        return None;
    }
    Some(if vite {
        ConfigMarker::Vite
    } else {
        ConfigMarker::TypeScript
    })
}

pub(super) fn is_config_marker(path: &Path) -> bool {
    path.file_name().and_then(config_marker_name).is_some()
}

#[cfg(test)]
mod tests;
