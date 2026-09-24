//! P6-8 acceptance: contract surfaces read from WIT sources are classified
//! exactly — a deliberately breaking change is flagged (and its additive-
//! sized version bump refused), a purely additive one is not.

#![expect(clippy::panic, reason = "tests assert by panicking")]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use vize_marquette::contracts::wit::{Protocol, surface_from_wit};
use vize_marquette::{
    CompatibilityChange, CompatibilityChangeKind, ContractSurface, canonical_surface_json,
    check_version_policy, compare_surfaces,
};
use vize_s0::{String, cstr};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/contracts")
        .join(name)
}

fn protocol(pages: &[(&str, u32)], features: &[&str]) -> Protocol {
    Protocol {
        protocol_version: 1,
        pages: pages
            .iter()
            .map(|&(page, schema)| (String::from(page), schema))
            .collect(),
        required_features: BTreeMap::from([(
            String::from("dialect"),
            features
                .iter()
                .copied()
                .map(String::from)
                .collect::<BTreeSet<_>>(),
        )]),
    }
}

fn surface(name: &str, protocol: &Protocol) -> ContractSurface {
    surface_from_wit(&fixture(name), protocol).unwrap_or_else(|error| panic!("{name}: {error}"))
}

fn base() -> ContractSurface {
    surface("base", &protocol(&[("s1-page", 1)], &["s1-page@1"]))
}

fn change(kind: CompatibilityChangeKind, path: &str, message: &str) -> CompatibilityChange {
    CompatibilityChange {
        kind,
        path: String::from(path),
        message: String::from(message),
    }
}

fn messages(previous: &ContractSurface, next: &ContractSurface) -> Vec<String> {
    let report = compare_surfaces(previous, next);
    check_version_policy(previous, next, &report)
        .iter()
        .map(|violation| cstr!("{violation}"))
        .collect()
}

#[test]
fn the_base_surface_is_pinned() {
    let expected = std::fs::read(fixture("base.surface.json")).expect("the committed surface");
    let base = base();
    assert_eq!(
        std::str::from_utf8(&canonical_surface_json(&base)).expect("UTF-8"),
        std::str::from_utf8(&expected).expect("UTF-8")
    );
    assert_eq!(compare_surfaces(&base, &base).changes, []);
}

#[test]
fn a_purely_additive_change_is_not_flagged() {
    use CompatibilityChangeKind::Additive;
    let base = base();
    let next = surface(
        "additive",
        &protocol(&[("s1-page", 1), ("s2-page", 1)], &["s1-page@1"]),
    );
    let report = compare_surfaces(&base, &next);
    assert_eq!(
        report.changes,
        [
            change(Additive, "interfaces.clock", "an interface was added"),
            change(
                Additive,
                "interfaces.host-log.functions.flush",
                "the host provides more"
            ),
            change(Additive, "interfaces.types.types.bytes", "a type was added"),
            change(Additive, "pages.s2-page", "a page kind was added"),
            change(
                Additive,
                "worlds.dialect.imports.clock",
                "the host provides more"
            ),
            change(Additive, "worlds.probe", "a world was added"),
        ]
    );
    assert!(!report.is_breaking());
    assert_eq!(messages(&base, &next), Vec::<String>::new());
}

#[test]
fn a_deliberately_breaking_change_is_flagged_and_its_small_bump_refused() {
    use CompatibilityChangeKind::Breaking;
    let base = base();
    let next = surface(
        "breaking",
        &protocol(&[("s1-page", 2)], &["s1-page@1", "s2-page@1"]),
    );
    let report = compare_surfaces(&base, &next);
    let guests_lack = "guests built against the older version do not implement it";
    assert_eq!(
        report.changes,
        [
            change(Breaking, "interfaces.host-log", "removed"),
            change(
                Breaking,
                "interfaces.lowering.functions.lower",
                "the signature changed"
            ),
            change(Breaking, "interfaces.lowering.functions.reset", guests_lack),
            change(
                Breaking,
                "interfaces.lowering.types.answer",
                "the type's shape changed"
            ),
            change(
                Breaking,
                "interfaces.types.types.severity",
                "the type's shape changed"
            ),
            change(Breaking, "pages.s1-page", "the page schema version changed"),
            change(
                Breaking,
                "worlds.dialect.imports.host-log",
                "guests importing it no longer instantiate"
            ),
            change(
                Breaking,
                "worlds.dialect.requiredFeatures.s2-page@1",
                "guests built against the older version do not offer it"
            ),
        ]
    );
    assert!(report.is_breaking());
    assert_eq!(
        messages(&base, &next),
        [
            "a breaking change needs version 0.2.0 or later, found 0.1.1",
            "a breaking change must raise the protocol version by exactly one: 1 -> 1",
        ]
    );
    let bumped = ContractSurface {
        version: String::from("0.2.0"),
        protocol_version: 2,
        ..next
    };
    assert_eq!(messages(&base, &bumped), Vec::<String>::new());
}
