//! Qualify the lockfile importers for the focused entry packages.

use super::{Checked, Value, bound, json, support};

const START: &str = "  npm/entry/compile:\n";
const END: &str = "  npm/formatter/prettier:\n";
const INSERTION_BYTES: usize = 1_784;
const INSERTION_SHA: &str = "1f68c66ca35cb7d589b49edae5c8090c241f7ccfaa4c6eb1daa63219b9d0f793";
const WORKSPACE_ENTRY_PREDECESSOR: &[u8] = b"  - \"npm/formatter/*\"\n";
const WORKSPACE_ENTRY: &[u8] = b"  - \"npm/entry/*\"\n";
const WORKSPACE_ENTRY_SUCCESSOR: &[u8] = b"  - \"npm/compose/*\"\n";
const IMPORTERS: [&str; 11] = [
    "compile",
    "compile-jsx",
    "compile-tsrx",
    "content-mapper",
    "curator",
    "doctor",
    "fmt",
    "lint",
    "lsp",
    "tui",
    "typecheck",
];

fn unique_offset(haystack: &[u8], needle: &[u8]) -> Checked<usize> {
    let mut offsets = haystack
        .windows(needle.len())
        .enumerate()
        .filter_map(|(offset, candidate)| (candidate == needle).then_some(offset));
    let offset = offsets
        .next()
        .ok_or("complete entry package importer boundary")?;
    if offsets.next().is_some() {
        return Err("one entry package importer boundary");
    }
    Ok(offset)
}

pub(super) fn reconstruct_lock_predecessor(lock: &[u8]) -> Checked<Vec<u8>> {
    let start = unique_offset(lock, START.as_bytes())?;
    let suffix = lock
        .get(start..)
        .ok_or("bounded entry package importer start")?;
    let end = start
        + suffix
            .windows(END.len())
            .position(|candidate| candidate == END.as_bytes())
            .ok_or("complete entry package importer end")?;
    let insertion = lock
        .get(start..end)
        .ok_or("bounded entry package importer insertion")?;
    if insertion.len() != INSERTION_BYTES || support::hash(insertion) != INSERTION_SHA {
        return Err("exact entry package importer insertion");
    }
    for importer in IMPORTERS {
        let header = format!("  npm/entry/{importer}:\n");
        if insertion
            .windows(header.len())
            .filter(|candidate| *candidate == header.as_bytes())
            .count()
            != 1
        {
            return Err("eleven exact entry package importers");
        }
    }
    let mut predecessor = lock.to_vec();
    predecessor.drain(start..end);
    Ok(predecessor)
}

pub(super) fn reconstruct_workspace_predecessor(workspace: &[u8]) -> Checked<Vec<u8>> {
    let offset = unique_offset(workspace, WORKSPACE_ENTRY)?;
    let end = offset
        .checked_add(WORKSPACE_ENTRY.len())
        .ok_or("bounded entry package workspace end")?;
    let predecessor_start = offset
        .checked_sub(WORKSPACE_ENTRY_PREDECESSOR.len())
        .ok_or("entry package workspace predecessor position")?;
    let successor_end = end
        .checked_add(WORKSPACE_ENTRY_SUCCESSOR.len())
        .ok_or("entry package workspace successor position")?;
    if workspace.get(predecessor_start..offset) != Some(WORKSPACE_ENTRY_PREDECESSOR)
        || workspace.get(end..successor_end) != Some(WORKSPACE_ENTRY_SUCCESSOR)
    {
        return Err("entry package workspace pattern position");
    }
    let mut predecessor = workspace.to_vec();
    predecessor.drain(offset..end);
    Ok(predecessor)
}

pub(super) fn authorize_successor(
    predecessor: &Value,
    predecessor_lock: &[u8],
    lock: &[u8],
    package: &[u8],
    workspace: &[u8],
) -> Checked<Value> {
    let predecessor_workspace = reconstruct_workspace_predecessor(workspace)?;
    if !bound(
        &predecessor["finalQualificationLock"],
        "pnpm-lock.yaml",
        predecessor_lock,
    ) || !bound(
        &predecessor["finalQualificationPackage"],
        "package.json",
        package,
    ) || !bound(
        &predecessor["workspaceManifest"],
        "pnpm-workspace.yaml",
        &predecessor_workspace,
    ) {
        return Err("exact entry package qualification predecessor");
    }
    if reconstruct_lock_predecessor(lock)? != predecessor_lock {
        return Err("complete entry package lock predecessor reconstruction");
    }
    let workspace_text = std::str::from_utf8(workspace).map_err(|_| "UTF-8 workspace manifest")?;
    if workspace_text.matches("  - \"npm/entry/*\"\n").count() != 1 {
        return Err("one focused entry package workspace pattern");
    }
    Ok(json!({
        "version": 1,
        "kind": "exact-provider-qualification-entry-packages-successor",
        "nativeAcceptanceCredit": false,
        "previousQualificationLock": predecessor["finalQualificationLock"],
        "previousQualificationPackage": predecessor["finalQualificationPackage"],
        "finalQualificationLock": {
            "path": "pnpm-lock.yaml",
            "bytes": lock.len(),
            "sha256": support::hash(lock),
        },
        "finalQualificationPackage": {
            "path": "package.json",
            "bytes": package.len(),
            "sha256": support::hash(package),
        },
        "workspaceManifest": {
            "path": "pnpm-workspace.yaml",
            "bytes": workspace.len(),
            "sha256": support::hash(workspace),
        },
        "previousWorkspaceManifest": {
            "path": "pnpm-workspace.yaml",
            "bytes": predecessor_workspace.len(),
            "sha256": support::hash(&predecessor_workspace),
        },
        "entryImporters": IMPORTERS,
        "verification": {
            "command": "vp install --frozen-lockfile",
            "lockSha256": support::hash(lock),
        },
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn current_lock_reconstructs_one_already_qualified_predecessor() {
        let repository = support::repository();
        let lock = fs::read(repository.join("pnpm-lock.yaml")).unwrap();
        let package = fs::read(repository.join("package.json")).unwrap();
        let workspace = fs::read(repository.join("pnpm-workspace.yaml")).unwrap();
        let previous_lock = reconstruct_lock_predecessor(&lock).unwrap();
        let previous_workspace = reconstruct_workspace_predecessor(&workspace).unwrap();
        let root = super::super::super::load();
        let predecessor = super::super::authorize(&root, &previous_lock, &package)
            .unwrap()
            .unwrap();
        let update =
            authorize_successor(&predecessor, &previous_lock, &lock, &package, &workspace).unwrap();
        assert_eq!(update["entryImporters"], json!(IMPORTERS));
        assert_eq!(
            update["previousWorkspaceManifest"]["sha256"],
            support::hash(&previous_workspace)
        );
        let mut duplicated = lock.clone();
        duplicated.extend_from_slice(&lock);
        assert!(reconstruct_lock_predecessor(&duplicated).is_err());
        let mut duplicated = workspace.clone();
        duplicated.extend_from_slice(&workspace);
        assert!(reconstruct_workspace_predecessor(&duplicated).is_err());
        let mut prepended = WORKSPACE_ENTRY.to_vec();
        prepended.extend_from_slice(&previous_workspace);
        assert!(reconstruct_workspace_predecessor(&prepended).is_err());
    }
}
