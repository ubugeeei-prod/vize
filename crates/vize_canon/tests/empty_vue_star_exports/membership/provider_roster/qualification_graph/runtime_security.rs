use super::{Checked, Value, bound, fs, load, support};
use serde_json::json;

pub(super) const UPDATE: &str = "malformed-membership/provider-runtime-security-update-12959.json";
pub(super) const UPDATE_HASH: &str =
    "88dfbe561c318caec623d15fbac24d125689246c1e5efc2ff71fe7eb2137ce0d";
const PREVIOUS_BINDING: &str = "tests/_fixtures/differential/typechecker/empty-vue-star-exports/malformed-membership/provider-node-forge-security-update-12789.json";

pub(super) fn update() -> Value {
    load(
        UPDATE,
        UPDATE_HASH,
        "exact-provider-qualification-runtime-security-successor",
    )
}

pub(super) struct RuntimeSecurityQualificationGraph<'a> {
    pub(super) lock: &'a [u8],
    pub(super) package: &'a [u8],
    pub(super) workspace: &'a [u8],
}

fn reconstruct(update: &Value, key: &str, current: &[u8]) -> Checked<Vec<u8>> {
    let replacements = update[key]
        .as_array()
        .ok_or("complete runtime security replacements")?;
    let mut predecessor = Vec::with_capacity(current.len());
    let mut cursor = 0;
    for replacement in replacements {
        let before = replacement["before"]
            .as_str()
            .ok_or("complete runtime security predecessor context")?
            .as_bytes();
        let after = replacement["after"]
            .as_str()
            .ok_or("complete runtime security successor context")?
            .as_bytes();
        if before.is_empty() || after.is_empty() {
            return Err("complete runtime security replacements");
        }
        let offset = replacement["offset"]
            .as_u64()
            .and_then(|value| usize::try_from(value).ok())
            .ok_or("bounded runtime security successor context")?;
        let end = offset
            .checked_add(after.len())
            .ok_or("bounded runtime security successor context")?;
        if offset < cursor || current.get(offset..end) != Some(after) {
            return Err("one exact runtime security successor context");
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
    graph: &RuntimeSecurityQualificationGraph<'_>,
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
        return Err("exact frozen runtime security predecessor");
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
            return Err("exact current runtime security graph bytes");
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
        return Err("complete runtime security predecessor reconstruction");
    }

    let lock = std::str::from_utf8(graph.lock).map_err(|_| "UTF-8 runtime security lock")?;
    let workspace =
        std::str::from_utf8(graph.workspace).map_err(|_| "UTF-8 runtime security workspace")?;
    for current in [
        "  '@hono/node-server': 2.1.6",
        "  hono: 4.13.16",
        "  nuxt: 4.6.4",
        "  '@nuxt/schema@4.6.4':",
        "  '@nuxt/nitro-server@4.6.4':",
        "  hono@4.13.16:",
        "  nuxt@4.6.4:",
    ] {
        if !lock.contains(current) && !workspace.contains(current) {
            return Err("all patched runtime security versions");
        }
    }
    for vulnerable in [
        "@hono/node-server@2.1.3",
        "hono@4.13.11",
        "@nuxt/schema@4.6.3",
        "@nuxt/nitro-server@4.6.3",
        "nuxt@4.6.3",
    ] {
        if lock.contains(vulnerable) || workspace.contains(vulnerable) {
            return Err("no vulnerable runtime security versions");
        }
    }

    let verification = &update["verification"];
    if verification["command"] != "vp node tools/commands/ci/npm-audit.ts"
        || verification["packages"]
            != json!({
                "@hono/node-server": "2.1.6",
                "hono": "4.13.16",
                "nuxt": "4.6.4",
                "@nuxt/schema": "4.6.4",
                "@nuxt/nitro-server": "4.6.4",
            })
        || verification["audit"] != json!({"moderate": 0, "high": 0, "critical": 0})
        || verification["advisoriesClosed"]
            != json!([
                "GHSA-qv5m-9pm6-hqp3",
                "GHSA-jpvv-r467-5p4w",
                "GHSA-4m76-xfj5-7975",
                "GHSA-h7mg-cq63-vhpq",
                "GHSA-46wx-v44x-9hh2",
                "GHSA-4h7h-rv7c-7m2v",
                "GHSA-mjrq-cffv-q4qc",
                "GHSA-r99w-jh2x-3mg9",
            ])
        || update["nativeAcceptanceCredit"] != json!(false)
    {
        return Err("exact runtime security verification");
    }
    Ok(())
}

pub(super) fn authorize_successor(lock: &[u8], package: &[u8], workspace: &[u8]) -> Checked<Value> {
    let predecessor_bytes = fs::read(support::corpus().join(super::node_forge_security::UPDATE))
        .map_err(|_| "complete runtime security predecessor")?;
    let predecessor = super::node_forge_security::update();
    let previous_lock = reconstruct_lock_predecessor(lock)?;
    let previous_workspace = reconstruct_workspace_predecessor(workspace)?;
    super::node_forge_security::authorize_successor(&previous_lock, package, &previous_workspace)?;
    let update = update();
    let graph = RuntimeSecurityQualificationGraph {
        lock,
        package,
        workspace,
    };
    verify(&predecessor, &predecessor_bytes, &update, &graph)?;
    Ok(update)
}
