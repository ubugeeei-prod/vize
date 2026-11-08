use super::{Checked, Value, bound, load};
use serde_json::json;

pub(super) const UPDATE: &str = "malformed-membership/provider-strict-checker-update-11628.json";
pub(super) const UPDATE_HASH: &str =
    "012f62dce9432e38cbe0651f6c4f71f80c2cc594c3302f5e37ff751aeaf1cc69";
const PREVIOUS_BINDING: &str = "tests/_fixtures/differential/typechecker/empty-vue-star-exports/malformed-membership/provider-root-dependency-ownership-update-12123.json";
pub(super) const MANIFEST: &str = "npm/cli/package.json";
pub(super) const CHECKER_DECLARATION: &str = "npm/cli/virtual-ts-checker.d.ts";
pub(super) const STRICT_DECLARATION: &str = "npm/cli/virtual-ts-checker-strict.d.ts";

pub(super) fn update() -> Value {
    load(
        UPDATE,
        UPDATE_HASH,
        "exact-provider-qualification-strict-checker-successor",
    )
}

pub(super) struct StrictCheckerQualificationGraph<'a> {
    pub(super) lock: &'a [u8],
    pub(super) package: &'a [u8],
    pub(super) workspace: &'a [u8],
    pub(super) manifest: &'a [u8],
    pub(super) checker_declaration: &'a [u8],
    pub(super) strict_declaration: &'a [u8],
}

pub(super) fn verify(
    predecessor: &Value,
    predecessor_bytes: &[u8],
    update: &Value,
    graph: &StrictCheckerQualificationGraph<'_>,
) -> Checked<()> {
    if update["previousQualificationLock"] != predecessor["finalQualificationLock"]
        || update["previousQualificationPackage"] != predecessor["finalQualificationPackage"]
        || !bound(
            &update["previousProviderQualificationRecord"],
            PREVIOUS_BINDING,
            predecessor_bytes,
        )
    {
        return Err("exact frozen strict checker qualification predecessor");
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
        (&update["cliManifest"], MANIFEST, graph.manifest),
        (
            &update["checkerDeclaration"],
            CHECKER_DECLARATION,
            graph.checker_declaration,
        ),
        (
            &update["strictDeclaration"],
            STRICT_DECLARATION,
            graph.strict_declaration,
        ),
    ] {
        if !bound(binding, path, body) {
            return Err("exact current strict checker qualification graph bytes");
        }
    }
    if update["resolver"]["command"]
        != "vp install --lockfile-only --no-frozen-lockfile --ignore-scripts"
        || update["resolver"]["fixedPointSha256"] != update["finalQualificationLock"]["sha256"]
    {
        return Err("exact strict checker resolver fixed point");
    }

    let manifest: Value =
        serde_json::from_slice(graph.manifest).map_err(|_| "complete CLI manifest")?;
    let files = manifest["files"]
        .as_array()
        .ok_or("one published strict checker declaration")?;
    if manifest["name"] != "vize"
        || files
            .iter()
            .filter(|entry| **entry == json!("virtual-ts-checker-strict.d.ts"))
            .count()
            != 1
        || manifest["exports"]["./virtual-ts-checker-strict"]["types"]
            != "./virtual-ts-checker-strict.d.ts"
    {
        return Err("one published strict checker declaration");
    }
    let checker =
        std::str::from_utf8(graph.checker_declaration).map_err(|_| "UTF-8 checker declaration")?;
    let strict = std::str::from_utf8(graph.strict_declaration)
        .map_err(|_| "UTF-8 strict checker declaration")?;
    if checker
        .matches("import type {} from \"./virtual-ts-checker-strict.js\"")
        .count()
        != 1
        || checker.matches("var __VLS_PROPS_FALLBACK").count() != 1
        || strict.matches("interface ImportMeta").count() != 1
        || strict.matches("interface IntrinsicAttributes").count() != 1
        || strict.contains("__VLS_PROPS_FALLBACK")
    {
        return Err("one strict-to-open checker policy boundary");
    }
    if update["nativeAcceptanceCredit"] != json!(false) {
        return Err("source provenance only");
    }
    Ok(())
}
