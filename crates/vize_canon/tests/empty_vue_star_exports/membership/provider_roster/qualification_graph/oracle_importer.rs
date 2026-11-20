//! Qualification of the historical JSX oracle importer successor.

use super::{Checked, Value, bound, final_graph};

pub(super) const PREVIOUS_PROVIDER: &str =
    "ad428607e450f807b15e9301d429fcbee7fe67729b52130f2d7e63c2f0cf1e0f";
pub(super) const UPDATE_BINDING: &str = "tests/_fixtures/differential/typechecker/empty-vue-star-exports/malformed-membership/provider-oracle-importer-update-11172.json";
pub(super) const MANIFEST: &str = "tests/tooling/jsx-vapor-oracle/package.json";

const ADDED: &str =
    "      '@playwright/test':\n        specifier: catalog:testing\n        version: 1.62.1\n";
const PREFIX: &str = "  tests/tooling/jsx-vapor-oracle:\n    devDependencies:\n      '@babel/core':\n        specifier: catalog:babel-jsx-oracle\n        version: 7.29.0(supports-color@10.2.2)\n";

pub(super) fn verify(
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
        || !bound(&update["oracleImporter"], MANIFEST, oracle)
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
