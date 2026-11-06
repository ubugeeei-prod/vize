//! Source-only graph successors; every provider and native membership gate remains live.
use super::{Checked, Value, final_graph, fs, json, support};

const PREVIOUS_PROVIDER: &str = "ad428607e450f807b15e9301d429fcbee7fe67729b52130f2d7e63c2f0cf1e0f";
const ORACLE_UPDATE: &str = "malformed-membership/provider-oracle-importer-update-11172.json";
const ORACLE_UPDATE_BINDING: &str = "tests/_fixtures/differential/typechecker/empty-vue-star-exports/malformed-membership/provider-oracle-importer-update-11172.json";
const ORACLE_UPDATE_HASH: &str = "d8e7a42a623f261eecd89b349224058993fd7ff2c008932d16c46dad29eae6a7";
const LOCKFILE_UPDATE: &str =
    "malformed-membership/provider-qualification-lockfile-update-11542.json";
const LOCKFILE_UPDATE_HASH: &str =
    "1191a1ee7876f54a25cc10aeaa46cf8c8cc61643ebaf7a8562a5d7c98ff107af";
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

#[allow(clippy::too_many_arguments)]
fn verify_lockfile_update(
    previous: &Value,
    predecessor: &Value,
    predecessor_bytes: &[u8],
    update: &Value,
    lock: &[u8],
    package: &[u8],
    workspace: &[u8],
    tests_package: &[u8],
    oracle: &[u8],
    support_script: &[u8],
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
        (&update["finalQualificationLock"], "pnpm-lock.yaml", lock),
        (
            &update["finalQualificationPackage"],
            "package.json",
            package,
        ),
        (&update["workspaceManifest"], WORKSPACE, workspace),
        (&update["testsManifest"], TESTS_PACKAGE, tests_package),
        (&update["oracleManifest"], SETUP_AWAIT_ORACLE, oracle),
        (
            &update["setupAwaitSupport"],
            SETUP_AWAIT_SUPPORT,
            support_script,
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

    let workspace = std::str::from_utf8(workspace).map_err(|_| "UTF-8 workspace manifest")?;
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
        serde_json::from_slice(tests_package).map_err(|_| "complete tests manifest")?;
    if tests_package["devDependencies"]["vue-setup-await-oracle"] != Value::Null {
        return Err("no shared setup-await oracle alias");
    }
    let oracle: Value =
        serde_json::from_slice(oracle).map_err(|_| "complete setup-await oracle manifest")?;
    let dependencies = oracle["devDependencies"]
        .as_object()
        .ok_or("two exact setup-await dependencies")?;
    if dependencies.len() != 2
        || dependencies.get("vue") != Some(&json!("3.5.38"))
        || dependencies.get("@vue/server-renderer") != Some(&json!("3.5.38"))
    {
        return Err("two exact setup-await dependencies");
    }
    let lock = std::str::from_utf8(lock).map_err(|_| "UTF-8 qualification lock")?;
    if lock.matches("  '@unhead/vue': 3.4.3").count() != 1
        || lock.matches(SETUP_AWAIT_IMPORTER).count() != 1
        || lock.contains("      vue-setup-await-oracle:\n        specifier: npm:vue@3.5.38")
    {
        return Err("exact generated lockfile boundary");
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
    let update = lockfile_update();
    let workspace = fs::read(support::repository().join(WORKSPACE))
        .map_err(|_| "complete workspace manifest")?;
    let tests_package = fs::read(support::repository().join(TESTS_PACKAGE))
        .map_err(|_| "complete tests manifest")?;
    let oracle = fs::read(support::repository().join(SETUP_AWAIT_ORACLE))
        .map_err(|_| "complete setup-await oracle manifest")?;
    let support_script = fs::read(support::repository().join(SETUP_AWAIT_SUPPORT))
        .map_err(|_| "complete setup-await support script")?;
    verify_lockfile_update(
        previous,
        &predecessor,
        &predecessor_bytes,
        &update,
        lock,
        package,
        &workspace,
        &tests_package,
        &oracle,
        &support_script,
    )?;
    Ok(Some(update))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Inputs {
        previous: Value,
        predecessor: Value,
        predecessor_bytes: Vec<u8>,
        update: Value,
        lock: Vec<u8>,
        package: Vec<u8>,
        workspace: Vec<u8>,
        tests_package: Vec<u8>,
        oracle: Vec<u8>,
        support_script: Vec<u8>,
    }

    fn inputs() -> Inputs {
        Inputs {
            previous: super::super::load(),
            predecessor: oracle_update(),
            predecessor_bytes: fs::read(support::corpus().join(ORACLE_UPDATE)).unwrap(),
            update: lockfile_update(),
            lock: fs::read(support::repository().join("pnpm-lock.yaml")).unwrap(),
            package: fs::read(support::repository().join("package.json")).unwrap(),
            workspace: fs::read(support::repository().join(WORKSPACE)).unwrap(),
            tests_package: fs::read(support::repository().join(TESTS_PACKAGE)).unwrap(),
            oracle: fs::read(support::repository().join(SETUP_AWAIT_ORACLE)).unwrap(),
            support_script: fs::read(support::repository().join(SETUP_AWAIT_SUPPORT)).unwrap(),
        }
    }

    fn verify(input: &Inputs, update: &Value, lock: &[u8]) -> Checked<()> {
        verify_lockfile_update(
            &input.previous,
            &input.predecessor,
            &input.predecessor_bytes,
            update,
            lock,
            &input.package,
            &input.workspace,
            &input.tests_package,
            &input.oracle,
            &input.support_script,
        )
    }

    #[test]
    fn exact_lockfile_successor_binds_the_whole_current_graph() {
        let input = inputs();
        assert_eq!(verify(&input, &input.update, &input.lock), Ok(()));
        assert_eq!(
            authorize(&input.previous, &input.lock, &input.package),
            Ok(Some(input.update.clone()))
        );
    }

    #[test]
    fn historical_oracle_successor_remains_an_exact_predecessor() {
        let input = inputs();
        assert_eq!(support::hash(&input.predecessor_bytes), ORACLE_UPDATE_HASH);
        assert_eq!(
            input.update["previousProviderQualificationRecord"]["sha256"],
            ORACLE_UPDATE_HASH
        );
        assert_eq!(
            input.update["previousQualificationLock"],
            input.predecessor["finalQualificationLock"]
        );
    }

    #[test]
    fn changed_graph_predecessor_and_fixed_point_are_refused() {
        let input = inputs();
        for changed in [
            b"foreign lock".as_slice(),
            input.lock.get(..input.lock.len() - 1).unwrap(),
        ] {
            assert_eq!(
                verify(&input, &input.update, changed),
                Err("exact current qualification graph bytes")
            );
        }

        let mut predecessor = input.predecessor.clone();
        predecessor["finalQualificationLock"]["sha256"] = json!(support::hash(b"foreign"));
        assert_eq!(
            verify_lockfile_update(
                &input.previous,
                &predecessor,
                &input.predecessor_bytes,
                &input.update,
                &input.lock,
                &input.package,
                &input.workspace,
                &input.tests_package,
                &input.oracle,
                &input.support_script,
            ),
            Err("exact frozen qualification successor")
        );

        let mut resolver = input.update.clone();
        resolver["resolver"]["fixedPointSha256"] = json!(support::hash(b"foreign"));
        assert_eq!(
            verify(&input, &resolver, &input.lock),
            Err("exact resolver fixed point")
        );
    }

    #[test]
    fn shared_alias_foreign_manifests_and_unreviewed_bytes_are_refused() {
        let input = inputs();
        let mut changed = input.update.clone();
        changed["workspaceManifest"]["sha256"] = json!(support::hash(b"foreign"));
        assert_eq!(
            verify(&input, &changed, &input.lock),
            Err("exact current qualification graph bytes")
        );

        let mut lock = input.lock.clone();
        *lock.first_mut().unwrap() ^= 1;
        let mut rehashed = input.update.clone();
        rehashed["finalQualificationLock"]["sha256"] = json!(support::hash(&lock));
        assert_eq!(
            verify(&input, &rehashed, &lock),
            Err("exact resolver fixed point")
        );
    }
}
