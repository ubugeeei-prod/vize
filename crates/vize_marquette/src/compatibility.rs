use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use vize_s0::{String, ToCompactString};

use crate::{ApplicationContract, ContractDiagnostic, DiagnosticSeverity, validate_contract};

/// Compatibility classification for one marquette graph change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CompatibilityChangeKind {
    /// Existing generated clients or adapters can continue unchanged.
    Additive,
    /// Existing generated clients or adapters may fail or change behavior.
    Breaking,
}

/// One stable compatibility change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompatibilityChange {
    /// Compatibility classification.
    pub kind: CompatibilityChangeKind,
    /// JSON-style path of the changed contract node.
    pub path: String,
    /// Human-readable summary.
    pub message: String,
}

/// Owned validation diagnostic retained with a compatibility report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompatibilityInputDiagnostic {
    /// Stable machine-readable diagnostic code.
    pub code: String,
    /// Validation severity used to decide whether comparison is safe.
    pub severity: DiagnosticSeverity,
    /// JSON-style path into the authored contract.
    pub path: String,
    /// Human-readable explanation and next action.
    pub message: String,
}

impl From<ContractDiagnostic> for CompatibilityInputDiagnostic {
    fn from(diagnostic: ContractDiagnostic) -> Self {
        Self {
            code: diagnostic.code.into(),
            severity: diagnostic.severity,
            path: diagnostic.path,
            message: diagnostic.message,
        }
    }
}

/// Deterministic compatibility report between two contracts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompatibilityReport {
    /// All changes in stable path order.
    pub changes: Vec<CompatibilityChange>,
    /// Validation diagnostics from the older contract.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub previous_diagnostics: Vec<CompatibilityInputDiagnostic>,
    /// Validation diagnostics from the newer contract.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub next_diagnostics: Vec<CompatibilityInputDiagnostic>,
}

impl CompatibilityReport {
    /// Returns `true` when an invalid input or breaking change prevents compatibility.
    pub fn is_breaking(&self) -> bool {
        self.has_invalid_inputs()
            || self
                .changes
                .iter()
                .any(|change| change.kind == CompatibilityChangeKind::Breaking)
    }

    /// Returns `true` when either contract contains a validation error.
    pub fn has_invalid_inputs(&self) -> bool {
        self.previous_diagnostics
            .iter()
            .chain(&self.next_diagnostics)
            .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
    }
}

/// Compares two contracts from older to newer.
pub fn compare_contracts(
    previous: &ApplicationContract,
    next: &ApplicationContract,
) -> CompatibilityReport {
    let previous_diagnostics = validate_contract(previous)
        .into_iter()
        .map(CompatibilityInputDiagnostic::from)
        .collect::<Vec<_>>();
    let next_diagnostics = validate_contract(next)
        .into_iter()
        .map(CompatibilityInputDiagnostic::from)
        .collect::<Vec<_>>();
    let mut report = CompatibilityReport {
        changes: Vec::new(),
        previous_diagnostics,
        next_diagnostics,
    };
    if report.has_invalid_inputs() {
        return report;
    }

    let mut changes = Vec::new();

    compare_by_id(
        "environments",
        &previous.environments,
        &next.environments,
        |environment| &environment.id,
        &mut changes,
    );
    compare_by_id(
        "backends",
        &previous.backends,
        &next.backends,
        |backend| &backend.id,
        &mut changes,
    );
    compare_by_id(
        "protocols",
        &previous.protocols,
        &next.protocols,
        |protocol| &protocol.id,
        &mut changes,
    );
    compare_by_id(
        "routes",
        &previous.routes,
        &next.routes,
        |route| &route.id,
        &mut changes,
    );

    for old in &previous.environments {
        if let Some(new) = next
            .environments
            .iter()
            .find(|candidate| candidate.id == old.id)
            && (old.target, old.consumer, old.runtime) != (new.target, new.consumer, new.runtime)
        {
            changes.push(breaking(
                "environments",
                &old.id,
                "execution identity changed",
            ));
        }
    }
    for old in &previous.backends {
        if let Some(new) = next
            .backends
            .iter()
            .find(|candidate| candidate.id == old.id)
            && old.family != new.family
        {
            changes.push(breaking("backends", &old.id, "backend family changed"));
        }
    }
    for old in &previous.protocols {
        if let Some(new) = next
            .protocols
            .iter()
            .find(|candidate| candidate.id == old.id)
            && (old.family, &old.backend) != (new.family, &new.backend)
        {
            changes.push(breaking("protocols", &old.id, "protocol identity changed"));
        }
    }
    for old in &previous.routes {
        if let Some(new) = next.routes.iter().find(|candidate| candidate.id == old.id)
            && (
                &old.path,
                &old.environment,
                old.rendering,
                &old.backend,
                &old.protocol,
            ) != (
                &new.path,
                &new.environment,
                new.rendering,
                &new.backend,
                &new.protocol,
            )
        {
            changes.push(breaking(
                "routes",
                &old.id,
                "route execution contract changed",
            ));
        }
    }

    changes.sort_by(|left, right| (&left.path, left.kind).cmp(&(&right.path, right.kind)));
    report.changes = changes;
    report
}

/// Compares a named collection by stable identifier.
fn compare_by_id<T: PartialEq>(
    collection: &str,
    previous: &[T],
    next: &[T],
    id: impl Fn(&T) -> &String,
    changes: &mut Vec<CompatibilityChange>,
) {
    let old = previous
        .iter()
        .map(|value| (id(value), value))
        .collect::<BTreeMap<_, _>>();
    let new = next
        .iter()
        .map(|value| (id(value), value))
        .collect::<BTreeMap<_, _>>();
    for removed in old.keys().filter(|candidate| !new.contains_key(*candidate)) {
        changes.push(breaking(collection, removed, "contract node was removed"));
    }
    for added in new.keys().filter(|candidate| !old.contains_key(*candidate)) {
        changes.push(change(
            CompatibilityChangeKind::Additive,
            collection,
            added,
            "contract node was added",
        ));
    }
}

/// Creates a breaking compatibility change for one collection member.
fn breaking(collection: &str, id: &str, message: &str) -> CompatibilityChange {
    change(CompatibilityChangeKind::Breaking, collection, id, message)
}

/// Creates a compatibility change with a deterministic graph path.
fn change(
    kind: CompatibilityChangeKind,
    collection: &str,
    id: &str,
    message: &str,
) -> CompatibilityChange {
    let mut path = collection.to_compact_string();
    path.push('.');
    path.push_str(id);
    CompatibilityChange {
        kind,
        path,
        message: message.into(),
    }
}

#[cfg(test)]
mod fail_closed_tests;

#[cfg(test)]
mod tests {
    use crate::{ApplicationContract, Environment, EnvironmentConsumer, RuntimeFamily, Target};

    use super::*;

    #[test]
    fn classifies_additions_and_execution_changes() {
        let mut previous = ApplicationContract::new("example");
        previous.targets.insert(Target::Web);
        previous.environments.push(Environment::new(
            "client",
            Target::Web,
            EnvironmentConsumer::Client,
            RuntimeFamily::Browser,
        ));
        let mut next = previous.clone();
        next.environments[0].consumer = EnvironmentConsumer::Server;
        next.environments.push(Environment::new(
            "server",
            Target::Web,
            EnvironmentConsumer::Server,
            RuntimeFamily::Rust,
        ));

        let report = compare_contracts(&previous, &next);
        assert!(report.is_breaking());
        assert!(report.changes.iter().any(|change| {
            change.kind == CompatibilityChangeKind::Additive && change.path == "environments.server"
        }));
    }
}
