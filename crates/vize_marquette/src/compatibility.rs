use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use vize_carton::{String, ToCompactString};

use crate::ApplicationContract;

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

/// Deterministic compatibility report between two contracts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompatibilityReport {
    /// All changes in stable path order.
    pub changes: Vec<CompatibilityChange>,
}

impl CompatibilityReport {
    /// Returns `true` when any existing client or adapter may be incompatible.
    pub fn is_breaking(&self) -> bool {
        self.changes
            .iter()
            .any(|change| change.kind == CompatibilityChangeKind::Breaking)
    }
}

/// Compares two contracts from older to newer.
pub fn compare_contracts(
    previous: &ApplicationContract,
    next: &ApplicationContract,
) -> CompatibilityReport {
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
    CompatibilityReport { changes }
}

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

fn breaking(collection: &str, id: &str, message: &str) -> CompatibilityChange {
    change(CompatibilityChangeKind::Breaking, collection, id, message)
}

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
