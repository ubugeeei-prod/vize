use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use vize_carton::{String, ToCompactString};

use crate::{
    ApplicationContract, CONTRACT_FORMAT_VERSION, EnvironmentConsumer, RenderingMode,
    RuntimeFamily, Target,
};

/// Severity of a platform-contract diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DiagnosticSeverity {
    /// Contract execution must stop.
    Error,
    /// Contract is valid but contains a likely configuration mistake.
    Warning,
}

/// Stable, source-addressable application-contract diagnostic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContractDiagnostic {
    /// Stable machine-readable diagnostic code.
    pub code: &'static str,
    /// Severity used by CLI, editor, and CI consumers.
    pub severity: DiagnosticSeverity,
    /// JSON-style path into the authored contract.
    pub path: String,
    /// Human-readable explanation and next action.
    pub message: String,
}

impl ContractDiagnostic {
    fn error(code: &'static str, path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code,
            severity: DiagnosticSeverity::Error,
            path: path.into(),
            message: message.into(),
        }
    }

    fn warning(code: &'static str, path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code,
            severity: DiagnosticSeverity::Warning,
            path: path.into(),
            message: message.into(),
        }
    }
}

/// Validates a complete application contract.
///
/// Diagnostics are deterministic and sorted by path, code, and message so the
/// same contract produces stable editor, CLI, test, and CI output.
pub fn validate_contract(contract: &ApplicationContract) -> Vec<ContractDiagnostic> {
    let mut diagnostics = Vec::new();

    if contract.format_version != CONTRACT_FORMAT_VERSION {
        diagnostics.push(ContractDiagnostic::error(
            "VIZE_PLATFORM_001",
            "formatVersion",
            "unsupported platform contract format version",
        ));
    }

    validate_identifier(
        &contract.application,
        "application",
        "VIZE_PLATFORM_002",
        &mut diagnostics,
    );

    let capability_ids = contract
        .capabilities
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();
    for (key, capability) in &contract.capabilities {
        validate_identifier(
            key,
            &contract_path("capabilities", key),
            "VIZE_PLATFORM_003",
            &mut diagnostics,
        );
        if key.as_str() != capability.id.as_str() {
            diagnostics.push(ContractDiagnostic::error(
                "VIZE_PLATFORM_004",
                contract_path("capabilities", key),
                "capability map key must equal capability.id",
            ));
        }
        if capability.version == 0 {
            diagnostics.push(ContractDiagnostic::error(
                "VIZE_PLATFORM_005",
                contract_path("capabilities", key),
                "capability version must be greater than zero",
            ));
        }
    }

    let environment_ids = collect_unique_ids(
        "environments",
        contract.environments.iter().map(|value| &value.id),
        &mut diagnostics,
    );
    let backend_ids = collect_unique_ids(
        "backends",
        contract.backends.iter().map(|value| &value.id),
        &mut diagnostics,
    );
    let protocol_ids = collect_unique_ids(
        "protocols",
        contract.protocols.iter().map(|value| &value.id),
        &mut diagnostics,
    );
    collect_unique_ids(
        "routes",
        contract.routes.iter().map(|value| &value.id),
        &mut diagnostics,
    );

    for environment in &contract.environments {
        let path = contract_path("environments", &environment.id);
        if !contract.targets.contains(&environment.target) {
            diagnostics.push(ContractDiagnostic::error(
                "VIZE_PLATFORM_007",
                path.clone(),
                "environment target must be declared in targets",
            ));
        }
        for dependency in &environment.depends_on {
            if dependency.as_str() == environment.id.as_str() {
                diagnostics.push(ContractDiagnostic::error(
                    "VIZE_PLATFORM_008",
                    path.clone(),
                    "environment cannot depend on itself",
                ));
            } else if !environment_ids.contains(dependency) {
                diagnostics.push(ContractDiagnostic::error(
                    "VIZE_PLATFORM_009",
                    path.clone(),
                    "environment dependency does not exist",
                ));
            }
        }
        validate_capabilities(
            &path,
            &environment.capabilities,
            &capability_ids,
            &mut diagnostics,
        );

        if environment.consumer == EnvironmentConsumer::Client
            && matches!(
                environment.runtime,
                RuntimeFamily::Rust | RuntimeFamily::Go | RuntimeFamily::Jvm
            )
        {
            diagnostics.push(ContractDiagnostic::warning(
                "VIZE_PLATFORM_010",
                path,
                "client environment uses a server-oriented runtime; declare an adapter capability if this is intentional",
            ));
        }
    }

    validate_environment_cycles(contract, &mut diagnostics);

    for backend in &contract.backends {
        let path = contract_path("backends", &backend.id);
        if let Some(environment) = &backend.environment {
            if !environment_ids.contains(environment) {
                diagnostics.push(ContractDiagnostic::error(
                    "VIZE_PLATFORM_011",
                    path.clone(),
                    "backend environment does not exist",
                ));
            } else if contract
                .environments
                .iter()
                .find(|candidate| candidate.id.as_str() == environment.as_str())
                .is_some_and(|candidate| candidate.consumer != EnvironmentConsumer::Server)
            {
                diagnostics.push(ContractDiagnostic::error(
                    "VIZE_PLATFORM_012",
                    path.clone(),
                    "backend environment must be a server consumer",
                ));
            }
        }
        validate_capabilities(
            &path,
            &backend.capabilities,
            &capability_ids,
            &mut diagnostics,
        );
    }

    for protocol in &contract.protocols {
        let path = contract_path("protocols", &protocol.id);
        if !backend_ids.contains(&protocol.backend) {
            diagnostics.push(ContractDiagnostic::error(
                "VIZE_PLATFORM_013",
                path.clone(),
                "protocol backend does not exist",
            ));
        }
        validate_capabilities(
            &path,
            &protocol.capabilities,
            &capability_ids,
            &mut diagnostics,
        );
    }

    let mut route_paths = BTreeMap::<(&String, &String), &String>::new();
    for route in &contract.routes {
        let path = contract_path("routes", &route.id);
        if !route.path.starts_with('/') {
            diagnostics.push(ContractDiagnostic::error(
                "VIZE_PLATFORM_014",
                path.clone(),
                "route path must start with /",
            ));
        }
        if !environment_ids.contains(&route.environment) {
            diagnostics.push(ContractDiagnostic::error(
                "VIZE_PLATFORM_015",
                path.clone(),
                "route environment does not exist",
            ));
        }
        if let Some(backend) = &route.backend
            && !backend_ids.contains(backend)
        {
            diagnostics.push(ContractDiagnostic::error(
                "VIZE_PLATFORM_016",
                path.clone(),
                "route backend does not exist",
            ));
        }
        if let Some(protocol) = &route.protocol {
            if !protocol_ids.contains(protocol) {
                diagnostics.push(ContractDiagnostic::error(
                    "VIZE_PLATFORM_017",
                    path.clone(),
                    "route protocol does not exist",
                ));
            } else if let Some(route_backend) = &route.backend
                && contract
                    .protocols
                    .iter()
                    .find(|candidate| candidate.id.as_str() == protocol.as_str())
                    .is_some_and(|candidate| candidate.backend.as_str() != route_backend.as_str())
            {
                diagnostics.push(ContractDiagnostic::error(
                    "VIZE_PLATFORM_018",
                    path.clone(),
                    "route protocol and backend must refer to the same service",
                ));
            }
        }
        validate_rendering_target(contract, route, &path, &mut diagnostics);
        validate_capabilities(
            &path,
            &route.capabilities,
            &capability_ids,
            &mut diagnostics,
        );

        let key = (&route.environment, &route.path);
        if let Some(previous) = route_paths.insert(key, &route.id) {
            diagnostics.push(ContractDiagnostic::error(
                "VIZE_PLATFORM_019",
                path,
                previous.clone(),
            ));
        }
    }

    for target in &contract.targets {
        if !contract
            .environments
            .iter()
            .any(|environment| &environment.target == target)
        {
            diagnostics.push(ContractDiagnostic::warning(
                "VIZE_PLATFORM_020",
                "targets",
                "declared target has no environment",
            ));
        }
    }

    diagnostics.sort_by(|left, right| {
        (&left.path, left.code, &left.message).cmp(&(&right.path, right.code, &right.message))
    });
    diagnostics
}

fn collect_unique_ids<'a>(
    collection: &str,
    ids: impl Iterator<Item = &'a String>,
    diagnostics: &mut Vec<ContractDiagnostic>,
) -> BTreeSet<String> {
    let mut unique = BTreeSet::new();
    for id in ids {
        validate_identifier(
            id,
            &contract_path(collection, id),
            "VIZE_PLATFORM_006",
            diagnostics,
        );
        if !unique.insert(id.clone()) {
            diagnostics.push(ContractDiagnostic::error(
                "VIZE_PLATFORM_006",
                contract_path(collection, id),
                "identifier must be unique within its collection",
            ));
        }
    }
    unique
}

fn validate_identifier(
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

fn validate_capabilities(
    path: &str,
    required: &BTreeSet<String>,
    declared: &BTreeSet<String>,
    diagnostics: &mut Vec<ContractDiagnostic>,
) {
    for capability in required {
        if !declared.contains(capability) {
            diagnostics.push(ContractDiagnostic::error(
                "VIZE_PLATFORM_021",
                path,
                "referenced capability is not declared",
            ));
        }
    }
}

fn validate_environment_cycles(
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
                "VIZE_PLATFORM_022",
                contract_path("environments", id),
                "environment dependency graph must be acyclic",
            ));
        }
    }
}

fn has_cycle<'a>(
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

fn validate_rendering_target(
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
            "VIZE_PLATFORM_023",
            path,
            "rendering mode is not compatible with the route environment target",
        ));
    }
}

fn contract_path(collection: &str, id: &str) -> String {
    let mut path = collection.to_compact_string();
    path.push('.');
    path.push_str(id);
    path
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Backend, BackendFamily, Environment, Protocol, ProtocolFamily, RenderingMode, Route,
        RuntimeFamily,
    };

    #[test]
    fn accepts_a_cross_language_web_contract() {
        let mut contract = ApplicationContract::new("shop");
        contract.targets.insert(Target::Web);
        contract.environments.push(Environment::new(
            "client",
            Target::Web,
            EnvironmentConsumer::Client,
            RuntimeFamily::Browser,
        ));
        contract.environments.push(Environment::new(
            "server",
            Target::Web,
            EnvironmentConsumer::Server,
            RuntimeFamily::Rust,
        ));
        contract
            .backends
            .push(Backend::new("api", BackendFamily::Rust).with_environment("server"));
        contract.protocols.push(Protocol::new(
            "api.query",
            ProtocolFamily::SchemaQuery,
            "api",
        ));
        let mut route = Route::new("home", "/", "client", RenderingMode::Hybrid);
        route.backend = Some("api".into());
        route.protocol = Some("api.query".into());
        contract.routes.push(route);

        assert_eq!(validate_contract(&contract), Vec::new());
    }

    #[test]
    fn reports_cycles_and_cross_reference_errors() {
        let mut contract = ApplicationContract::new("Broken App");
        contract.targets.insert(Target::Web);
        let mut first = Environment::new(
            "first",
            Target::Web,
            EnvironmentConsumer::Server,
            RuntimeFamily::JavaScript,
        );
        first.depends_on.insert("second".into());
        let mut second = Environment::new(
            "second",
            Target::Web,
            EnvironmentConsumer::Server,
            RuntimeFamily::JavaScript,
        );
        second.depends_on.insert("first".into());
        contract.environments.extend([first, second]);
        contract.routes.push(Route::new(
            "home",
            "missing-slash",
            "missing",
            RenderingMode::Native,
        ));

        let codes = validate_contract(&contract)
            .into_iter()
            .map(|diagnostic| diagnostic.code)
            .collect::<BTreeSet<_>>();
        assert!(codes.contains("VIZE_PLATFORM_002"));
        assert!(codes.contains("VIZE_PLATFORM_014"));
        assert!(codes.contains("VIZE_PLATFORM_015"));
        assert!(codes.contains("VIZE_PLATFORM_022"));
    }
}
