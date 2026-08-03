use std::panic::catch_unwind;

use crate::{
    ApplicationContract, Backend, BackendFamily, DiagnosticSeverity, Environment,
    EnvironmentConsumer, RuntimeFamily, Target,
};

use super::{CompatibilityChangeKind, CompatibilityReport, compare_contracts};

fn valid_contract() -> ApplicationContract {
    let mut contract = ApplicationContract::new("example");
    contract.targets.insert(Target::Web);
    contract.environments.push(Environment::new(
        "client",
        Target::Web,
        EnvironmentConsumer::Client,
        RuntimeFamily::Browser,
    ));
    contract
}

#[test]
fn duplicate_identifiers_fail_closed_without_comparing_overwritten_nodes() {
    let previous = valid_contract();
    let mut next = previous.clone();
    next.environments.push(Environment::new(
        "client",
        Target::Web,
        EnvironmentConsumer::Server,
        RuntimeFamily::Rust,
    ));

    let report = catch_unwind(|| compare_contracts(&previous, &next))
        .expect("invalid contracts must produce a report instead of panicking");

    assert!(report.changes.is_empty());
    assert!(report.has_invalid_inputs());
    assert!(report.is_breaking());
    assert!(report.previous_diagnostics.is_empty());
    assert_eq!(report.next_diagnostics.len(), 1);
    assert_eq!(report.next_diagnostics[0].code, "VIZE_MARQUETTE_006");
    assert_eq!(report.next_diagnostics[0].path, "environments.client");

    let json = serde_json::to_value(&report).expect("report must serialize");
    assert_eq!(json["nextDiagnostics"][0]["code"], "VIZE_MARQUETTE_006");
    assert_eq!(json["changes"], serde_json::json!([]));
    let round_trip: CompatibilityReport =
        serde_json::from_value(json).expect("report must deserialize");
    assert_eq!(round_trip, report);
}

#[test]
fn previous_and_next_validation_diagnostics_keep_their_provenance() {
    let mut previous = valid_contract();
    previous.routes.push(crate::Route::new(
        "home",
        "/",
        "missing",
        crate::RenderingMode::Client,
    ));
    let next = valid_contract();

    let report = compare_contracts(&previous, &next);

    assert!(report.changes.is_empty());
    assert!(report.has_invalid_inputs());
    assert!(report.is_breaking());
    assert!(
        report
            .previous_diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "VIZE_MARQUETTE_015")
    );
    assert!(report.next_diagnostics.is_empty());
}

#[test]
fn dangling_references_and_cycles_remain_deterministic_and_do_not_panic() {
    let previous = valid_contract();
    let mut next = valid_contract();
    next.environments[0].depends_on.insert("server".into());
    let mut server = Environment::new(
        "server",
        Target::Web,
        EnvironmentConsumer::Server,
        RuntimeFamily::Rust,
    );
    server.depends_on.insert("client".into());
    next.environments.push(server);
    next.backends
        .push(Backend::new("api", BackendFamily::Rust).with_environment("missing-environment"));

    let first = catch_unwind(|| compare_contracts(&previous, &next))
        .expect("cyclic and dangling input must not panic");
    let second = compare_contracts(&previous, &next);

    assert_eq!(first, second);
    assert!(first.changes.is_empty());
    assert!(first.next_diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "VIZE_MARQUETTE_011" && diagnostic.path == "backends.api"
    }));
    assert!(first.next_diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "VIZE_MARQUETTE_022" && diagnostic.path.starts_with("environments.")
    }));
    assert!(first.next_diagnostics.windows(2).all(|pair| {
        (&pair[0].path, &pair[0].code, &pair[0].message)
            <= (&pair[1].path, &pair[1].code, &pair[1].message)
    }));
}

#[test]
fn warnings_do_not_block_compatibility_classification() {
    let previous = ApplicationContract::new("example");
    let mut next = previous.clone();
    next.targets.insert(Target::Web);
    next.backends.push(Backend::new("api", BackendFamily::Rust));

    let report = compare_contracts(&previous, &next);

    assert!(!report.has_invalid_inputs());
    assert!(!report.is_breaking());
    assert!(report.previous_diagnostics.is_empty());
    assert_eq!(report.next_diagnostics.len(), 1);
    assert_eq!(
        report.next_diagnostics[0].severity,
        DiagnosticSeverity::Warning
    );
    assert_eq!(report.changes.len(), 1);
    assert_eq!(report.changes[0].kind, CompatibilityChangeKind::Additive);
    assert_eq!(report.changes[0].path, "backends.api");
}

#[test]
fn valid_contracts_report_changes_without_validation_diagnostics() {
    let previous = valid_contract();
    let mut next = previous.clone();
    next.environments[0].consumer = EnvironmentConsumer::Server;

    let report = compare_contracts(&previous, &next);

    assert!(!report.has_invalid_inputs());
    assert!(report.is_breaking());
    assert!(report.previous_diagnostics.is_empty());
    assert!(report.next_diagnostics.is_empty());
    assert_eq!(report.changes.len(), 1);
    assert_eq!(report.changes[0].kind, CompatibilityChangeKind::Breaking);
}
