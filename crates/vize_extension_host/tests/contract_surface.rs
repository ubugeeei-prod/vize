//! The released `vize:contracts` surfaces, versioned alongside the WIT
//! (P6-8, `docs/davinci/contracts-compat-policy.md`).
//!
//! `crates/vize_extension_sdk/versions/` holds one canonical surface per
//! released package version. This test holds the history to the policy — every consecutive
//! pair is classified and its version movement checked — and pins the newest
//! entry to what `crates/vize_extension_sdk/wit/` and this host's handshake
//! constants describe today, byte for byte. Changing the WIT therefore means choosing
//! a version the policy accepts and committing its surface
//! (`VIZE_CONTRACT_SURFACE_BLESS=1` writes it for review).

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use vize_extension_host::contract::{
    PROTOCOL_VERSION, REQUIRED_FEATURES, S1_PAGE_SCHEMA, S2_PAGE_SCHEMA,
};
use vize_extension_host::expression::{self, FACTS_PAGE_SCHEMA, PROJECTION_PAGE_SCHEMA};
use vize_extension_host::output::{self, EMIT_DOCUMENT_PAGE_SCHEMA, S3_PAGE_SCHEMA};
use vize_marquette::contracts::wit::{Protocol, surface_from_wit};
use vize_marquette::{
    ContractSurface, ContractVersion, canonical_surface_json, check_version_policy,
    compare_surfaces,
};
use vize_s0::{String, cstr};

const BLESS_ENV: &str = "VIZE_CONTRACT_SURFACE_BLESS";

fn sdk() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../vize_extension_sdk")
}

fn features(required: &[&str]) -> BTreeSet<String> {
    required.iter().copied().map(String::from).collect()
}

/// What this host implements beside the WIT.
fn host_protocol() -> Protocol {
    Protocol {
        protocol_version: PROTOCOL_VERSION,
        pages: BTreeMap::from([
            (
                String::from("emit-document-page"),
                EMIT_DOCUMENT_PAGE_SCHEMA,
            ),
            (String::from("facts-page"), FACTS_PAGE_SCHEMA),
            (String::from("projection-page"), PROJECTION_PAGE_SCHEMA),
            (String::from("s1-page"), S1_PAGE_SCHEMA),
            (String::from("s2-page"), S2_PAGE_SCHEMA),
            (String::from("s3-page"), S3_PAGE_SCHEMA),
        ]),
        required_features: BTreeMap::from([
            (
                String::from("expression-dialect"),
                features(expression::REQUIRED_FEATURES),
            ),
            (String::from("input-dialect"), features(REQUIRED_FEATURES)),
            (
                String::from("output-target"),
                features(output::REQUIRED_FEATURES),
            ),
        ]),
    }
}

/// Every committed surface, oldest first, with its file bytes.
fn released() -> Vec<(ContractVersion, ContractSurface, Vec<u8>)> {
    let mut released: Vec<_> = std::fs::read_dir(sdk().join("versions"))
        .expect("crates/vize_extension_sdk/versions exists")
        .map(|entry| {
            let path = entry.expect("readable entry").path();
            let bytes = std::fs::read(&path).expect("readable surface");
            let surface: ContractSurface = serde_json::from_slice(&bytes)
                .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            assert_eq!(
                name,
                cstr!(
                    "{}@{}.json",
                    surface.package.replace(':', "-"),
                    surface.version
                ),
                "a released surface is named after its package and version"
            );
            let version = ContractVersion::parse(&surface.version).expect("semver");
            (version, surface, bytes)
        })
        .collect();
    released.sort_by_key(|(version, ..)| *version);
    released
}

#[test]
fn released_surfaces_are_canonical_and_follow_the_policy() {
    let released = released();
    assert_eq!(
        released
            .iter()
            .map(|(version, ..)| cstr!("{version}"))
            .collect::<Vec<_>>(),
        ["0.1.0", "0.1.1", "0.1.2"]
    );
    for (_, surface, bytes) in &released {
        assert_eq!(
            &canonical_surface_json(surface),
            bytes,
            "{} is canonical",
            surface.version
        );
    }
    for pair in released.windows(2) {
        let (previous, next) = (&pair[0].1, &pair[1].1);
        let report = compare_surfaces(previous, next);
        let violations: Vec<String> = check_version_policy(previous, next, &report)
            .iter()
            .map(|violation| cstr!("{violation}"))
            .collect();
        assert_eq!(
            violations,
            Vec::<String>::new(),
            "{} -> {}",
            previous.version,
            next.version
        );
    }
}

#[test]
fn the_newest_released_surface_is_the_wit_and_this_host() {
    let current = surface_from_wit(&sdk().join("wit"), &host_protocol())
        .unwrap_or_else(|error| panic!("crates/vize_extension_sdk/wit: {error}"));
    let canonical = canonical_surface_json(&current);
    if std::env::var_os(BLESS_ENV).is_some() {
        let name = cstr!(
            "{}@{}.json",
            current.package.replace(':', "-"),
            current.version
        );
        std::fs::write(sdk().join("versions").join(name.as_str()), &canonical)
            .expect("writes the surface");
    }
    let released = released();
    let (_, newest, bytes) = released.last().expect("a released surface");
    assert_eq!(
        (&current.package, &current.version),
        (&newest.package, &newest.version),
        "the WIT package version has no released surface: choose the version the policy \
         requires and rerun with {BLESS_ENV}=1"
    );
    assert_eq!(
        std::str::from_utf8(&canonical).expect("UTF-8"),
        std::str::from_utf8(bytes).expect("UTF-8"),
        "crates/vize_extension_sdk/wit changed without a new version"
    );
}
