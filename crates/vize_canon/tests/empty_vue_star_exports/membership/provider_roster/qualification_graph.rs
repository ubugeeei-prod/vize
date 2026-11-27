//! Source-only graph successors; every provider and native membership gate remains live.
use super::{Checked, Value, final_graph, fs, json, support};

mod dependency_ownership;
use dependency_ownership::{
    OWNER_MANIFESTS, OwnershipQualificationGraph, update as dependency_ownership_update,
    verify as verify_dependency_ownership_update,
};
mod create_fresco;
use create_fresco::{
    CreateFrescoQualificationGraph, MANIFEST as CREATE_FRESCO_MANIFEST,
    update as create_fresco_update, verify as verify_create_fresco_update,
};
mod docs_metadata;
use docs_metadata::{
    DocsMetadataQualificationGraph, MANIFEST as DOCS_METADATA_MANIFEST,
    update as docs_metadata_update, verify as verify_docs_metadata_update,
};
mod entry_packages;
mod eslint_vue_scorecard;
mod fixture_updates;
use fixture_updates::{
    LOCKFILE_UPDATE, ORACLE_UPDATE, PARCEL_UPDATE, load, lockfile_update, oracle_update,
    parcel_update,
};
mod js_tooling_refresh;
mod node_forge_security;
mod oracle_importer;
use oracle_importer::{
    MANIFEST as JSX_ORACLE, PREVIOUS_PROVIDER, UPDATE_BINDING as ORACLE_UPDATE_BINDING,
    verify as verify_oracle_update,
};
mod picomatch_security;
mod runtime_security;
mod strict_checker;
use strict_checker::{authorize as authorize_strict_checker, update as strict_checker_update};
mod fresco;
use fresco::{
    FRESCO_MANIFEST, FRESCO_UPDATE, FrescoQualificationGraph, TOOLING_UPDATE, TOOLING_UPDATE_HASH,
    fresco_update, verify_fresco_update,
};
mod tooling;
mod translation_worker;
use tooling::{
    BABEL_MANIFEST, ESLINT_MANIFEST, OXLINT_MANIFEST, PRETTIER_MANIFEST, ToolingQualificationGraph,
    tooling_update, verify_tooling_update,
};

const LOCKFILE_UPDATE_BINDING: &str = "tests/_fixtures/differential/typechecker/empty-vue-star-exports/malformed-membership/provider-qualification-lockfile-update-11542.json";
const PARCEL_MANIFEST: &str = "npm/builder/parcel-native/package.json";
const CARGO_WORKSPACE: &str = "Cargo.toml";
const CARGO_LOCK: &str = "Cargo.lock";
const SETUP_AWAIT_ORACLE: &str = "tests/tooling/vue-setup-await-oracle/package.json";
const WORKSPACE: &str = "pnpm-workspace.yaml";
const TESTS_PACKAGE: &str = "tests/package.json";
const SETUP_AWAIT_SUPPORT: &str = "tests/tooling/support/setup-scope-await.mjs";
const SETUP_AWAIT_IMPORTER: &str = "  tests/tooling/vue-setup-await-oracle:\n    devDependencies:\n      '@vue/server-renderer':\n        specifier: 3.5.38\n        version: 3.5.38(vue@3.5.38(typescript@6.0.3))\n      vue:\n        specifier: 3.5.38\n        version: 3.5.38(typescript@6.0.3)\n";

fn bound(binding: &Value, path: &str, body: &[u8]) -> bool {
    binding["path"] == path
        && binding["bytes"] == json!(body.len())
        && binding["sha256"] == support::hash(body)
}

struct QualificationGraph<'a> {
    lock: &'a [u8],
    package: &'a [u8],
    workspace: &'a [u8],
    tests_package: &'a [u8],
    oracle: &'a [u8],
    support_script: &'a [u8],
}

struct ParcelQualificationGraph<'a> {
    lock: &'a [u8],
    package: &'a [u8],
    parcel_manifest: &'a [u8],
    cargo_workspace: &'a [u8],
    cargo_lock: &'a [u8],
}

fn verify_lockfile_update(
    _previous: &Value,
    predecessor: &Value,
    predecessor_bytes: &[u8],
    update: &Value,
    graph: &QualificationGraph<'_>,
) -> Checked<()> {
    if predecessor["previousProviderRecord"]["sha256"] != PREVIOUS_PROVIDER
        || update["previousQualificationLock"] != predecessor["finalQualificationLock"]
        || update["finalQualificationPackage"] != predecessor["finalQualificationPackage"]
        || !bound(
            &update["previousProviderQualificationRecord"],
            ORACLE_UPDATE_BINDING,
            predecessor_bytes,
        )
    {
        return Err("exact frozen qualification successor");
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
        (&update["workspaceManifest"], WORKSPACE, graph.workspace),
        (&update["testsManifest"], TESTS_PACKAGE, graph.tests_package),
        (&update["oracleManifest"], SETUP_AWAIT_ORACLE, graph.oracle),
        (
            &update["setupAwaitSupport"],
            SETUP_AWAIT_SUPPORT,
            graph.support_script,
        ),
    ] {
        if !bound(binding, path, body) {
            return Err("exact current qualification graph bytes");
        }
    }
    if update["resolver"]["command"]
        != "vp install --lockfile-only --no-frozen-lockfile --ignore-scripts"
        || update["resolver"]["fixedPointSha256"] != update["finalQualificationLock"]["sha256"]
    {
        return Err("exact resolver fixed point");
    }
    let workspace = std::str::from_utf8(graph.workspace).map_err(|_| "UTF-8 workspace manifest")?;
    if workspace
        .matches("  - \"tests/tooling/vue-setup-await-oracle\"")
        .count()
        != 1
        || workspace.matches("  \"@unhead/vue\": \"3.4.3\"").count() != 1
        || workspace.contains("  \"@unhead/vue\": \"3.3.1\"")
    {
        return Err("only the declared workspace and Unhead boundary");
    }
    let tests_package: Value =
        serde_json::from_slice(graph.tests_package).map_err(|_| "complete tests manifest")?;
    if tests_package["devDependencies"]["vue-setup-await-oracle"] != Value::Null {
        return Err("no shared setup-await oracle alias");
    }
    let oracle: Value =
        serde_json::from_slice(graph.oracle).map_err(|_| "complete setup-await oracle manifest")?;
    let dependencies = oracle["devDependencies"]
        .as_object()
        .ok_or("two exact setup-await dependencies")?;
    if dependencies.len() != 2
        || dependencies.get("vue") != Some(&json!("3.5.38"))
        || dependencies.get("@vue/server-renderer") != Some(&json!("3.5.38"))
    {
        return Err("two exact setup-await dependencies");
    }
    let lock = std::str::from_utf8(graph.lock).map_err(|_| "UTF-8 qualification lock")?;
    if !lock.contains("  '@unhead/vue': 3.4.3")
        || lock.matches(SETUP_AWAIT_IMPORTER).count() != 1
        || lock.contains("      vue-setup-await-oracle:\n        specifier: npm:vue@3.5.38")
    {
        return Err("exact generated lockfile boundary");
    }
    Ok(())
}

fn verify_parcel_update(
    predecessor: &Value,
    predecessor_bytes: &[u8],
    update: &Value,
    graph: &ParcelQualificationGraph<'_>,
) -> Checked<()> {
    if update["previousQualificationLock"] != predecessor["finalQualificationLock"]
        || update["finalQualificationPackage"] != predecessor["finalQualificationPackage"]
        || !bound(
            &update["previousProviderQualificationRecord"],
            LOCKFILE_UPDATE_BINDING,
            predecessor_bytes,
        )
    {
        return Err("exact frozen Parcel qualification successor");
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
            &update["parcelNativeManifest"],
            PARCEL_MANIFEST,
            graph.parcel_manifest,
        ),
        (
            &update["cargoWorkspaceManifest"],
            CARGO_WORKSPACE,
            graph.cargo_workspace,
        ),
        (&update["cargoLock"], CARGO_LOCK, graph.cargo_lock),
    ] {
        if !bound(binding, path, body) {
            return Err("exact current Parcel qualification graph bytes");
        }
    }
    if update["resolvers"]["pnpm"]["command"]
        != "vp install --lockfile-only --no-frozen-lockfile --ignore-scripts"
        || update["resolvers"]["pnpm"]["fixedPointSha256"]
            != update["finalQualificationLock"]["sha256"]
        || update["resolvers"]["cargo"]["command"]
            != "cargo metadata --locked --no-deps --format-version 1"
        || update["resolvers"]["cargo"]["fixedPointSha256"] != update["cargoLock"]["sha256"]
    {
        return Err("exact Parcel resolver fixed points");
    }

    let manifest: Value = serde_json::from_slice(graph.parcel_manifest)
        .map_err(|_| "complete Parcel native manifest")?;
    let artifacts = manifest["parcel"]["artifacts"]
        .as_object()
        .ok_or("eight Parcel native artifacts")?;
    if manifest["name"] != "@vizejs/parcel-transformer-native"
        || manifest["devDependencies"]["parcel"] != "3.0.0-alpha.2"
        || manifest["devDependencies"]["vue"] != "catalog:vue-stable"
        || manifest["parcel"]["abi"] != 1
        || artifacts.len() != 8
    {
        return Err("one declared Parcel 3 native package");
    }
    let lock = std::str::from_utf8(graph.lock).map_err(|_| "UTF-8 pnpm lockfile")?;
    if lock.matches("  npm/builder/parcel-native:\n").count() != 1
        || lock.matches("  parcel@3.0.0-alpha.2:\n").count() != 2
    {
        return Err("one exact Parcel importer and package");
    }
    let cargo_workspace =
        std::str::from_utf8(graph.cargo_workspace).map_err(|_| "UTF-8 Cargo workspace manifest")?;
    if cargo_workspace
        .matches("  \"npm/builder/parcel-native\",\n")
        .count()
        != 1
        || cargo_workspace
            .matches("parcel-plugin = \"=0.1.0\"\n")
            .count()
            != 1
    {
        return Err("one declared Parcel Cargo workspace member");
    }
    let cargo_lock = std::str::from_utf8(graph.cargo_lock).map_err(|_| "UTF-8 Cargo lockfile")?;
    for package in [
        "name = \"parcel-plugin\"\n",
        "name = \"sha2-const-stable\"\n",
        "name = \"vize_parcel_transformer\"\n",
    ] {
        if cargo_lock.matches(package).count() != 1 {
            return Err("exact Parcel Cargo lock packages");
        }
    }
    Ok(())
}

pub(super) fn authorize(previous: &Value, lock: &[u8], package: &[u8]) -> Checked<Option<Value>> {
    if let Some(previous_lock) =
        crate::package_task_history::before_parser_free_builders("pnpm-lock.yaml", lock)
    {
        let predecessor = authorize(previous, &previous_lock, package)?
            .ok_or("the parser-free builders require a qualified predecessor")?;
        let successor = crate::package_task_history::qualification_successor;
        return successor(&predecessor, &previous_lock, lock).map(Some);
    }
    if let Some(previous_lock) =
        crate::package_task_history::before_vite_resolver_importer("pnpm-lock.yaml", lock)
    {
        let predecessor = authorize(previous, &previous_lock, package)?
            .ok_or("the Vite resolver importer requires a qualified predecessor")?;
        if !bound(
            &predecessor["finalQualificationLock"],
            "pnpm-lock.yaml",
            &previous_lock,
        ) {
            return Err("exact Vite resolver importer qualification predecessor");
        }
        return Ok(Some(json!({
            "version": 1,
            "kind": "exact-provider-qualification-vite-resolver-importer-successor",
            "nativeAcceptanceCredit": false,
            "previousQualificationLock": predecessor["finalQualificationLock"],
            "finalQualificationLock": { "path": "pnpm-lock.yaml", "bytes": lock.len(), "sha256": support::hash(lock) },
            "finalQualificationPackage": predecessor["finalQualificationPackage"],
            "workspaceManifest": predecessor["workspaceManifest"],
            "importer": "npm/builder/vite",
            "dependency": "@rolldown/pluginutils",
            "specifier": "1.0.1",
            "resolvedVersion": "1.0.1",
        })));
    }
    if let Some(previous_lock) =
        crate::package_task_history::before_translation_worker("pnpm-lock.yaml", lock)
    {
        let previous_update = authorize(previous, &previous_lock, package)?
            .ok_or("the translation worker requires a qualified dependency graph")?;
        return translation_worker::successor(&previous_update, &previous_lock, lock).map(Some);
    }
    if let Ok(previous_lock) = entry_packages::reconstruct_lock_predecessor(lock) {
        let previous_update = authorize(previous, &previous_lock, package)?
            .ok_or("entry packages require a qualification predecessor")?;
        let workspace = fs::read(support::repository().join(WORKSPACE))
            .map_err(|_| "complete workspace manifest")?;
        return entry_packages::authorize_successor(
            &previous_update,
            &previous_lock,
            lock,
            package,
            &workspace,
        )
        .map(Some);
    }
    if final_graph(previous, lock, package).is_ok() {
        return Ok(None);
    }
    let predecessor = oracle_update();
    let jsx_oracle =
        fs::read(support::repository().join(JSX_ORACLE)).map_err(|_| "complete JSX oracle")?;
    if verify_oracle_update(previous, &predecessor, lock, package, &jsx_oracle).is_ok() {
        return Ok(Some(predecessor));
    }

    let predecessor_bytes = fs::read(support::corpus().join(ORACLE_UPDATE))
        .map_err(|_| "complete qualification predecessor")?;
    let predecessor = lockfile_update();
    let current_workspace = fs::read(support::repository().join(WORKSPACE))
        .map_err(|_| "complete workspace manifest")?;
    let current_workspace = entry_packages::reconstruct_workspace_predecessor(&current_workspace)
        .unwrap_or(current_workspace);
    let picomatch_predecessor =
        picomatch_security::reconstruct_workspace_predecessor(&current_workspace)
            .unwrap_or_else(|_| current_workspace.clone());
    let runtime_predecessor =
        runtime_security::reconstruct_workspace_predecessor(&picomatch_predecessor)
            .unwrap_or(picomatch_predecessor);
    let workspace = node_forge_security::reconstruct_workspace_predecessor(&runtime_predecessor)
        .unwrap_or(runtime_predecessor);
    let tests_package = fs::read(support::repository().join(TESTS_PACKAGE))
        .map_err(|_| "complete tests manifest")?;
    let oracle = fs::read(support::repository().join(SETUP_AWAIT_ORACLE))
        .map_err(|_| "complete setup-await oracle manifest")?;
    let support_script = fs::read(support::repository().join(SETUP_AWAIT_SUPPORT))
        .map_err(|_| "complete setup-await support script")?;
    let graph = QualificationGraph {
        lock,
        package,
        workspace: &workspace,
        tests_package: &tests_package,
        oracle: &oracle,
        support_script: &support_script,
    };
    if verify_lockfile_update(
        previous,
        &oracle_update(),
        &predecessor_bytes,
        &predecessor,
        &graph,
    )
    .is_ok()
    {
        return Ok(Some(predecessor));
    }

    let predecessor_bytes = fs::read(support::corpus().join(LOCKFILE_UPDATE))
        .map_err(|_| "complete Parcel qualification predecessor")?;
    let update = parcel_update();
    let parcel_manifest = fs::read(support::repository().join(PARCEL_MANIFEST))
        .map_err(|_| "complete Parcel native manifest")?;
    let cargo_workspace = fs::read(support::repository().join(CARGO_WORKSPACE))
        .map_err(|_| "complete Cargo workspace manifest")?;
    let cargo_lock =
        fs::read(support::repository().join(CARGO_LOCK)).map_err(|_| "complete Cargo lockfile")?;
    let graph = ParcelQualificationGraph {
        lock,
        package,
        parcel_manifest: &parcel_manifest,
        cargo_workspace: &cargo_workspace,
        cargo_lock: &cargo_lock,
    };
    if verify_parcel_update(&predecessor, &predecessor_bytes, &update, &graph).is_ok() {
        return Ok(Some(update));
    }

    let predecessor_bytes = fs::read(support::corpus().join(PARCEL_UPDATE))
        .map_err(|_| "complete JavaScript tooling qualification predecessor")?;
    let update = tooling_update();
    let babel_manifest = fs::read(support::repository().join(BABEL_MANIFEST))
        .map_err(|_| "complete Babel manifest")?;
    let eslint_manifest = fs::read(support::repository().join(ESLINT_MANIFEST))
        .map_err(|_| "complete ESLint manifest")?;
    let oxlint_manifest = fs::read(support::repository().join(OXLINT_MANIFEST))
        .map_err(|_| "complete Oxlint manifest")?;
    let prettier_manifest = fs::read(support::repository().join(PRETTIER_MANIFEST))
        .map_err(|_| "complete Prettier manifest")?;
    let graph = ToolingQualificationGraph {
        lock,
        package,
        workspace: &workspace,
        babel_manifest: &babel_manifest,
        eslint_manifest: &eslint_manifest,
        oxlint_manifest: &oxlint_manifest,
        prettier_manifest: &prettier_manifest,
    };
    if verify_tooling_update(&parcel_update(), &predecessor_bytes, &update, &graph).is_ok() {
        return Ok(Some(update));
    }

    let predecessor_bytes = fs::read(support::corpus().join(TOOLING_UPDATE))
        .map_err(|_| "complete Fresco qualification predecessor")?;
    let update = fresco_update();
    let fresco_manifest = fs::read(support::repository().join(FRESCO_MANIFEST))
        .map_err(|_| "complete Fresco manifest")?;
    let graph = FrescoQualificationGraph {
        lock,
        package,
        fresco_manifest: &fresco_manifest,
    };
    if verify_fresco_update(&tooling_update(), &predecessor_bytes, &update, &graph).is_ok() {
        return Ok(Some(update));
    }

    let predecessor_bytes = fs::read(support::corpus().join(FRESCO_UPDATE))
        .map_err(|_| "complete root dependency ownership predecessor")?;
    let update = dependency_ownership_update();
    let owner_bytes = OWNER_MANIFESTS
        .iter()
        .map(|path| {
            fs::read(support::repository().join(path))
                .map_err(|_| "complete root dependency owner manifest")
        })
        .collect::<Result<Vec<_>, _>>()?;
    let owners = OWNER_MANIFESTS
        .iter()
        .zip(&owner_bytes)
        .map(|(path, bytes)| (*path, bytes.as_slice()))
        .collect::<Vec<_>>();
    let graph = OwnershipQualificationGraph {
        lock,
        package,
        workspace: &workspace,
        owners: &owners,
    };
    if verify_dependency_ownership_update(&fresco_update(), &predecessor_bytes, &update, &graph)
        .is_ok()
    {
        return Ok(Some(update));
    }

    let predecessor_bytes = fs::read(support::corpus().join(dependency_ownership::UPDATE))
        .map_err(|_| "complete strict checker qualification predecessor")?;
    let strict_update = strict_checker_update();
    if authorize_strict_checker(&update, &predecessor_bytes, lock, package, &workspace).is_ok() {
        return Ok(Some(strict_update));
    }
    let predecessor_bytes = fs::read(support::corpus().join(docs_metadata::UPDATE))
        .map_err(|_| "complete JavaScript tooling refresh predecessor")?;
    if let Ok(update) = js_tooling_refresh::authorize(
        &docs_metadata_update(),
        &predecessor_bytes,
        lock,
        package,
        &workspace,
    ) {
        return Ok(Some(update));
    }

    if let Ok(update) = picomatch_security::authorize_successor(lock, package, &current_workspace) {
        return Ok(Some(update));
    }
    if let Ok(update) = runtime_security::authorize_successor(lock, package, &current_workspace) {
        return Ok(Some(update));
    }
    if let Ok(update) = node_forge_security::authorize_successor(lock, package, &current_workspace)
    {
        return Ok(Some(update));
    }
    eslint_vue_scorecard::authorize_successor(lock, package, &workspace).map(Some)
}

#[cfg(test)]
mod tests;
