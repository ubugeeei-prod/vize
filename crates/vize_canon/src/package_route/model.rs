//! Public package-route identities and importer-scoped resolution context.

use std::path::{Path, PathBuf};

use vize_carton::String;

/// TypeScript resolution inputs that can change which conditional target wins.
///
/// Vize records these in the cache identity, but deliberately does not interpret
/// their priority. The unchanged package manifest is materialized for TypeScript
/// to apply its own resolution algorithm.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct PackageResolutionContext {
    pub module_resolution: Option<String>,
    pub mode: PackageResolutionMode,
    pub active_conditions: Vec<String>,
    pub scope_manifest_path: Option<PathBuf>,
}

impl PackageResolutionContext {
    pub fn new(
        module_resolution: Option<impl Into<String>>,
        mode: PackageResolutionMode,
        active_conditions: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        let mut active_conditions = active_conditions
            .into_iter()
            .map(Into::into)
            .collect::<Vec<_>>();
        active_conditions.sort();
        active_conditions.dedup();
        Self {
            module_resolution: module_resolution
                .map(Into::into)
                .map(|value: String| value.to_ascii_lowercase()),
            mode,
            active_conditions,
            scope_manifest_path: None,
        }
    }

    /// Derive the effective TypeScript resolution identity for one authored
    /// occurrence. CLI and editor callers share this implementation so `.cts`,
    /// CommonJS, package `type`, and custom-condition cache keys cannot drift.
    pub fn for_importer(
        importer: &Path,
        occurrence_mode: PackageResolutionMode,
        module_resolution: Option<&str>,
        module: Option<&str>,
        active_conditions: impl IntoIterator<Item = impl Into<String>>,
        nearest_package_is_module: Option<bool>,
    ) -> Self {
        let inferred_module_resolution = module.and_then(|module| {
            if matches_ignore_ascii_case(module, &["node16", "node18", "node20", "nodenext"]) {
                Some(module)
            } else if module.eq_ignore_ascii_case("preserve") {
                Some("bundler")
            } else {
                None
            }
        });
        let module_resolution = module_resolution.or(inferred_module_resolution);
        let mode = match occurrence_mode {
            PackageResolutionMode::Import | PackageResolutionMode::Require => occurrence_mode,
            PackageResolutionMode::Contextual => {
                match importer
                    .extension()
                    .and_then(|extension| extension.to_str())
                {
                    Some("cts" | "cjs") => PackageResolutionMode::Require,
                    Some("mts" | "mjs") => PackageResolutionMode::Import,
                    _ if module_resolution
                        .is_some_and(|resolution| resolution.eq_ignore_ascii_case("bundler")) =>
                    {
                        PackageResolutionMode::Import
                    }
                    _ if module.is_some_and(|module| module.eq_ignore_ascii_case("commonjs")) => {
                        PackageResolutionMode::Require
                    }
                    _ if module_resolution.is_some_and(|resolution| {
                        matches_ignore_ascii_case(
                            resolution,
                            &["node16", "node18", "node20", "nodenext"],
                        ) && nearest_package_is_module != Some(true)
                    }) =>
                    {
                        PackageResolutionMode::Require
                    }
                    _ => PackageResolutionMode::Import,
                }
            }
        };
        Self::new(module_resolution, mode, active_conditions)
    }
}

fn matches_ignore_ascii_case(value: &str, expected: &[&str]) -> bool {
    expected
        .iter()
        .any(|expected| value.eq_ignore_ascii_case(expected))
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PackageResolutionMode {
    /// Static imports/exports and import types whose branch follows the
    /// importing file's module format unless an explicit attribute overrides.
    Contextual,
    #[default]
    Import,
    Require,
}

impl PackageResolutionMode {
    /// Interpret TypeScript's explicit per-occurrence `resolution-mode` value.
    /// Syntax-specific collectors share this value contract; unknown values
    /// remain native parser diagnostics and never silently select a branch.
    pub fn from_explicit_attribute(value: &str) -> Option<Self> {
        match value {
            "import" => Some(Self::Import),
            "require" => Some(Self::Require),
            _ => None,
        }
    }
}

/// One physical package reached through an importer-local package spelling.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageRoute {
    /// Every existing source target reachable through the matching manifest
    /// entry. TypeScript selects one after Vize materializes the raw manifest.
    pub source_paths: Vec<PathBuf>,
    /// Package-local script/declaration/Vue files reached from manifest
    /// candidates. These are materialization inputs, never candidate winners.
    pub dependency_paths: Vec<PathBuf>,
    /// Manifest target -> source identities that must exist in the shadow.
    /// This is candidate topology only; ordering never selects a winner.
    pub source_targets: Vec<PackageRouteSource>,
    pub package_root: PathBuf,
    pub package_link_root: PathBuf,
    pub manifest_path: PathBuf,
    pub package_name: Option<String>,
    pub workspace_source: bool,
    /// Bare-package targets reached through this package's private `imports`
    /// map. Keeping the graph nested lets the raw parent manifest select the
    /// branch while every external package gets its own native shadow scope.
    pub nested_routes: Vec<PackageRoute>,
}

impl PackageRoute {
    pub fn invalidation_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        self.collect_invalidation_paths(&mut paths);
        paths.sort();
        paths.dedup();
        paths
    }

    /// Gather without ordering; the public entry sorts and dedups once.
    ///
    /// Sorting per level instead made the cost quadratic in the nesting depth:
    /// every level sorted its own subtree and the parent then re-sorted the
    /// union it had just built, with `Path`'s component-wise comparison doing
    /// the work. A `workspace:*` chain nests one route per package, so a 130
    /// package chain paid 130 sorts of a list that grew at every level (#4426).
    fn collect_invalidation_paths(&self, out: &mut Vec<PathBuf>) {
        out.extend(self.source_paths.iter().cloned());
        out.extend(self.dependency_paths.iter().cloned());
        out.extend([
            self.manifest_path.clone(),
            self.package_link_root.clone(),
            self.package_link_root.join("package.json"),
        ]);
        for route in &self.nested_routes {
            route.collect_invalidation_paths(out);
        }
    }

    /// Return a target only when the manifest topology has one source identity.
    /// Callers that cannot preserve importer context must fail closed instead of
    /// silently selecting a conditional branch.
    pub fn unambiguous_source_path(&self) -> Option<&PathBuf> {
        let mut paths = self.source_paths.iter().collect::<Vec<_>>();
        for route in &self.nested_routes {
            paths.extend(route.candidate_source_paths());
        }
        paths.sort();
        paths.dedup();
        (paths.len() == 1).then(|| paths.first().copied()).flatten()
    }

    pub fn all_source_paths(&self) -> Vec<&PathBuf> {
        let mut paths = Vec::new();
        self.collect_all_source_paths(&mut paths);
        paths.sort();
        paths.dedup();
        paths
    }

    pub fn requires_workspace_source_shadow(&self) -> bool {
        (self.workspace_source
            && self.source_targets.iter().any(|source| {
                source.source_path != source.native_probe_path
                    && source.source_path.starts_with(&self.package_root)
                    && source.native_probe_path.starts_with(&self.package_root)
            }))
            || self
                .nested_routes
                .iter()
                .any(PackageRoute::requires_workspace_source_shadow)
    }

    /// See [`Self::collect_invalidation_paths`] for why the recursion does not
    /// sort as it descends.
    fn collect_all_source_paths<'a>(&'a self, out: &mut Vec<&'a PathBuf>) {
        out.extend(self.source_paths.iter());
        out.extend(self.dependency_paths.iter());
        for route in &self.nested_routes {
            route.collect_all_source_paths(out);
        }
    }

    fn candidate_source_paths(&self) -> Vec<&PathBuf> {
        let mut paths = self.source_paths.iter().collect::<Vec<_>>();
        for route in &self.nested_routes {
            paths.extend(route.candidate_source_paths());
        }
        paths
    }

    #[cfg(feature = "native")]
    pub(crate) fn all_routes(&self) -> Vec<&PackageRoute> {
        let mut routes = vec![self];
        for route in &self.nested_routes {
            routes.extend(route.all_routes());
        }
        routes
    }

    /// Map a native TypeScript definition inside a materialized package shadow
    /// back to the authored source candidate it selected. The native location
    /// is the authority; this method only reverses Canon's deterministic shadow
    /// spelling and never chooses between manifest conditions itself.
    pub fn source_for_native_shadow_path(
        &self,
        shadow_root: &Path,
        shadow_path: &Path,
    ) -> Option<&PathBuf> {
        super::shadow_mapping::source_for_native_shadow_path(self, shadow_root, shadow_path)
    }
}

/// One native manifest target and a source file Vize can materialize for it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageRouteSource {
    pub target_path: PathBuf,
    pub source_path: PathBuf,
    pub native_probe_path: PathBuf,
}

/// Importer-local route retained by Canon until the virtual package topology is
/// materialized. Keeping the authored specifier here prevents a global exact
/// specifier map from collapsing duplicate installed package versions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageRouteBinding {
    pub importer_path: PathBuf,
    pub specifier: String,
    pub occurrence_mode: PackageResolutionMode,
    pub context: PackageResolutionContext,
    /// `None` retains a cold negative lookup. The consulted filesystem inputs
    /// remain reverse-indexed so creating a package link or manifest activates
    /// this same importer/specifier identity without restarting the checker.
    pub route: Option<PackageRoute>,
    /// Positive and negative filesystem inputs consulted for this binding.
    /// Incremental refresh indexes these paths instead of scanning all routes.
    pub invalidation_paths: Vec<PathBuf>,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg(feature = "native")]
pub(crate) struct PackageRouteKey {
    pub importer_path: PathBuf,
    pub specifier: String,
    pub occurrence_mode: PackageResolutionMode,
}

impl PackageRouteBinding {
    #[cfg(feature = "native")]
    pub(crate) fn key(&self) -> PackageRouteKey {
        PackageRouteKey {
            importer_path: self.importer_path.clone(),
            specifier: self.specifier.clone(),
            occurrence_mode: self.occurrence_mode,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PackageRouteMetrics {
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub manifest_reads: u64,
    pub refresh_considered_routes: u64,
    pub refresh_affected_routes: u64,
    pub last_refresh_considered_routes: u64,
    pub last_refresh_affected_routes: u64,
    pub last_refresh_total_routes: u64,
    pub resolution_cache_entries: u64,
    pub manifest_cache_entries: u64,
    pub resolution_cache_evictions: u64,
    pub manifest_cache_evictions: u64,
    pub reachability_checks: u64,
    pub reachability_budget_exceeded: u64,
    pub last_reachability_files: u64,
    pub last_reachability_bytes: u64,
    pub last_reachability_edges: u64,
    pub last_reachability_parses: u64,
    pub last_reachability_packages: u64,
}
