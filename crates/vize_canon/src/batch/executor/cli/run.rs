use super::{
    CorsaError, CorsaResult, Path, TypeCheckResult, VirtualProject, checker_count, is_vue_original,
    partition_virtual_files, profile, run_cli_for_config, shard_count,
};

pub(in crate::batch::executor) fn check_with_cli(
    corsa_path: &Path,
    project: &VirtualProject,
) -> CorsaResult<TypeCheckResult> {
    let config_path = project.virtual_root().join("tsconfig.json");
    run_cli_for_config(corsa_path, project, &config_path, checker_count(), &|_| {
        true
    })
}

/// Run the project check sharded across `servers` concurrent Corsa CLI
/// processes. Corsa's own checker pool saturates around four cores, so on
/// wider machines a single process leaves most of the CPU idle; partitioning
/// the project along the connected components of its import graph restores
/// the parallelism while keeping each shard's program disjoint.
///
/// Ambient `.d.ts` files and sources carrying module/global declarations are
/// included in every shard so augmentations behave exactly as in the single
/// program. Each diagnostic is reported by the shard that owns its file, so
/// the merged result matches an unsharded run.
pub(in crate::batch::executor) fn check_with_cli_sharded(
    corsa_path: &Path,
    project: &VirtualProject,
    servers: usize,
) -> CorsaResult<TypeCheckResult> {
    let plan = partition_virtual_files(project, servers);
    if plan.shards.len() <= 1 {
        return check_with_cli(corsa_path, project);
    }

    let mut config_paths = Vec::with_capacity(plan.shards.len());
    for (index, shard) in plan.shards.iter().enumerate() {
        config_paths.push(profile!(
            "canon.corsa.cli.write_shard_tsconfig",
            project.write_shard_tsconfig(index, shard)
        )?);
    }

    let owners = &plan.owners;
    // Shards already parallelize across processes; each process keeps the
    // deterministic per-program checker count (see `checker_count`).
    let checkers = checker_count();
    let results = profile!("canon.corsa.cli.sharded", {
        std::thread::scope(|scope| {
            let handles: Vec<_> = config_paths
                .iter()
                .enumerate()
                .map(|(index, config_path)| {
                    scope.spawn(move || {
                        // Each shard evaluates the template directives of the
                        // files it owns; the merge keeps only the owner's.
                        let owns = |path: &Path| {
                            !matches!(owners.get(path), Some(owner) if *owner != index)
                        };
                        run_cli_for_config(corsa_path, project, config_path, checkers, &owns)
                    })
                })
                .collect();
            handles
                .into_iter()
                .map(|handle| {
                    handle.join().unwrap_or_else(|_| {
                        Err(CorsaError::CorsaExecution {
                            exit_code: -1,
                            message: "sharded corsa CLI worker panicked".into(),
                        })
                    })
                })
                .collect::<Vec<_>>()
        })
    });

    let mut merged = TypeCheckResult {
        exit_code: 0,
        success: true,
        diagnostics: Vec::new(),
    };
    for (index, result) in results.into_iter().enumerate() {
        let result = result?;
        merged.exit_code = merged.exit_code.max(result.exit_code);
        merged.success = merged.success && result.success;
        merged
            .diagnostics
            .extend(result.diagnostics.into_iter().filter(
            |diagnostic| !matches!(owners.get(&diagnostic.file), Some(owner) if *owner != index),
        ));
    }
    Ok(merged)
}

/// Pick the shard count for a project when the caller did not request one.
pub(in crate::batch::executor) fn auto_server_count(project: &VirtualProject) -> usize {
    let vue_files = project
        .virtual_files_sorted()
        .iter()
        .filter(|file| is_vue_original(&file.original_path))
        .count();
    let threads = std::thread::available_parallelism()
        .map(std::num::NonZero::get)
        .unwrap_or(1);
    shard_count(threads, checker_count(), vue_files)
}
