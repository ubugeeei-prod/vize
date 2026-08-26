use std::collections::{BTreeMap, BTreeSet};

use vize_s0::{String, ToCompactString};

use crate::{ApplicationContract, RenderingMode, Target};

use super::ContractDiagnostic;

/// Validates and collects unique identifiers for one contract collection.
pub(super) fn collect_unique_ids<'a>(
    collection: &str,
    ids: impl Iterator<Item = &'a String>,
    diagnostics: &mut Vec<ContractDiagnostic>,
) -> BTreeSet<String> {
    let mut unique = BTreeSet::new();
    for id in ids {
        validate_identifier(
            id,
            &contract_path(collection, id),
            "VIZE_MARQUETTE_006",
            diagnostics,
        );
        if !unique.insert(id.clone()) {
            diagnostics.push(ContractDiagnostic::error(
                "VIZE_MARQUETTE_006",
                contract_path(collection, id),
                "identifier must be unique within its collection",
            ));
        }
    }
    unique
}

/// Validates the portable lowercase identifier grammar used by every adapter.
pub(crate) fn validate_identifier(
    id: &str,
    path: &str,
    code: &'static str,
    diagnostics: &mut Vec<ContractDiagnostic>,
) {
    let mut characters = id.bytes();
    let valid_first = characters
        .next()
        .is_some_and(|value| value.is_ascii_lowercase() || value.is_ascii_digit());
    let valid_rest = characters.all(|value| {
        value.is_ascii_lowercase() || value.is_ascii_digit() || matches!(value, b'-' | b'_' | b'.')
    });
    if !valid_first || !valid_rest {
        diagnostics.push(ContractDiagnostic::error(
            code,
            path,
            "identifier must use lowercase ASCII letters, digits, dash, underscore, or dot",
        ));
    }
}

/// Reports capability references that are absent from the root declaration.
pub(super) fn validate_capabilities(
    path: &str,
    required: &BTreeSet<String>,
    declared: &BTreeSet<String>,
    diagnostics: &mut Vec<ContractDiagnostic>,
) {
    for capability in required {
        if !declared.contains(capability) {
            diagnostics.push(ContractDiagnostic::error(
                "VIZE_MARQUETTE_021",
                path,
                "referenced capability is not declared",
            ));
        }
    }
}

/// Reports cycles in the environment dependency graph.
pub(super) fn validate_environment_cycles(
    contract: &ApplicationContract,
    diagnostics: &mut Vec<ContractDiagnostic>,
) {
    let graph = contract
        .environments
        .iter()
        .map(|environment| (&environment.id, &environment.depends_on))
        .collect::<BTreeMap<_, _>>();
    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();

    for id in graph.keys() {
        if has_cycle(id, &graph, &mut visiting, &mut visited) {
            diagnostics.push(ContractDiagnostic::error(
                "VIZE_MARQUETTE_022",
                contract_path("environments", id),
                "environment dependency graph must be acyclic",
            ));
        }
    }
}

/// Visits one dependency node and returns whether its path closes a cycle.
pub(super) fn has_cycle<'a>(
    id: &'a String,
    graph: &BTreeMap<&'a String, &'a BTreeSet<String>>,
    visiting: &mut BTreeSet<String>,
    visited: &mut BTreeSet<String>,
) -> bool {
    if visited.contains(id) {
        return false;
    }
    if !visiting.insert(id.clone()) {
        return true;
    }
    let cyclic = graph.get(id).is_some_and(|dependencies| {
        dependencies.iter().any(|dependency| {
            graph
                .get_key_value(dependency)
                .is_some_and(|(canonical, _)| has_cycle(canonical, graph, visiting, visited))
        })
    });
    visiting.remove(id);
    visited.insert(id.clone());
    cyclic
}

/// Ensures a route's rendering mode can execute on its target environment.
pub(super) fn validate_rendering_target(
    contract: &ApplicationContract,
    route: &crate::Route,
    path: &str,
    diagnostics: &mut Vec<ContractDiagnostic>,
) {
    let Some(environment) = contract
        .environments
        .iter()
        .find(|environment| environment.id == route.environment)
    else {
        return;
    };
    let valid = match route.rendering {
        RenderingMode::Native => environment.target == Target::Native,
        RenderingMode::Desktop => environment.target == Target::Desktop,
        RenderingMode::Terminal => environment.target == Target::Terminal,
        RenderingMode::Client
        | RenderingMode::Static
        | RenderingMode::Server
        | RenderingMode::Stream
        | RenderingMode::Partial
        | RenderingMode::Hybrid => environment.target == Target::Web,
    };
    if !valid {
        diagnostics.push(ContractDiagnostic::error(
            "VIZE_MARQUETTE_023",
            path,
            "rendering mode is not compatible with the route environment target",
        ));
    }
}

/// Builds the stable diagnostic path for a named collection member.
pub(crate) fn contract_path(collection: &str, id: &str) -> String {
    let mut path = collection.to_compact_string();
    path.push('.');
    path.push_str(id);
    path
}
