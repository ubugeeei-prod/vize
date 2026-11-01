//! Untimed, measurement-overlay-only census of the route handoff inputs.
//! Never compiled into the product candidate or used as a timing sample.

use std::{fs::OpenOptions, io::Write, mem::size_of, path::Path};

use serde::Serialize;
use vize_canon::{PackageRoute, PackageRouteBinding, PackageRouteSource};

#[derive(Default, Serialize)]
struct Counts {
    route_nodes: usize,
    source_paths: usize,
    dependency_paths: usize,
    source_targets: usize,
    nested_routes: usize,
    path_buffer_length_bytes: usize,
    path_buffer_capacity_bytes: usize,
    route_vector_initialized_bytes: usize,
    route_vector_capacity_bytes: usize,
    package_name_length_bytes_excluded_from_totals: usize,
}

impl Counts {
    fn path(&mut self, path: &std::path::PathBuf) {
        self.path_buffer_length_bytes += path.as_os_str().as_encoded_bytes().len();
        self.path_buffer_capacity_bytes += path.capacity();
    }

    fn route(&mut self, route: &PackageRoute) {
        self.route_nodes += 1;
        self.source_paths += route.source_paths.len();
        self.dependency_paths += route.dependency_paths.len();
        self.source_targets += route.source_targets.len();
        self.nested_routes += route.nested_routes.len();
        self.route_vector_initialized_bytes += (route.source_paths.len()
            + route.dependency_paths.len())
            * size_of::<std::path::PathBuf>()
            + route.source_targets.len() * size_of::<PackageRouteSource>()
            + route.nested_routes.len() * size_of::<PackageRoute>();
        self.route_vector_capacity_bytes += (route.source_paths.capacity()
            + route.dependency_paths.capacity())
            * size_of::<std::path::PathBuf>()
            + route.source_targets.capacity() * size_of::<PackageRouteSource>()
            + route.nested_routes.capacity() * size_of::<PackageRoute>();
        for path in route.source_paths.iter().chain(&route.dependency_paths) {
            self.path(path);
        }
        for target in &route.source_targets {
            self.path(&target.target_path);
            self.path(&target.source_path);
            self.path(&target.native_probe_path);
        }
        self.path(&route.package_root);
        self.path(&route.package_link_root);
        self.path(&route.manifest_path);
        if let Some(name) = &route.package_name {
            self.package_name_length_bytes_excluded_from_totals += name.len();
        }
        for nested in &route.nested_routes {
            self.route(nested);
        }
    }
}

#[derive(Serialize)]
struct Observation<'a> {
    schema: &'static str,
    program_root: &'a Path,
    bindings: usize,
    positive_bindings: usize,
    negative_bindings: usize,
    binding_inline_initialized_bytes: usize,
    routes: Counts,
    binding_invalidation_path_lengths: usize,
    binding_invalidation_path_capacities: usize,
    shared_invalidation_arc_slots: usize,
    binding_population: &'static str,
    positive_route_population: &'static str,
    outer_vec_capacity: Option<usize>,
    scope: &'static str,
}

/// Caller inserts this immediately before the original consuming setter in
/// BOTH instrumented roles: `record(&input.package_routes, &input.program_root)`.
/// A fresh, closed one-program cohort owns one output file. A repeated call
/// refuses instead of truncating or silently accepting partial coverage.
pub(super) fn record(
    routes: &[PackageRouteBinding],
    program_root: &Path,
) -> Result<(), std::io::Error> {
    let output = std::env::var_os("VIZE_ROUTE_MEMORY_OBSERVATION").ok_or_else(|| {
        std::io::Error::other("route-memory observation path is required for this overlay")
    })?;
    let mut counts = Counts::default();
    let mut positives = 0;
    let mut invalidation_lengths = 0;
    let mut invalidation_capacities = 0;
    let mut shared_arc_slots = 0;
    for binding in routes {
        if let Some(route) = &binding.route {
            positives += 1;
            counts.route(route);
        }
        for path in &binding.invalidation_paths {
            invalidation_lengths += path.as_os_str().as_encoded_bytes().len();
            invalidation_capacities += path.capacity();
        }
        shared_arc_slots += binding.shared_invalidation_paths.len();
    }
    let observation = Observation {
        schema: "vize.routeHandoffMemoryObservation.v2",
        program_root,
        bindings: routes.len(),
        positive_bindings: positives,
        negative_bindings: routes.len() - positives,
        binding_inline_initialized_bytes: routes.len() * size_of::<PackageRouteBinding>(),
        routes: counts,
        binding_invalidation_path_lengths: invalidation_lengths,
        binding_invalidation_path_capacities: invalidation_capacities,
        shared_invalidation_arc_slots: shared_arc_slots,
        binding_population: if routes.is_empty() {
            "empty"
        } else {
            "nonempty"
        },
        positive_route_population: if positives == 0 { "none" } else { "present" },
        outer_vec_capacity: None,
        scope: "all binding initialized slots plus route graph vectors and path buffers; negative bindings can still own invalidation buffers and Arc slots; outer Vec capacity unknown through slice; capacities are not clone allocations; package name heap, context, indexes, allocator overhead and shared Arc payload are excluded; no RSS/live-heap saving claim",
    };
    let raw = serde_json::to_vec_pretty(&observation)?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)?;
    file.write_all(&raw)?;
    file.write_all(b"\n")?;
    Ok(())
}
