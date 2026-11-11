use super::{Checked, Value, bound, fs, load, support};
use serde_json::json;

pub(super) const UPDATE: &str =
    "malformed-membership/provider-node-forge-security-update-12789.json";
pub(super) const UPDATE_HASH: &str =
    "c62860c79cddfff2bd76b8528ef3b40c1e01fe40dbc86f1b0077130fc7ae5778";
const PREVIOUS_BINDING: &str = "tests/_fixtures/differential/typechecker/empty-vue-star-exports/malformed-membership/provider-eslint-vue-scorecard-update-12539.json";

pub(super) fn update() -> Value {
    load(
        UPDATE,
        UPDATE_HASH,
        "exact-provider-qualification-node-forge-security-successor",
    )
}

pub(super) struct NodeForgeSecurityQualificationGraph<'a> {
    pub(super) lock: &'a [u8],
    pub(super) package: &'a [u8],
    pub(super) workspace: &'a [u8],
}

fn reconstruct(update: &Value, key: &str, current: &[u8]) -> Checked<Vec<u8>> {
    let mut predecessor = current.to_vec();
    let replacements = update[key]
        .as_array()
        .ok_or("complete node-forge security replacements")?;
    for replacement in replacements.iter().rev() {
        let before = replacement["before"]
            .as_str()
            .ok_or("complete node-forge predecessor spelling")?
            .as_bytes();
        let after = replacement["after"]
            .as_str()
            .ok_or("complete node-forge successor spelling")?
            .as_bytes();
        if before.is_empty() || after.is_empty() {
            return Err("complete node-forge security replacements");
        }
        let offsets = predecessor
            .windows(after.len())
            .enumerate()
            .filter_map(|(offset, candidate)| (candidate == after).then_some(offset))
            .collect::<Vec<_>>();
        if offsets.len() != 1 {
            return Err("one exact node-forge successor spelling");
        }
        predecessor.splice(offsets[0]..offsets[0] + after.len(), before.iter().copied());
    }
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
    graph: &NodeForgeSecurityQualificationGraph<'_>,
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
        return Err("exact frozen node-forge security predecessor");
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
            return Err("exact current node-forge security graph bytes");
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
        return Err("complete node-forge security predecessor reconstruction");
    }
    if update["verification"]["command"] != "vp install --frozen-lockfile --lockfile-only"
        || update["verification"]["package"] != "node-forge@1.4.3"
        || update["verification"]["packageSha256"]
            != "9b6be415be2445073b7c08e1ebcfdbcee25de2a48d89ff3b98f92a33867ad96f"
        || update["verification"]["advisoriesClosed"]
            != json!([
                "GHSA-w4xr-hqv8-ch44",
                "GHSA-6cv4-p8m7-4r25",
                "GHSA-j8rw-jp99-435v"
            ])
        || update["nativeAcceptanceCredit"] != json!(false)
    {
        return Err("exact node-forge security verification");
    }
    Ok(())
}

pub(super) fn authorize_successor(lock: &[u8], package: &[u8], workspace: &[u8]) -> Checked<Value> {
    let predecessor_bytes = fs::read(support::corpus().join(super::eslint_vue_scorecard::UPDATE))
        .map_err(|_| "complete node-forge security predecessor")?;
    let predecessor = super::eslint_vue_scorecard::update();
    let previous_lock = reconstruct_lock_predecessor(lock)?;
    let previous_workspace = reconstruct_workspace_predecessor(workspace)?;
    super::eslint_vue_scorecard::authorize_successor(&previous_lock, package, &previous_workspace)?;
    let update = update();
    let graph = NodeForgeSecurityQualificationGraph {
        lock,
        package,
        workspace,
    };
    verify(&predecessor, &predecessor_bytes, &update, &graph)?;
    Ok(update)
}
