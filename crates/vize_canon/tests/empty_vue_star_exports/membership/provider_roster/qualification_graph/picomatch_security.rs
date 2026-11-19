use super::{Checked, Value, bound, fs, load, support};
use serde_json::json;

pub(super) const UPDATE: &str =
    "malformed-membership/provider-picomatch-security-update-13323.json";
pub(super) const UPDATE_HASH: &str =
    "b145d7d4150ae527e0a33096d281e8711b6e053e99783a042a44515afcad60f2";
const PREVIOUS_BINDING: &str = "tests/_fixtures/differential/typechecker/empty-vue-star-exports/malformed-membership/provider-runtime-security-update-12959.json";

const PATCHES: [(&str, &str); 2] = [
    (
        "2.3.2",
        "465376b36bcea4ee299ad72d4a4e2aeae248b6d7746af3e51ef336ecad3f2a06",
    ),
    (
        "4.0.7",
        "ed03f84a8ec301c3a74404210aa22dc09aec09ce1141f7ceabe10bc1b0f08056",
    ),
];

pub(super) fn update() -> Value {
    load(
        UPDATE,
        UPDATE_HASH,
        "exact-provider-qualification-picomatch-security-successor",
    )
}

pub(super) struct PicomatchSecurityQualificationGraph<'a> {
    pub(super) lock: &'a [u8],
    pub(super) package: &'a [u8],
    pub(super) workspace: &'a [u8],
}

fn reconstruct(update: &Value, key: &str, current: &[u8]) -> Checked<Vec<u8>> {
    let replacements = update[key]
        .as_array()
        .ok_or("complete picomatch security replacements")?;
    let mut predecessor = Vec::with_capacity(current.len());
    let mut cursor = 0;
    for replacement in replacements {
        let before = replacement["before"]
            .as_str()
            .ok_or("complete picomatch security predecessor context")?
            .as_bytes();
        let after = replacement["after"]
            .as_str()
            .ok_or("complete picomatch security successor context")?
            .as_bytes();
        if before.is_empty() || after.is_empty() {
            return Err("complete picomatch security replacements");
        }
        let offset = replacement["offset"]
            .as_u64()
            .and_then(|value| usize::try_from(value).ok())
            .ok_or("bounded picomatch security successor context")?;
        let end = offset
            .checked_add(after.len())
            .ok_or("bounded picomatch security successor context")?;
        if offset < cursor || current.get(offset..end) != Some(after) {
            return Err("one exact picomatch security successor context");
        }
        predecessor.extend_from_slice(&current[cursor..offset]);
        predecessor.extend_from_slice(before);
        cursor = end;
    }
    predecessor.extend_from_slice(&current[cursor..]);
    Ok(predecessor)
}

pub(super) fn reconstruct_lock_predecessor(current: &[u8]) -> Checked<Vec<u8>> {
    reconstruct(&update(), "lockReplacements", current)
}

pub(super) fn reconstruct_workspace_predecessor(current: &[u8]) -> Checked<Vec<u8>> {
    reconstruct(&update(), "workspaceReplacements", current)
}

pub(super) fn verify(
    predecessor: &Value,
    predecessor_bytes: &[u8],
    update: &Value,
    graph: &PicomatchSecurityQualificationGraph<'_>,
) -> Checked<()> {
    if update["previousQualificationLock"] != predecessor["finalQualificationLock"]
        || update["previousQualificationPackage"] != predecessor["finalQualificationPackage"]
        || update["previousWorkspaceManifest"] != predecessor["workspaceManifest"]
        || !bound(
            &update["previousProviderQualificationRecord"],
            PREVIOUS_BINDING,
            predecessor_bytes,
        )
    {
        return Err("exact frozen picomatch security predecessor");
    }
    for (binding, path, body) in [
        (
            &update["finalQualificationLock"],
            "pnpm-lock.yaml",
            graph.lock,
        ),
        (
            &update["finalQualificationPackage"],
            "package.json",
            graph.package,
        ),
        (
            &update["workspaceManifest"],
            "pnpm-workspace.yaml",
            graph.workspace,
        ),
    ] {
        if !bound(binding, path, body) {
            return Err("exact current picomatch security graph bytes");
        }
    }

    let previous_lock = reconstruct(update, "lockReplacements", graph.lock)?;
    let previous_workspace = reconstruct(update, "workspaceReplacements", graph.workspace)?;
    if !bound(
        &predecessor["finalQualificationLock"],
        "pnpm-lock.yaml",
        &previous_lock,
    ) || !bound(
        &predecessor["workspaceManifest"],
        "pnpm-workspace.yaml",
        &previous_workspace,
    ) {
        return Err("complete picomatch security predecessor reconstruction");
    }

    let lock = std::str::from_utf8(graph.lock).map_err(|_| "UTF-8 picomatch security lock")?;
    let workspace =
        std::str::from_utf8(graph.workspace).map_err(|_| "UTF-8 picomatch security workspace")?;
    for (version, hash) in PATCHES {
        let selector = format!("  picomatch@{version}: {hash}\n");
        let declared = format!("  picomatch@{version}: patches/picomatch@{version}.patch\n");
        let snapshot = format!("  picomatch@{version}(patch_hash={hash}): {{}}\n");
        if lock.matches(&selector).count() != 1
            || lock.matches(&snapshot).count() != 1
            || workspace.matches(&declared).count() != 1
        {
            return Err("both patched picomatch versions");
        }
        for unpatched in [
            format!("picomatch: {version}\n"),
            format!("(picomatch@{version})"),
            format!("  picomatch@{version}: {{}}\n"),
        ] {
            if lock.contains(&unpatched) {
                return Err("no unpatched picomatch references");
            }
        }
    }

    let verification = &update["verification"];
    if verification["command"] != "node tools/commands/ci/npm-audit.ts"
        || verification["frozenInstall"] != "vp install --frozen-lockfile --prefer-offline"
        || verification["patches"]
            != json!({
                "picomatch@2.3.2": PATCHES[0].1,
                "picomatch@4.0.7": PATCHES[1].1,
            })
        || verification["advisoryRemediated"] != "GHSA-fc5q-39xc-vjpf"
        || verification["upstreamFix"] != "https://github.com/micromatch/picomatch/pull/266"
        || update["nativeAcceptanceCredit"] != json!(false)
    {
        return Err("exact picomatch security verification");
    }
    Ok(())
}

pub(super) fn authorize_successor(lock: &[u8], package: &[u8], workspace: &[u8]) -> Checked<Value> {
    let predecessor_bytes = fs::read(support::corpus().join(super::runtime_security::UPDATE))
        .map_err(|_| "complete picomatch security predecessor")?;
    let predecessor = super::runtime_security::update();
    let previous_lock = reconstruct_lock_predecessor(lock)?;
    let previous_workspace = reconstruct_workspace_predecessor(workspace)?;
    super::runtime_security::authorize_successor(&previous_lock, package, &previous_workspace)?;
    let update = update();
    let graph = PicomatchSecurityQualificationGraph {
        lock,
        package,
        workspace,
    };
    verify(&predecessor, &predecessor_bytes, &update, &graph)?;
    Ok(update)
}
