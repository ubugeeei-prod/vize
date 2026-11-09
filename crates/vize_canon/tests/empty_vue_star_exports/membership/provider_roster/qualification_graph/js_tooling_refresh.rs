use super::{Checked, Value, bound, fs, load, support};
use serde_json::json;

pub(super) const UPDATE: &str =
    "malformed-membership/provider-js-tooling-refresh-update-12212.json";
pub(super) const UPDATE_HASH: &str =
    "912ab62d53b6903d3d07c633c471d42fed27d11746265827f7785eeacd3c23b2";
const PREVIOUS_BINDING: &str = "tests/_fixtures/differential/typechecker/empty-vue-star-exports/malformed-membership/provider-docs-metadata-update-12236.json";
const UNPLUGIN_MANIFEST: &str = "npm/builder/unplugin/package.json";
const EXAMPLE_MANIFEST: &str = "npm/builder/unplugin/example/package.json";

pub(super) fn update() -> Value {
    load(
        UPDATE,
        UPDATE_HASH,
        "exact-provider-qualification-js-tooling-refresh-successor",
    )
}

pub(super) struct JsToolingRefreshQualificationGraph<'a> {
    pub(super) lock: &'a [u8],
    pub(super) package: &'a [u8],
    pub(super) workspace: &'a [u8],
    pub(super) unplugin_manifest: &'a [u8],
    pub(super) example_manifest: &'a [u8],
}

pub(super) fn authorize(
    predecessor: &Value,
    predecessor_bytes: &[u8],
    lock: &[u8],
    package: &[u8],
    workspace: &[u8],
) -> Checked<Value> {
    let update = update();
    let unplugin_manifest = fs::read(support::repository().join(UNPLUGIN_MANIFEST))
        .map_err(|_| "complete unplugin manifest")?;
    let example_manifest = fs::read(support::repository().join(EXAMPLE_MANIFEST))
        .map_err(|_| "complete unplugin example manifest")?;
    let graph = JsToolingRefreshQualificationGraph {
        lock,
        package,
        workspace,
        unplugin_manifest: &unplugin_manifest,
        example_manifest: &example_manifest,
    };
    verify(predecessor, predecessor_bytes, &update, &graph)?;
    Ok(update)
}

pub(super) fn verify(
    predecessor: &Value,
    predecessor_bytes: &[u8],
    update: &Value,
    graph: &JsToolingRefreshQualificationGraph<'_>,
) -> Checked<()> {
    if update["previousQualificationLock"] != predecessor["finalQualificationLock"]
        || update["previousQualificationPackage"] != predecessor["finalQualificationPackage"]
        || !bound(
            &update["previousProviderQualificationRecord"],
            PREVIOUS_BINDING,
            predecessor_bytes,
        )
    {
        return Err("exact frozen JavaScript tooling refresh predecessor");
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
        (
            &update["unpluginManifest"],
            UNPLUGIN_MANIFEST,
            graph.unplugin_manifest,
        ),
        (
            &update["exampleManifest"],
            EXAMPLE_MANIFEST,
            graph.example_manifest,
        ),
    ] {
        if !bound(binding, path, body) {
            return Err("exact current JavaScript tooling refresh graph bytes");
        }
    }
    if update["resolver"]["command"]
        != "vp install --lockfile-only --no-frozen-lockfile --ignore-scripts"
        || update["resolver"]["fixedPointSha256"] != update["finalQualificationLock"]["sha256"]
    {
        return Err("exact JavaScript tooling refresh resolver fixed point");
    }

    let workspace = std::str::from_utf8(graph.workspace).map_err(|_| "UTF-8 workspace manifest")?;
    if workspace
        .matches("  - \"npm/builder/unplugin/example\"\n")
        .count()
        != 1
    {
        return Err("one unplugin example workspace owner");
    }
    let unplugin: Value = serde_json::from_slice(graph.unplugin_manifest)
        .map_err(|_| "complete unplugin manifest")?;
    if unplugin["name"] != "@vizejs/unplugin"
        || unplugin["dependencies"]["unplugin"] != "catalog:vite-stack"
        || unplugin["scripts"]["test"] != "vp pack && node --test src/*.test.ts src/**/*.test.ts"
    {
        return Err("one executable unplugin package contract");
    }
    let example: Value = serde_json::from_slice(graph.example_manifest)
        .map_err(|_| "complete unplugin example manifest")?;
    if example["name"] != "@vizejs/example-builder-matrix"
        || example["private"] != json!(true)
        || example["devDependencies"]["@vizejs/unplugin"] != "workspace:*"
        || example["scripts"]["build"]
            != "vp run --filter @vizejs/unplugin build && node build/build-all.ts"
    {
        return Err("one executable unplugin example contract");
    }
    let lock = std::str::from_utf8(graph.lock).map_err(|_| "UTF-8 qualification lock")?;
    if lock.matches("\n  npm/builder/unplugin/example:\n").count() != 1 {
        return Err("one exact unplugin example importer");
    }
    if update["nativeAcceptanceCredit"] != json!(false) {
        return Err("source provenance only");
    }
    Ok(())
}
