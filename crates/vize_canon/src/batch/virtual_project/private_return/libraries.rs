//! One known packaged NoEmbed distribution; unknown native providers stay inline.
//!
//! Actual image/build metadata and all physical asset hashes are held in
//! source-before e45769e3629eaca0cd91ddab7142fedf7f644c9cb6254805e91133e32abe8204.
//! The primary packaged-NoEmbed recipe copies adjacent libs and tests physical
//! lib.es5 resolution (Hereby d04edeea...). This is a package capability, NOT
//! ownership of the modified Go source or observed loaded-file membership.

use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};
use vize_carton::String;

pub(super) struct LibrarySource {
    pub(super) path: PathBuf,
    pub(super) body: String,
}

pub(super) struct PackagedLibraries {
    pub(super) root: PathBuf,
    pub(super) sources: Vec<LibrarySource>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Asset {
    name: String,
    bytes: u64,
    sha256: String,
}
const IMAGE_SHA256: &str = "a183b1956edbe7e7d98f2249858eedf2d8a9790809cc4909db1276e202faaa70";
const IMAGE_BYTES: u64 = 24_522_912;
const ASSETS: &str = include_str!("libraries/assets.json");

pub(super) fn inspect(native: &Path) -> Option<PackagedLibraries> {
    if !cfg!(target_os = "linux") {
        return None;
    }
    let assets: Vec<Asset> = serde_json::from_str(ASSETS).ok()?;
    if assets.len() != 116 {
        return None;
    }
    inspect_manifest(native, IMAGE_BYTES, IMAGE_SHA256, &assets)
}

fn inspect_manifest(
    native: &Path,
    bytes: u64,
    hash: &str,
    assets: &[Asset],
) -> Option<PackagedLibraries> {
    if native.file_name()?.to_str()? != "tsc" || native != fs::canonicalize(native).ok()? {
        return None;
    }
    let root = native.parent()?;
    if root.file_name()?.to_str()? != "lib" || file_hash(native)? != (bytes, String::from(hash)) {
        return None;
    }
    let mut names = Vec::new();
    for entry in fs::read_dir(root).ok()? {
        let entry = entry.ok()?;
        let name = entry.file_name().into_string().ok()?;
        if !name.is_ascii() {
            return None;
        }
        if name.ends_with(".ts") || name.ends_with(".mts") || name.ends_with(".cts") {
            names.push(name);
        }
    }
    names.sort();
    if names.len() != assets.len()
        || names
            .iter()
            .zip(assets)
            .any(|(name, asset)| name.as_str() != asset.name.as_str())
    {
        return None;
    }
    let mut sources = Vec::with_capacity(assets.len());
    for asset in assets {
        let path = root.join(asset.name);
        let before = fs::symlink_metadata(&path).ok()?;
        if !before.is_file() || before.file_type().is_symlink() || before.len() != asset.bytes {
            return None;
        }
        let body = fs::read_to_string(&path).ok()?;
        if encode(Sha256::digest(body.as_bytes())) != asset.sha256
            || !same_file(&before, &fs::symlink_metadata(&path).ok()?)
        {
            return None;
        }
        sources.push(LibrarySource {
            path,
            body: String::from(body),
        });
    }
    Some(PackagedLibraries {
        root: root.to_path_buf(),
        sources,
    })
}

fn file_hash(path: &Path) -> Option<(u64, String)> {
    let before = fs::symlink_metadata(path).ok()?;
    if !before.is_file() || before.file_type().is_symlink() {
        return None;
    }
    let mut file = fs::File::open(path).ok()?;
    if !same_file(&before, &file.metadata().ok()?) {
        return None;
    }
    let mut digest = Sha256::new();
    let mut bytes = [0_u8; 65536];
    loop {
        let read = file.read(&mut bytes).ok()?;
        if read == 0 {
            break;
        }
        digest.update(&bytes[..read]);
    }
    if !same_file(&before, &fs::symlink_metadata(path).ok()?) {
        return None;
    }
    Some((before.len(), encode(digest.finalize())))
}

fn same_file(a: &fs::Metadata, b: &fs::Metadata) -> bool {
    if !b.is_file()
        || b.file_type().is_symlink()
        || a.len() != b.len()
        || a.modified().ok() != b.modified().ok()
    {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        a.dev() == b.dev() && a.ino() == b.ino() && a.mode() == b.mode()
    }
    #[cfg(not(unix))]
    {
        true
    }
}

fn encode(bytes: impl AsRef<[u8]>) -> String {
    const HEX: &[u8] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.as_ref().len() * 2);
    for byte in bytes.as_ref() {
        out.push(char::from(HEX[usize::from(byte >> 4)]));
        out.push(char::from(HEX[usize::from(byte & 15)]));
    }
    out
}

#[cfg(test)]
mod tests;
