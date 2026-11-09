use super::{Checked, Value, bound, load};
use serde_json::json;

pub(super) const UPDATE: &str = "malformed-membership/provider-create-fresco-update-12233.json";
pub(super) const UPDATE_HASH: &str =
    "204af49ff268ee250bf328796c6e5abccc034cd116d94dac0439475568a536a6";
const PREVIOUS_BINDING: &str = "tests/_fixtures/differential/typechecker/empty-vue-star-exports/malformed-membership/provider-strict-checker-update-11628.json";
pub(super) const MANIFEST: &str = "npm/create-fresco/package.json";

pub(super) fn update() -> Value {
    load(
        UPDATE,
        UPDATE_HASH,
        "exact-provider-qualification-create-fresco-successor",
    )
}

pub(super) struct CreateFrescoQualificationGraph<'a> {
    pub(super) lock: &'a [u8],
    pub(super) package: &'a [u8],
    pub(super) workspace: &'a [u8],
    pub(super) manifest: &'a [u8],
}

pub(super) fn verify(
    predecessor: &Value,
    predecessor_bytes: &[u8],
    update: &Value,
    graph: &CreateFrescoQualificationGraph<'_>,
) -> Checked<()> {
    if update["previousQualificationLock"] != predecessor["finalQualificationLock"]
        || update["previousQualificationPackage"] != predecessor["finalQualificationPackage"]
        || !bound(
            &update["previousProviderQualificationRecord"],
            PREVIOUS_BINDING,
            predecessor_bytes,
        )
    {
        return Err("exact frozen create-fresco qualification predecessor");
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
        (&update["createFrescoManifest"], MANIFEST, graph.manifest),
    ] {
        if !bound(binding, path, body) {
            return Err("exact current create-fresco qualification graph bytes");
        }
    }
    if update["resolver"]["command"]
        != "vp install --lockfile-only --no-frozen-lockfile --ignore-scripts"
        || update["resolver"]["fixedPointSha256"] != update["finalQualificationLock"]["sha256"]
    {
        return Err("exact create-fresco resolver fixed point");
    }

    let manifest: Value =
        serde_json::from_slice(graph.manifest).map_err(|_| "complete create-fresco manifest")?;
    let dependencies = manifest["devDependencies"]
        .as_object()
        .ok_or("four create-fresco development dependencies")?;
    if manifest["name"] != "@vizejs/create-fresco"
        || manifest["bin"]["create-fresco"] != "./dist/cli.mjs"
        || manifest["dependencies"] != Value::Null
        || dependencies.len() != 4
        || dependencies.get("@types/node") != Some(&json!("catalog:typescript"))
        || dependencies.get("tsx") != Some(&json!("catalog:typescript"))
        || dependencies.get("typescript") != Some(&json!("catalog:typescript"))
        || dependencies.get("vite-plus") != Some(&json!("catalog:vite-stack"))
        || manifest["engines"]["node"] != ">=22"
        || manifest["metadata"]["vize"]["stability"] != "experimental"
    {
        return Err("one declared create-fresco package contract");
    }
    let workspace = std::str::from_utf8(graph.workspace).map_err(|_| "UTF-8 workspace manifest")?;
    if workspace.matches("  - \"npm/*\"\n").count() != 1 {
        return Err("one create-fresco workspace owner pattern");
    }
    let lock = std::str::from_utf8(graph.lock).map_err(|_| "UTF-8 qualification lock")?;
    if lock.matches("\n  npm/create-fresco:\n").count() != 1
        || !lock.contains("      tsx:\n        specifier: catalog:typescript")
        || !lock.contains("      vite-plus:\n        specifier: catalog:vite-stack")
    {
        return Err("one exact create-fresco importer");
    }
    if update["nativeAcceptanceCredit"] != json!(false) {
        return Err("source provenance only");
    }
    Ok(())
}
