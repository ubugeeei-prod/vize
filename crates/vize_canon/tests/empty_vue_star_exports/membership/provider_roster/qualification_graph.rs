//! A source-only graph successor; every provider and native membership gate remains live.
use super::{Checked, Value, final_graph, fs, json, support};

const PREVIOUS: &str = "ad428607e450f807b15e9301d429fcbee7fe67729b52130f2d7e63c2f0cf1e0f";
const ORACLE: &str = "tests/tooling/jsx-vapor-oracle/package.json";
const ADDED: &str =
    "      '@playwright/test':\n        specifier: catalog:testing\n        version: 1.62.1\n";
const PREFIX: &str = "  tests/tooling/jsx-vapor-oracle:\n    devDependencies:\n      '@babel/core':\n        specifier: catalog:babel-jsx-oracle\n        version: 7.29.0(supports-color@10.2.2)\n";

fn load() -> Value {
    let bytes = fs::read(
        support::corpus().join("malformed-membership/provider-oracle-importer-update-11172.json"),
    )
    .unwrap();
    assert_eq!(
        support::hash(&bytes),
        "d8e7a42a623f261eecd89b349224058993fd7ff2c008932d16c46dad29eae6a7"
    );
    let record: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(record["version"], 1);
    assert_eq!(
        record["kind"],
        "exact-provider-qualification-oracle-importer-successor"
    );
    assert_eq!(record["nativeAcceptanceCredit"], false);
    record
}

fn bound(binding: &Value, path: &str, body: &[u8]) -> bool {
    binding["path"] == path
        && binding["bytes"] == json!(body.len())
        && binding["sha256"] == support::hash(body)
}

fn verify(
    previous: &Value,
    update: &Value,
    lock: &[u8],
    package: &[u8],
    oracle: &[u8],
) -> Checked<()> {
    if update["previousProviderRecord"]["sha256"] != PREVIOUS
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
        || !bound(&update["oracleImporter"], ORACLE, oracle)
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

pub(super) fn authorize(previous: &Value, lock: &[u8], package: &[u8]) -> Checked<Option<Value>> {
    if final_graph(previous, lock, package).is_ok() {
        return Ok(None);
    }
    let update = load();
    let oracle =
        fs::read(support::repository().join(ORACLE)).map_err(|_| "complete oracle manifest")?;
    verify(previous, &update, lock, package, &oracle)?;
    Ok(Some(update))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inputs() -> (Value, Value, Vec<u8>, Vec<u8>, Vec<u8>) {
        (
            super::super::load(),
            load(),
            fs::read(support::repository().join("pnpm-lock.yaml")).unwrap(),
            fs::read(support::repository().join("package.json")).unwrap(),
            fs::read(support::repository().join(ORACLE)).unwrap(),
        )
    }

    #[test]
    fn exact_importer_successor_reconstructs_the_whole_frozen_graph() {
        let (previous, update, lock, package, oracle) = inputs();
        assert_eq!(verify(&previous, &update, &lock, &package, &oracle), Ok(()));
        assert_eq!(
            authorize(&previous, &lock, &package),
            Ok(Some(update.clone()))
        );
        let offset = update["insertion"]["offset"].as_u64().unwrap() as usize;
        let before = [
            lock.get(..offset).unwrap(),
            lock.get(offset + ADDED.len()..).unwrap(),
        ]
        .concat();
        assert_eq!(final_graph(&previous, &before, &package), Ok(()));
        assert_eq!(authorize(&previous, &before, &package), Ok(None));
    }

    #[test]
    fn changed_graph_oracle_position_and_foreign_insertions_are_refused() {
        let (previous, update, lock, package, oracle) = inputs();
        for changed in [
            b"foreign lock".as_slice(),
            lock.get(..lock.len() - 1).unwrap(),
        ] {
            assert_eq!(
                verify(&previous, &update, changed, &package, &oracle),
                Err("exact current qualification and oracle bytes")
            );
        }
        assert_eq!(
            verify(&previous, &update, &lock, b"foreign root package", &oracle),
            Err("exact current qualification and oracle bytes")
        );
        assert_eq!(
            verify(&previous, &update, &lock, &package, b"foreign oracle"),
            Err("exact current qualification and oracle bytes")
        );
        let mut shifted = update.clone();
        shifted["insertion"]["offset"] = json!(update["insertion"]["offset"].as_u64().unwrap() + 1);
        assert_eq!(
            verify(&previous, &shifted, &lock, &package, &oracle),
            Err("exact positioned oracle importer insertion")
        );
        let mut foreign = update.clone();
        foreign["insertion"]["addedUtf8"] = json!("unreviewed catalog dependency");
        assert_eq!(
            verify(&previous, &foreign, &lock, &package, &oracle),
            Err("only the three declared catalog importer lines")
        );
        foreign = update.clone();
        foreign["insertion"]["precedingUtf8"] = json!("");
        assert_eq!(
            verify(&previous, &foreign, &lock, &package, &oracle),
            Err("only the declared oracle importer")
        );
        let mut edited = lock.clone();
        *edited.first_mut().unwrap() ^= 1;
        let mut rehashed = update.clone();
        rehashed["finalQualificationLock"]["sha256"] = json!(support::hash(&edited));
        assert_eq!(
            verify(&previous, &rehashed, &edited, &package, &oracle),
            Err("complete original qualification graph reconstruction")
        );
    }
}
