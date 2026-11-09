//! Frozen provider-qualification fixture loaders.

use super::{Value, fs, support};

pub(super) const ORACLE_UPDATE: &str =
    "malformed-membership/provider-oracle-importer-update-11172.json";
const ORACLE_UPDATE_HASH: &str = "d8e7a42a623f261eecd89b349224058993fd7ff2c008932d16c46dad29eae6a7";
pub(super) const LOCKFILE_UPDATE: &str =
    "malformed-membership/provider-qualification-lockfile-update-11542.json";
const LOCKFILE_UPDATE_HASH: &str =
    "1191a1ee7876f54a25cc10aeaa46cf8c8cc61643ebaf7a8562a5d7c98ff107af";
pub(super) const PARCEL_UPDATE: &str =
    "malformed-membership/provider-parcel-native-update-12011.json";
const PARCEL_UPDATE_HASH: &str = "e48ee44b774800a5b0624ca168280f15541f9348fea25e2a6d3a422d6bec438a";

fn load(path: &str, hash: &str, kind: &str) -> Value {
    let bytes = fs::read(support::corpus().join(path)).unwrap();
    assert_eq!(support::hash(&bytes), hash);
    let record: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(record["version"], 1);
    assert_eq!(record["kind"], kind);
    assert_eq!(record["nativeAcceptanceCredit"], false);
    record
}

pub(super) fn oracle_update() -> Value {
    load(
        ORACLE_UPDATE,
        ORACLE_UPDATE_HASH,
        "exact-provider-qualification-oracle-importer-successor",
    )
}

pub(super) fn lockfile_update() -> Value {
    load(
        LOCKFILE_UPDATE,
        LOCKFILE_UPDATE_HASH,
        "exact-provider-qualification-lockfile-successor",
    )
}

pub(super) fn parcel_update() -> Value {
    load(
        PARCEL_UPDATE,
        PARCEL_UPDATE_HASH,
        "exact-provider-qualification-parcel-native-successor",
    )
}
