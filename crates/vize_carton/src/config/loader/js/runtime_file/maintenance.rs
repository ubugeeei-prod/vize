use std::{
    fs::{self, File},
    path::Path,
    time::UNIX_EPOCH,
};

use super::{CACHE_LIMIT, private_open_options};

pub(super) fn maintain_cache(root: &Path, current: &Path, catalog: &File) {
    // Never wait for the catalog while holding a runtime lease: a constructor
    // may hold the catalog shared while it waits for this runtime exclusively.
    if catalog.try_lock().is_err() {
        return;
    }
    recover_interrupted_writes_locked(root);
    prune_cache_locked(root, current, CACHE_LIMIT);
    let _ = catalog.unlock();
}

pub(super) fn reserve_cache_capacity(root: &Path, current: &Path, catalog: &File) {
    if catalog.try_lock().is_err() {
        return;
    }
    // The target lease is already open but its runtime is not published yet.
    // Recovery would mistake that stable lease for an orphan and unlink it,
    // splitting later constructors onto a different file identity.
    prune_cache_locked(root, current, CACHE_LIMIT.saturating_sub(1));
    let _ = catalog.unlock();
}

fn recover_interrupted_writes_locked(root: &Path) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    let entries: Vec<_> = entries.flatten().collect();
    for entry in &entries {
        let path = entry.path();
        if !entry.file_name().to_string_lossy().ends_with(".tmp") {
            continue;
        }
        let Ok(metadata) = fs::symlink_metadata(&path) else {
            continue;
        };
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            continue;
        }
        let Ok(file) = private_open_options().open(&path) else {
            continue;
        };
        if file.try_lock().is_err() {
            continue;
        }
        if file.unlock().is_err() {
            continue;
        }
        drop(file);
        let _ = fs::remove_file(path);
    }
    for entry in entries {
        let path = entry.path();
        if path
            .extension()
            .is_none_or(|extension| extension != "lease")
        {
            continue;
        }
        let runtime = path.with_extension("mjs");
        if fs::symlink_metadata(&runtime)
            .is_ok_and(|metadata| metadata.is_file() && !metadata.file_type().is_symlink())
        {
            continue;
        }
        let Ok(file) = private_open_options().open(&path) else {
            continue;
        };
        if file.try_lock().is_err() {
            continue;
        }
        if file.unlock().is_err() {
            continue;
        }
        drop(file);
        let _ = fs::remove_file(path);
    }
}

fn prune_cache_locked(root: &Path, current: &Path, limit: usize) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    let mut runtimes: Vec<_> = entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            is_runtime_entry(&path).then(|| {
                let modified = entry
                    .metadata()
                    .and_then(|metadata| metadata.modified())
                    .unwrap_or(UNIX_EPOCH);
                (path, modified)
            })
        })
        .collect();
    runtimes.sort_by_key(|(_, modified)| std::cmp::Reverse(*modified));
    let mut retained = runtimes.len();
    for (path, _) in runtimes.into_iter().rev() {
        if retained <= limit {
            break;
        }
        if path == current || !remove_inactive_runtime(&path) {
            continue;
        }
        retained -= 1;
    }
}

fn is_runtime_entry(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    name.strip_suffix(".mjs")
        .is_some_and(|stem| stem.len() == 64 && stem.bytes().all(|byte| byte.is_ascii_hexdigit()))
        && fs::symlink_metadata(path)
            .is_ok_and(|metadata| metadata.is_file() && !metadata.file_type().is_symlink())
}

fn remove_inactive_runtime(path: &Path) -> bool {
    let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
        return false;
    };
    let lease_path = path.with_file_name(crate::cstr!("{stem}.lease").as_str());
    let Ok(metadata) = fs::symlink_metadata(&lease_path) else {
        return false;
    };
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return false;
    }
    let Ok(lease) = private_open_options().open(&lease_path) else {
        return false;
    };
    if lease.try_lock().is_err() || fs::remove_file(path).is_err() {
        return false;
    }
    drop(lease);
    let _ = fs::remove_file(lease_path);
    true
}
