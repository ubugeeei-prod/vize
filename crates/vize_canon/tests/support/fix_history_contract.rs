//! Existing fixture schemas; visibility changes only permit the moved consumers.

use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {
    pub(crate) file: String,
    pub(crate) source: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Diagnostic {
    pub(crate) file: String,
    pub(crate) line: u32,
    pub(crate) column: u32,
    pub(crate) severity: u8,
    pub(crate) code: Option<u32>,
    pub(crate) message: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Case {
    pub(crate) id: String,
    pub(crate) inputs: Vec<Input>,
    pub(crate) diagnostics: Vec<Diagnostic>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Contract {
    #[serde(rename = "requiredTier")]
    pub(crate) required_tier: String,
    #[serde(rename = "positionBase")]
    pub(crate) position_base: u32,
    pub(crate) positions: String,
    pub(crate) comparison: String,
    #[serde(rename = "missingFields")]
    pub(crate) missing_fields: Vec<String>,
    pub(crate) native: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CheckerOptions {
    #[serde(default)]
    pub(crate) options_api: bool,
    #[serde(default)]
    pub(crate) jsx_typecheck: bool,
    #[serde(default)]
    pub(crate) legacy_vue2: bool,
    pub(crate) virtual_ts_checks: Option<VirtualTsChecks>,
    /// `VirtualTsOptions` the `vize check` Nuxt detection derives from a
    /// project's generated declaration graph.
    #[serde(default)]
    pub(crate) strict_instance_globals: bool,
    #[serde(default)]
    pub(crate) auto_import_stubs: Vec<String>,
    /// Package routes `vize check` installs, as `(importer, specifier)` pairs.
    #[serde(default)]
    pub(crate) package_routes: Vec<PackageRoute>,
    /// Workspace packages linked into `node_modules`, as `(package, link)`.
    #[serde(default)]
    pub(crate) workspace_links: Vec<WorkspaceLink>,
    /// Explicit authored check roots, matching the CLI's owning input scope.
    #[serde(default)]
    pub(crate) scan_paths: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct WorkspaceLink {
    pub(crate) package: String,
    pub(crate) link: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct PackageRoute {
    pub(crate) importer: String,
    pub(crate) specifier: String,
}

/// `BatchTypeChecker::set_virtual_ts_checks` as the original test called it.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct VirtualTsChecks {
    pub(crate) props: bool,
    pub(crate) template_bindings: bool,
    pub(crate) emits: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Pack {
    pub(crate) version: u32,
    pub(crate) issue: u32,
    pub(crate) regression_commit: String,
    pub(crate) historical_issue: Option<u32>,
    pub(crate) historical_issues: Option<Vec<u32>>,
    pub(crate) source_revision: String,
    pub(crate) diagnostic_contract: Contract,
    pub(crate) checker_options: Option<CheckerOptions>,
    pub(crate) project_options: Option<serde_json::Value>,
    pub(crate) cases: Vec<Case>,
}
