//! The contract version policy (P6-8), over hand-built surfaces: how the
//! package version and the protocol version must move for an additive or a
//! breaking change, with every violation's exact message.

#![expect(clippy::expect_used, reason = "tests assert by panicking")]

use std::collections::BTreeMap;

use vize_marquette::contracts::{InterfaceSurface, TypeShape};
use vize_marquette::{
    CONTRACT_SURFACE_FORMAT, CONTRACT_SURFACE_FORMAT_VERSION, CompatibilityChangeKind,
    ContractSurface, ContractVersion, canonical_surface_json, check_version_policy,
    compare_surfaces, surface_fingerprint,
};
use vize_s0::{String, cstr};

fn surface(version: &str, protocol_version: u32, severity: &[&str]) -> ContractSurface {
    let mut types = BTreeMap::new();
    types.insert(
        String::from("severity"),
        TypeShape::Enum(severity.iter().copied().map(String::from).collect()),
    );
    let mut interfaces = BTreeMap::new();
    interfaces.insert(
        String::from("types"),
        InterfaceSurface {
            types,
            functions: BTreeMap::new(),
        },
    );
    ContractSurface {
        format: String::from(CONTRACT_SURFACE_FORMAT),
        format_version: CONTRACT_SURFACE_FORMAT_VERSION,
        package: String::from("fixture:contracts"),
        version: String::from(version),
        protocol_version,
        pages: BTreeMap::new(),
        interfaces,
        worlds: BTreeMap::new(),
    }
}

fn violations(previous: &ContractSurface, next: &ContractSurface) -> Vec<String> {
    let report = compare_surfaces(previous, next);
    check_version_policy(previous, next, &report)
        .iter()
        .map(|violation| cstr!("{violation}"))
        .collect()
}

fn version(text: &str) -> ContractVersion {
    ContractVersion::parse(text).expect("a MAJOR.MINOR.PATCH version")
}

#[test]
fn versions_parse_only_as_canonical_major_minor_patch() {
    assert_eq!(
        ContractVersion::parse("1.20.3"),
        Some(ContractVersion {
            major: 1,
            minor: 20,
            patch: 3
        })
    );
    for text in [
        "1.2",
        "1.2.3.4",
        "01.2.3",
        "1.2.3-rc.1",
        "1.2.x",
        "",
        "1..3",
    ] {
        assert_eq!(ContractVersion::parse(text), None, "{text:?}");
    }
}

#[test]
fn the_least_next_version_follows_the_zero_major_rule() {
    use CompatibilityChangeKind::{Additive, Breaking};
    let cases = [
        ("0.1.4", Additive, "0.1.5"),
        ("0.1.4", Breaking, "0.2.0"),
        ("1.3.4", Additive, "1.4.0"),
        ("1.3.4", Breaking, "2.0.0"),
    ];
    for (from, kind, least) in cases {
        assert_eq!(cstr!("{}", version(from).least_after(kind)), least);
    }
}

const ORIGINAL: &[&str] = &["error", "warning"];
const EXTENDED: &[&str] = &["error", "warning", "note"];

#[test]
fn a_breaking_change_needs_the_breaking_bump_and_a_new_protocol() {
    let base = surface("0.1.0", 1, ORIGINAL);
    assert_eq!(
        violations(&base, &surface("0.1.1", 1, EXTENDED)),
        [
            "a breaking change needs version 0.2.0 or later, found 0.1.1",
            "a breaking change must raise the protocol version by exactly one: 1 -> 1",
        ]
    );
    assert_eq!(
        violations(&base, &surface("0.2.0", 3, EXTENDED)),
        ["a breaking change must raise the protocol version by exactly one: 1 -> 3"]
    );
    assert_eq!(
        violations(&base, &surface("0.2.0", 2, EXTENDED)),
        Vec::<String>::new()
    );
    let stable = surface("1.4.2", 7, ORIGINAL);
    assert_eq!(
        violations(&stable, &surface("1.5.0", 8, EXTENDED)),
        ["a breaking change needs version 2.0.0 or later, found 1.5.0"]
    );
    assert_eq!(
        violations(&stable, &surface("2.0.0", 8, EXTENDED)),
        Vec::<String>::new()
    );
}

#[test]
fn the_version_must_move_with_any_change_and_never_backwards() {
    let base = surface("0.1.0", 1, ORIGINAL);
    assert_eq!(
        violations(&base, &surface("0.1.0", 2, EXTENDED)),
        ["the surface changed but the version stayed 0.1.0"]
    );
    assert_eq!(
        violations(
            &surface("0.2.0", 1, ORIGINAL),
            &surface("0.1.9", 1, ORIGINAL)
        ),
        ["the version must not decrease: 0.2.0 -> 0.1.9"]
    );
    assert_eq!(
        violations(&base, &surface("0.1", 1, ORIGINAL)),
        ["the next version \"0.1\" is not MAJOR.MINOR.PATCH"]
    );
    assert_eq!(violations(&base, &base.clone()), Vec::<String>::new());
    assert_eq!(
        violations(&base, &surface("0.1.3", 1, ORIGINAL)),
        Vec::<String>::new()
    );
}

#[test]
fn canonical_json_is_stable_and_round_trips() {
    let base = surface("0.1.0", 1, ORIGINAL);
    let bytes = canonical_surface_json(&base);
    assert_eq!(
        std::str::from_utf8(&bytes).expect("UTF-8"),
        "{\n  \"format\": \"vize.contract-surface\",\n  \"formatVersion\": 1,\n  \
         \"package\": \"fixture:contracts\",\n  \"version\": \"0.1.0\",\n  \
         \"protocolVersion\": 1,\n  \"pages\": {},\n  \"interfaces\": {\n    \"types\": {\n      \
         \"types\": {\n        \"severity\": {\n          \"enum\": [\n            \"error\",\n            \
         \"warning\"\n          ]\n        }\n      },\n      \"functions\": {}\n    }\n  },\n  \
         \"worlds\": {}\n}\n"
    );
    let parsed: ContractSurface = serde_json::from_slice(&bytes).expect("parses");
    assert_eq!(parsed, base);
    assert_eq!(surface_fingerprint(&parsed), surface_fingerprint(&base));
    assert_ne!(
        surface_fingerprint(&base),
        surface_fingerprint(&surface("0.1.0", 1, EXTENDED))
    );
    let unknown = br#"{"format":"vize.contract-surface","formatVersion":1,"package":"p","version":"0.1.0","protocolVersion":1,"pages":{},"interfaces":{},"worlds":{},"extra":1}"#;
    assert_eq!(
        serde_json::from_slice::<ContractSurface>(unknown)
            .map_err(|error| cstr!("{error}"))
            .map(|_| ()),
        Err(String::from(
            "unknown field `extra`, expected one of `format`, `formatVersion`, `package`, \
             `version`, `protocolVersion`, `pages`, `interfaces`, `worlds` at line 1 column 150"
        ))
    );
}
