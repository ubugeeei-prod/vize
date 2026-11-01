//! Reuse the existing exclusive storage/lifetime machinery under a new purpose key.

use std::io;

use sha2::{Digest, Sha256};
use vize_carton::String;

use super::super::{
    installed_sources::PersistentStorageOwner, installed_storage::InstalledStorage,
};
use crate::batch::VirtualProject;

const PURPOSE: &[u8] = b"vize-cli-private-return-storage:v1\0";

fn purpose_key(project: &VirtualProject, owner: Option<&PersistentStorageOwner>) -> String {
    let mut hash = Sha256::new();
    hash.update(PURPOSE);
    hash.update(project.installed_storage_key().as_bytes());
    hash.update([u8::from(owner.is_some())]);
    if let Some(owner) = owner {
        let (root, config) = owner.identity_paths();
        for path in [Some(root), config] {
            hash.update([u8::from(path.is_some())]);
            if let Some(path) = path {
                let bytes = path.as_os_str().as_encoded_bytes();
                hash.update((bytes.len() as u64).to_le_bytes());
                hash.update(bytes);
            }
        }
    }
    let mut key = String::with_capacity(64);
    const HEX: &[u8] = b"0123456789abcdef";
    for byte in hash.finalize() {
        key.push(char::from(HEX[usize::from(byte >> 4)]));
        key.push(char::from(HEX[usize::from(byte & 15)]));
    }
    key
}

pub(super) fn claim(
    project: &mut VirtualProject,
    owner: Option<&PersistentStorageOwner>,
) -> io::Result<Option<InstalledStorage>> {
    if !project.can_scope_cli_diagnostics() {
        return Ok(None);
    }
    let Some(base) = project.virtual_root().parent() else {
        return Ok(None);
    };
    let mut storage = InstalledStorage::claim(base, purpose_key(project, owner).as_str())?;
    if storage.path().starts_with(project.project_root()) {
        return Ok(None);
    }
    storage.reset(&project.batch_build_info_dir(storage.path()))?;
    project.scope_cli_diagnostics(storage.path());
    Ok(Some(storage))
}

#[cfg(test)]
mod tests;
