//! Deterministically bounded Vue reachability for importer-scoped packages.

#![expect(
    clippy::disallowed_types,
    reason = "one sorted input set is shared by every importer of a route"
)]

use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use vize_carton::{FxHashSet, String as CompactString};

use super::dependency_scan::resolve_dependency_with_inputs;
use super::is_vue_runtime_support_specifier;

/// Cache and evidence identity for the fixed reachability work contract.
pub const PACKAGE_REACHABILITY_BUDGET_REVISION: u8 = 2;

// One route may inspect at most 128 package identities, source candidates, or
// parses, 512 unique edges, 128 KiB per source, and 512 KiB total. Metadata is
// checked before a source is read, so generated/minified bundles cannot consume
// the parse budget. Queued candidates are bounded separately because seeding a
// package enumerates its whole source and dependency list before any file is
// popped, and every queued path is retained as an invalidation input.
const DEFAULT_BUDGET: ReachabilityBudget = ReachabilityBudget {
    max_packages: 128,
    max_files: 128,
    max_queued_files: 4096,
    max_file_bytes: 128 * 1024,
    max_total_bytes: 512 * 1024,
    max_edges: 512,
    max_parses: 128,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ReachabilityOutcome {
    #[default]
    DoesNotReachVue,
    ReachesVue,
    BudgetExceeded,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ReachabilityWork {
    pub files: usize,
    pub bytes: usize,
    pub edges: usize,
    pub parses: usize,
    pub packages: usize,
}

#[derive(Clone, Debug, Default)]
pub struct PackageRouteReachability {
    pub outcome: ReachabilityOutcome,
    /// Unique, byte-ordered (`as_os_str`) inputs. Shared, so every importer
    /// of the route reuses one sorted set instead of copying and re-sorting it.
    pub inputs: Arc<[PathBuf]>,
    pub work: ReachabilityWork,
}

impl PackageRouteReachability {
    pub fn requires_shadow(&self) -> bool {
        self.outcome == ReachabilityOutcome::ReachesVue
    }

    pub fn requires_tracking(&self) -> bool {
        self.outcome != ReachabilityOutcome::DoesNotReachVue
    }

    pub fn record_work(&self, resolver: &mut crate::PackageRouteResolver) {
        resolver.record_reachability_work(
            self.work.files,
            self.work.bytes,
            self.work.edges,
            self.work.parses,
            self.work.packages,
            self.outcome == ReachabilityOutcome::BudgetExceeded,
        );
    }
}

#[derive(Clone, Copy, Debug)]
struct ReachabilityBudget {
    max_packages: usize,
    max_files: usize,
    max_queued_files: usize,
    max_file_bytes: usize,
    max_total_bytes: usize,
    max_edges: usize,
    max_parses: usize,
}

#[path = "package_route_reachability_route_graph.rs"]
mod route_graph;
#[path = "package_route_reachability_sources.rs"]
mod sources;
pub use sources::ReachabilitySources;

#[expect(clippy::disallowed_types, reason = "serde_json keys are std String")] // Compiler-option aliases originate in serde_json maps.
pub(crate) fn package_route_reaches_vue(
    route: &crate::PackageRoute,
    aliases: &[(std::string::String, std::string::String)],
    resolution: &super::package_resolution::PackageResolutionSettings,
    resolver: &mut crate::PackageRouteResolver,
    source_options: crate::PackageSourceOptions,
    sources: &mut ReachabilitySources,
) -> PackageRouteReachability {
    package_route_reaches_vue_with_budget(
        route,
        aliases,
        resolution,
        resolver,
        source_options,
        sources,
        DEFAULT_BUDGET,
    )
}

/// Determine whether one package route reaches a Vue source without allowing
/// an arbitrary dependency graph to delay native project startup.
pub fn scan_package_route_reachability<ResolveLocal, ResolvePackage>(
    route: &crate::PackageRoute,
    sources: &mut ReachabilitySources,
    resolve_local: ResolveLocal,
    resolve_package: ResolvePackage,
) -> PackageRouteReachability
where
    ResolveLocal: FnMut(&Path, &str) -> (Option<PathBuf>, Vec<PathBuf>),
    ResolvePackage: FnMut(
        &Path,
        &str,
        crate::PackageResolutionMode,
    ) -> (Option<crate::PackageRoute>, Vec<PathBuf>),
{
    scan_package_route_reachability_with_budget(
        route,
        sources,
        resolve_local,
        resolve_package,
        DEFAULT_BUDGET,
    )
}

#[expect(clippy::disallowed_types, reason = "serde_json keys are std String")] // Compiler-option aliases originate in serde_json maps.
fn package_route_reaches_vue_with_budget(
    route: &crate::PackageRoute,
    aliases: &[(std::string::String, std::string::String)],
    resolution: &super::package_resolution::PackageResolutionSettings,
    resolver: &mut crate::PackageRouteResolver,
    source_options: crate::PackageSourceOptions,
    sources: &mut ReachabilitySources,
    budget: ReachabilityBudget,
) -> PackageRouteReachability {
    scan_package_route_reachability_with_budget(
        route,
        sources,
        |importer, specifier| {
            let importer_dir = importer.parent().unwrap_or(importer);
            resolve_dependency_with_inputs(specifier, importer_dir, &route.package_root, aliases)
        },
        |importer, specifier, mode| {
            let importer_dir = importer.parent().unwrap_or(importer);
            let (context, mut inputs) = resolution.context(resolver, importer, mode);
            let (nested, consulted) = resolver
                .lookup_with_context(importer_dir, specifier, source_options, context)
                .into_parts();
            inputs.extend(consulted);
            (nested, inputs)
        },
        budget,
    )
}

fn scan_package_route_reachability_with_budget<ResolveLocal, ResolvePackage>(
    route: &crate::PackageRoute,
    sources: &mut ReachabilitySources,
    mut resolve_local: ResolveLocal,
    mut resolve_package: ResolvePackage,
    budget: ReachabilityBudget,
) -> PackageRouteReachability
where
    ResolveLocal: FnMut(&Path, &str) -> (Option<PathBuf>, Vec<PathBuf>),
    ResolvePackage: FnMut(
        &Path,
        &str,
        crate::PackageResolutionMode,
    ) -> (Option<crate::PackageRoute>, Vec<PathBuf>),
{
    let mut queued = FxHashSet::default();
    let mut queue = BinaryHeap::new();
    // Deduplicated on arrival: lookups re-report the same manifests and
    // context files for every edge, which made the final sort dominate.
    let mut inputs = FxHashSet::default();
    let mut work = ReachabilityWork::default();
    let mut packages = FxHashSet::default();
    if !route_graph::seed_route_graph(
        route,
        &mut packages,
        &mut queue,
        &mut queued,
        &mut inputs,
        &mut work,
        budget,
    ) {
        return finish(ReachabilityOutcome::BudgetExceeded, inputs, work, budget);
    }

    if queued
        .iter()
        .any(|path| path.extension().is_some_and(|extension| extension == "vue"))
    {
        return finish(ReachabilityOutcome::ReachesVue, inputs, work, budget);
    }

    let mut edges = FxHashSet::<(PathBuf, CompactString, crate::PackageResolutionMode)>::default();
    while let Some((_, Reverse(path))) = queue.pop() {
        record_input(&mut inputs, &path);
        if work.files == budget.max_files {
            return finish(ReachabilityOutcome::BudgetExceeded, inputs, work, budget);
        }
        work.files += 1;

        if !sources.is_loaded(&path) {
            // Load the files this scan can still pop, in pop order, at once;
            // later pops and other scans of this pass reuse them.
            let mut next = queue.iter().collect::<Vec<_>>();
            next.sort_unstable_by(|left, right| right.cmp(left));
            let remaining = budget.max_files - work.files;
            sources.prefetch(
                std::iter::once(&path).chain(
                    next.into_iter()
                        .take(remaining)
                        .map(|(_, Reverse(path))| path),
                ),
                budget.max_file_bytes,
            );
        }
        let Some(file_len) = sources.len(&path) else {
            continue;
        };
        let Ok(file_bytes) = usize::try_from(file_len) else {
            return finish(ReachabilityOutcome::BudgetExceeded, inputs, work, budget);
        };
        if file_bytes > budget.max_file_bytes
            || work.bytes.saturating_add(file_bytes) > budget.max_total_bytes
        {
            return finish(ReachabilityOutcome::BudgetExceeded, inputs, work, budget);
        }
        let Some(content) = sources.text(&path) else {
            continue;
        };
        if content.len > budget.max_file_bytes
            || work.bytes.saturating_add(content.len) > budget.max_total_bytes
        {
            return finish(ReachabilityOutcome::BudgetExceeded, inputs, work, budget);
        }
        work.bytes += content.len;
        if work.parses == budget.max_parses {
            return finish(ReachabilityOutcome::BudgetExceeded, inputs, work, budget);
        }
        work.parses += 1;
        for (specifier, mode) in content.occurrences.iter() {
            let (specifier, mode) = (specifier.clone(), *mode);
            if !edges.insert((path.clone(), specifier.clone(), mode)) {
                continue;
            }
            if work.edges == budget.max_edges {
                return finish(ReachabilityOutcome::BudgetExceeded, inputs, work, budget);
            }
            work.edges += 1;
            let (local_dependency, local_inputs) = resolve_local(&path, &specifier);
            inputs.extend(local_inputs);
            if let Some(dependency) = local_dependency {
                record_input(&mut inputs, &dependency);
                if dependency
                    .extension()
                    .is_some_and(|extension| extension == "vue")
                {
                    return finish(ReachabilityOutcome::ReachesVue, inputs, work, budget);
                }
                if !queued.contains(&dependency) {
                    if queued.len() == budget.max_queued_files {
                        return finish(ReachabilityOutcome::BudgetExceeded, inputs, work, budget);
                    }
                    queued.insert(dependency.clone());
                    queue.push((source_priority(&dependency), Reverse(dependency)));
                }
                continue;
            }
            if specifier.starts_with('.') || Path::new(specifier.as_str()).is_absolute() {
                continue;
            }
            // Canon supplies Vue's compiler/runtime types. They are terminal
            // support edges, not importer-scoped component packages.
            if is_vue_runtime_support_specifier(&specifier) {
                continue;
            }
            let (nested, consulted) = resolve_package(&path, &specifier, mode);
            inputs.extend(consulted);
            if let Some(nested) = nested {
                if !route_graph::seed_route_graph(
                    &nested,
                    &mut packages,
                    &mut queue,
                    &mut queued,
                    &mut inputs,
                    &mut work,
                    budget,
                ) {
                    return finish(ReachabilityOutcome::BudgetExceeded, inputs, work, budget);
                }
                if queued.iter().any(|source| {
                    source
                        .extension()
                        .is_some_and(|extension| extension == "vue")
                }) {
                    return finish(ReachabilityOutcome::ReachesVue, inputs, work, budget);
                }
            }
        }
    }
    finish(ReachabilityOutcome::DoesNotReachVue, inputs, work, budget)
}

fn enqueue(
    queue: &mut BinaryHeap<(u8, Reverse<PathBuf>)>,
    queued: &mut FxHashSet<PathBuf>,
    path: PathBuf,
) {
    if queued.insert(path.clone()) {
        queue.push((source_priority(&path), Reverse(path)));
    }
}

fn source_priority(path: &Path) -> u8 {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("vue") => 3,
        Some("ts" | "tsx" | "mts" | "cts") => 2,
        Some("js" | "jsx" | "mjs" | "cjs") => 1,
        _ => 0,
    }
}

fn record_input(inputs: &mut FxHashSet<PathBuf>, path: &Path) {
    if !inputs.contains(path) {
        inputs.insert(path.to_path_buf());
    }
}

fn finish(
    outcome: ReachabilityOutcome,
    inputs: FxHashSet<PathBuf>,
    work: ReachabilityWork,
    budget: ReachabilityBudget,
) -> PackageRouteReachability {
    let mut inputs = inputs.into_iter().collect::<Vec<_>>();
    inputs.sort_unstable_by(|left, right| left.as_os_str().cmp(right.as_os_str()));
    debug_assert!(
        outcome == ReachabilityOutcome::BudgetExceeded
            || (work.files <= budget.max_files
                && work.packages <= budget.max_packages
                && work.bytes <= budget.max_total_bytes
                && work.edges <= budget.max_edges
                && work.parses <= budget.max_parses)
    );
    PackageRouteReachability {
        outcome,
        inputs: inputs.into(),
        work,
    }
}

#[cfg(test)]
#[path = "package_route_reachability_tests.rs"]
mod tests;
