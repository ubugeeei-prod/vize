//! Connected-component cost model for concurrent CLI programs.

use std::path::{Path, PathBuf};

use super::import_resolution::resolve_virtual_import;
use super::union_find::UnionFind;
use super::{ShardPlan, declares_program_wide_types, is_ambient_declaration};
use crate::batch::{VirtualFile, VirtualProject};
use vize_carton::{FxHashMap, String, cstr};

mod dependency_visibility;
mod shared_leaves;

/// Partition the project's source files into shard programs along the
/// connected components of their import graph. Files in different components
/// never load each other, so component-aligned shards check disjoint code and
/// duplicate no work; interconnected projects collapse into one big component
/// and degrade to a single, unsharded run instead of paying N near-full
/// programs. Only ambient `.d.ts` files and sources carrying module/global
/// declarations stay visible to every shard, since they affect the whole
/// program without being imported. Bounded cheap shared script leaves also
/// remain visible everywhere, rather than coupling all their importers.
pub(super) fn partition_virtual_files(project: &VirtualProject, servers: usize) -> ShardPlan<'_> {
    partition(project, servers, true)
}

fn partition(project: &VirtualProject, servers: usize, allow_leaves: bool) -> ShardPlan<'_> {
    let requested_servers = servers;
    let files = project.virtual_files_sorted();
    let mut partitioned: Vec<&VirtualFile> = Vec::new();
    let mut shared: Vec<&Path> = Vec::new();
    for file in files {
        // The program-wide check reads the original source: the generated Vue
        // wrapper carries no `declare global` of its own — the shared
        // ImportMeta augmentation lives once per program in the hoisted
        // helpers file (SHARED_HELPERS_FILE), which every shard includes.
        let program_wide = project
            .original_content_for_virtual(&file.virtual_path)
            .is_some_and(declares_program_wide_types);
        if program_wide || is_ambient_declaration(&file.original_path) {
            shared.push(file.virtual_path.as_path());
        } else {
            partitioned.push(file);
        }
    }

    let servers = servers.clamp(1, partitioned.len().max(1));
    let no_sharding = ShardPlan {
        shards: Vec::new(),
        owners: FxHashMap::default(),
    };
    if servers <= 1 {
        return no_sharding;
    }

    // Union files that load each other or the same unresolved modules. The
    // graph is a cost model, not a correctness requirement — ownership
    // filtering already deduplicates diagnostics — but files coupled through
    // shared sources would otherwise be re-checked by every shard whose
    // program loads them. Relative imports resolve exactly; project path
    // aliases (`@/…`) and workspace packages symlinked into `node_modules`
    // couple their importers, while bare npm specifiers are dependency cost
    // every program pays anyway.
    let index_by_virtual: FxHashMap<&Path, usize> = partitioned
        .iter()
        .enumerate()
        .map(|(index, file)| (file.virtual_path.as_path(), index))
        .collect();
    let imports: Vec<_> = partitioned
        .iter()
        .map(|file| import_specifiers(&file.content))
        .collect();
    let leaves = if allow_leaves {
        shared_leaves::select(&partitioned, &imports, &index_by_virtual, servers)
    } else {
        Default::default()
    };
    for (index, file) in partitioned.iter().enumerate() {
        if leaves.contains(&index) {
            shared.push(file.virtual_path.as_path());
        }
    }
    let alias_prefixes = project.path_alias_prefixes();
    let mut components = UnionFind::new(partitioned.len());
    let mut coupling_keys: FxHashMap<String, usize> = FxHashMap::default();
    for (index, (file, specifiers)) in partitioned.iter().zip(&imports).enumerate() {
        for &specifier in specifiers {
            if specifier.starts_with("./") || specifier.starts_with("../") {
                let Some(base) = file.virtual_path.parent() else {
                    continue;
                };
                let target = normalize_join(base, specifier);
                if let Some(target_index) = resolve_virtual_import(&target, &index_by_virtual) {
                    // Cheap script leaves stay present in every program but
                    // do not join otherwise independent source components.
                    if !leaves.contains(&target_index) {
                        components.union(index, target_index);
                    }
                } else {
                    // An unresolved local module: couple its importers.
                    let key = String::from(target.to_string_lossy());
                    match coupling_keys.get(key.as_str()) {
                        Some(&first) => components.union(index, first),
                        None => {
                            coupling_keys.insert(key, index);
                        }
                    }
                }
            } else if let Some(alias) = alias_prefixes
                .iter()
                .find(|alias| specifier.starts_with(alias.as_str()))
            {
                let key = cstr!("alias:{alias}");
                match coupling_keys.get(key.as_str()) {
                    Some(&first) => components.union(index, first),
                    None => {
                        coupling_keys.insert(key, index);
                    }
                }
            } else if let Some(package) =
                project.workspace_package_route_identity(&file.original_path, specifier)
            {
                let key = cstr!("workspace:{}", package.display());
                match coupling_keys.get(key.as_str()) {
                    Some(&first) => components.union(index, first),
                    None => {
                        coupling_keys.insert(key, index);
                    }
                }
            }
        }
    }

    // Bin-pack components (heaviest first) into the requested shard count and
    // only keep the plan when it buys real parallelism: a dominant component
    // means each extra program would mostly re-check the same files. Weights
    // are generated-content bytes, a usable proxy for parse+check cost.
    let mut component_files: FxHashMap<usize, Vec<usize>> = FxHashMap::default();
    for index in (0..partitioned.len()).filter(|index| !leaves.contains(index)) {
        component_files
            .entry(components.find(index))
            .or_default()
            .push(index);
    }
    let weight = |file_indices: &[usize]| -> usize {
        file_indices
            .iter()
            .filter_map(|&index| partitioned.get(index))
            .map(|file| file.content.len())
            .sum()
    };
    let mut component_groups: Vec<Vec<usize>> = component_files.into_values().collect();
    if component_groups.len() < 2 {
        return no_sharding;
    }
    let total_weight: usize = component_groups.iter().map(|group| weight(group)).sum();
    component_groups.sort_by(|left, right| {
        weight(right)
            .cmp(&weight(left))
            .then_with(|| left.first().cmp(&right.first()))
    });

    let servers = servers.min(component_groups.len());
    let mut bins: Vec<(usize, Vec<usize>)> = vec![(0, Vec::new()); servers];
    for group in component_groups {
        let Some(bin) = bins.iter_mut().min_by_key(|(bin_weight, _)| *bin_weight) else {
            return no_sharding;
        };
        bin.0 += weight(&group);
        bin.1.extend(group);
    }
    let largest = bins.iter().map(|(bin_weight, _)| *bin_weight).max();
    // Wall time tracks the heaviest shard; below ~25% savings the duplicated
    // per-program work on shared and ambient sources outweighs the win.
    if largest.unwrap_or(0) * 4 >= total_weight * 3 {
        return no_sharding;
    }

    // Only a useful leaf plan pays the additional semantic visibility screen.
    // Unchanged transitive/barrel graphs stop above without parsing scripts.
    // Uncertain plans replay the original cost model exactly, including its
    // independent-component parallelism and diagnostic ownership.
    if !leaves.is_empty()
        && (!shared_leaves::has_explicit_modules(&partitioned)
            || !dependency_visibility::permits_sharing(project, &partitioned, &index_by_virtual))
    {
        return partition(project, requested_servers, false);
    }

    let mut shards: Vec<Vec<&Path>> = Vec::with_capacity(bins.len());
    let mut owners = FxHashMap::default();
    for (shard_index, (_, file_indices)) in bins.into_iter().enumerate() {
        let mut include = shared.clone();
        for file in file_indices
            .into_iter()
            .filter_map(|file_index| partitioned.get(file_index))
        {
            include.push(file.virtual_path.as_path());
            owners.insert(file.original_path.clone(), shard_index);
        }
        shards.push(include);
    }

    shared_leaves::record_selection(&partitioned, &leaves, shards.len());
    vize_carton::profiler::global_profiler()
        .record_counter("canon.corsa.cli.shards", shards.len() as u64);
    ShardPlan { shards, owners }
}

/// Quoted module specifiers in generated virtual TS: `from '<spec>'`,
/// `import('<spec>')`, `import '<spec>'`, `require('<spec>')`. A lexical scan
/// is enough here — the result only feeds the shard cost model.
fn import_specifiers(content: &str) -> Vec<&str> {
    let mut specifiers = Vec::new();
    for token in ["from ", "import(", "import ", "require("] {
        for (at, _) in content.match_indices(token) {
            let rest = content
                .get(at + token.len()..)
                .unwrap_or_default()
                .trim_start();
            let Some((quote, rest)) = ['\'', '"']
                .into_iter()
                .find_map(|quote| Some((quote, rest.strip_prefix(quote)?)))
            else {
                continue;
            };
            let Some((specifier, _)) = rest.split_once(quote) else {
                continue;
            };
            specifiers.push(specifier);
        }
    }
    specifiers
}

fn normalize_join(base: &Path, specifier: &str) -> PathBuf {
    let mut normalized = base.to_path_buf();
    for component in Path::new(specifier).components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}
