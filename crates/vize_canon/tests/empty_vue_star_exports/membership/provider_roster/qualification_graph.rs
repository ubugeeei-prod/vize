//! Source-only graph successors; every provider and native membership gate remains live.
use super::{Checked, Value, final_graph, fs, json, support};

mod fresco;
use fresco::{
    FRESCO_MANIFEST, FrescoQualificationGraph, TOOLING_UPDATE, TOOLING_UPDATE_HASH, fresco_update,
    verify_fresco_update,
};
mod tooling;
use tooling::{
    BABEL_MANIFEST, ESLINT_MANIFEST, OXLINT_MANIFEST, PRETTIER_MANIFEST, ToolingQualificationGraph,
    tooling_update, verify_tooling_update,
};

const PREVIOUS_PROVIDER: &str = "ad428607e450f807b15e9301d429fcbee7fe67729b52130f2d7e63c2f0cf1e0f";
const ORACLE_UPDATE: &str = "malformed-membership/provider-oracle-importer-update-11172.json";
const ORACLE_UPDATE_BINDING: &str = "tests/_fixtures/differential/typechecker/empty-vue-star-exports/malformed-membership/provider-oracle-importer-update-11172.json";
const ORACLE_UPDATE_HASH: &str = "d8e7a42a623f261eecd89b349224058993fd7ff2c008932d16c46dad29eae6a7";
const LOCKFILE_UPDATE: &str =
    "malformed-membership/provider-qualification-lockfile-update-11542.json";
const LOCKFILE_UPDATE_HASH: &str =
    "1191a1ee7876f54a25cc10aeaa46cf8c8cc61643ebaf7a8562a5d7c98ff107af";
const PARCEL_UPDATE: &str = "malformed-membership/provider-parcel-native-update-12011.json";
const PARCEL_UPDATE_HASH: &str = "e48ee44b774800a5b0624ca168280f15541f9348fea25e2a6d3a422d6bec438a";
const LOCKFILE_UPDATE_BINDING: &str = "tests/_fixtures/differential/typechecker/empty-vue-star-exports/malformed-membership/provider-qualification-lockfile-update-11542.json";
const PARCEL_MANIFEST: &str = "npm/builder/parcel-native/package.json";
const CARGO_WORKSPACE: &str = "Cargo.toml";
const CARGO_LOCK: &str = "Cargo.lock";
const JSX_ORACLE: &str = "tests/tooling/jsx-vapor-oracle/package.json";
const SETUP_AWAIT_ORACLE: &str = "tests/tooling/vue-setup-await-oracle/package.json";
const WORKSPACE: &str = "pnpm-workspace.yaml";
const TESTS_PACKAGE: &str = "tests/package.json";
const SETUP_AWAIT_SUPPORT: &str = "tests/tooling/support/setup-scope-await.mjs";
const ADDED: &str =
    "      '@playwright/test':\n        specifier: catalog:testing\n        version: 1.62.1\n";
const PREFIX: &str = "  tests/tooling/jsx-vapor-oracle:\n    devDependencies:\n      '@babel/core':\n        specifier: catalog:babel-jsx-oracle\n        version: 7.29.0(supports-color@10.2.2)\n";
const SETUP_AWAIT_IMPORTER: &str = "  tests/tooling/vue-setup-await-oracle:\n    devDependencies:\n      '@vue/server-renderer':\n        specifier: 3.5.38\n        version: 3.5.38(vue@3.5.38(typescript@6.0.3))\n      vue:\n        specifier: 3.5.38\n        version: 3.5.38(typescript@6.0.3)\n";

fn load(path: &str, hash: &str, kind: &str) -> Value {
    let bytes = fs::read(support::corpus().join(path)).unwrap();
    assert_eq!(support::hash(&bytes), hash);
    let record: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(record["version"], 1);
    assert_eq!(record["kind"], kind);
    assert_eq!(record["nativeAcceptanceCredit"], false);
    record
}

fn oracle_update() -> Value {
    load(
        ORACLE_UPDATE,
        ORACLE_UPDATE_HASH,
        "exact-provider-qualification-oracle-importer-successor",
    )
}

fn lockfile_update() -> Value {
    load(
        LOCKFILE_UPDATE,
        LOCKFILE_UPDATE_HASH,
        "exact-provider-qualification-lockfile-successor",
    )
}

fn parcel_update() -> Value {
    load(
        PARCEL_UPDATE,
        PARCEL_UPDATE_HASH,
        "exact-provider-qualification-parcel-native-successor",
    )
}

fn bound(binding: &Value, path: &str, body: &[u8]) -> bool {
    binding["path"] == path
        && binding["bytes"] == json!(body.len())
        && binding["sha256"] == support::hash(body)
}

fn verify_oracle_update(
    previous: &Value,
    update: &Value,
    lock: &[u8],
    package: &[u8],
    oracle: &[u8],
) -> Checked<()> {
    if update["previousProviderRecord"]["sha256"] != PREVIOUS_PROVIDER
        || update["previousQualificationLock"] != previous["finalQualificationLock"]
        || update["finalQualificationPackage"] != previous["finalQualificationPackage"]
    {
        return Err("exact frozen qualification predecessor");
    }
    if !bound(&update["finalQualificationLock"], "pnpm-lock.yaml", lock)
        || !bound(
            &update["finalQualificationPackage"],
            "package.json",
            package,
        )
        || !bound(&update["oracleImporter"], JSX_ORACLE, oracle)
    {
        return Err("exact current qualification and oracle bytes");
    }
    let oracle: Value = serde_json::from_slice(oracle).map_err(|_| "complete oracle manifest")?;
    if oracle["devDependencies"]["@playwright/test"] != "catalog:testing" {
        return Err("the declared catalog oracle dependency");
    }
    let insertion = &update["insertion"];
    if insertion["addedUtf8"] != ADDED {
        return Err("only the three declared catalog importer lines");
    }
    if insertion["precedingUtf8"] != PREFIX {
        return Err("only the declared oracle importer");
    }
    let offset = insertion["offset"]
        .as_u64()
        .and_then(|value| usize::try_from(value).ok())
        .ok_or("bounded insertion offset")?;
    let end = offset
        .checked_add(ADDED.len())
        .ok_or("bounded insertion end")?;
    let prefix = insertion["precedingUtf8"]
        .as_str()
        .ok_or("complete importer prefix")?;
    let prefix_start = offset
        .checked_sub(prefix.len())
        .ok_or("bounded importer prefix")?;
    if lock.get(offset..end) != Some(ADDED.as_bytes())
        || lock.get(prefix_start..offset) != Some(prefix.as_bytes())
    {
        return Err("exact positioned oracle importer insertion");
    }
    let before = lock.get(..offset).ok_or("bounded original prefix")?;
    let after = lock.get(end..).ok_or("bounded original suffix")?;
    let inverse = [before, after].concat();
    std::str::from_utf8(&inverse).map_err(|_| "whole UTF-8 predecessor")?;
    final_graph(previous, &inverse, package)
        .map_err(|_| "complete original qualification graph reconstruction")
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
    let workspace = fs::read(support::repository().join(WORKSPACE))
        .map_err(|_| "complete workspace manifest")?;
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
    let workspace = fs::read(support::repository().join(WORKSPACE))
        .map_err(|_| "complete workspace manifest")?;
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
    verify_fresco_update(&tooling_update(), &predecessor_bytes, &update, &graph)?;
    Ok(Some(update))
}

#[cfg(test)]
mod tests;
