use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use vize_s0::{String, cstr};

use crate::{ApplicationContract, CONTRACT_FORMAT_VERSION, EnvironmentConsumer, RuntimeFamily};

pub(crate) mod rules;
#[cfg(test)]
mod tests;

use rules::{
    collect_unique_ids, contract_path, validate_capabilities, validate_environment_cycles,
    validate_identifier, validate_rendering_target,
};

/// Severity of an application-marquette diagnostic.
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
    /// Creates an error diagnostic that prevents adapter execution.
    pub(crate) fn error(
        code: &'static str,
        path: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            severity: DiagnosticSeverity::Error,
            path: path.into(),
            message: message.into(),
        }
    }

    /// Creates a warning diagnostic for a valid but suspicious contract.
    pub(crate) fn warning(
        code: &'static str,
        path: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
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
            "VIZE_MARQUETTE_001",
            "formatVersion",
            "unsupported marquette format version",
        ));
    }

    validate_identifier(
        &contract.application,
        "application",
        "VIZE_MARQUETTE_002",
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
            "VIZE_MARQUETTE_003",
            &mut diagnostics,
        );
        if key.as_str() != capability.id.as_str() {
            diagnostics.push(ContractDiagnostic::error(
                "VIZE_MARQUETTE_004",
                contract_path("capabilities", key),
                "capability map key must equal capability.id",
            ));
        }
        if capability.version == 0 {
            diagnostics.push(ContractDiagnostic::error(
                "VIZE_MARQUETTE_005",
                contract_path("capabilities", key),
                "capability version must be greater than zero",
            ));
        }
        if capability.description.trim().is_empty() {
            diagnostics.push(ContractDiagnostic::error(
                "VIZE_MARQUETTE_024",
                contract_path("capabilities", key),
                "capability description must not be empty",
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
                "VIZE_MARQUETTE_007",
                path.clone(),
                "environment target must be declared in targets",
            ));
        }
        for dependency in &environment.depends_on {
            if dependency.as_str() == environment.id.as_str() {
                diagnostics.push(ContractDiagnostic::error(
                    "VIZE_MARQUETTE_008",
                    path.clone(),
                    "environment cannot depend on itself",
                ));
            } else if !environment_ids.contains(dependency) {
                diagnostics.push(ContractDiagnostic::error(
                    "VIZE_MARQUETTE_009",
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
                "VIZE_MARQUETTE_010",
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
                    "VIZE_MARQUETTE_011",
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
                    "VIZE_MARQUETTE_012",
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
                "VIZE_MARQUETTE_013",
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
                "VIZE_MARQUETTE_014",
                path.clone(),
                "route path must start with /",
            ));
        }
        if !environment_ids.contains(&route.environment) {
            diagnostics.push(ContractDiagnostic::error(
                "VIZE_MARQUETTE_015",
                path.clone(),
                "route environment does not exist",
            ));
        }
        if let Some(backend) = &route.backend
            && !backend_ids.contains(backend)
        {
            diagnostics.push(ContractDiagnostic::error(
                "VIZE_MARQUETTE_016",
                path.clone(),
                "route backend does not exist",
            ));
        }
        if let Some(protocol) = &route.protocol {
            if !protocol_ids.contains(protocol) {
                diagnostics.push(ContractDiagnostic::error(
                    "VIZE_MARQUETTE_017",
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
                    "VIZE_MARQUETTE_018",
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
                "VIZE_MARQUETTE_019",
                path,
                cstr!("route path is already used by route \"{previous}\""),
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
                "VIZE_MARQUETTE_020",
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
