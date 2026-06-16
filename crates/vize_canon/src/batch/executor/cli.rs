use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};

use super::super::{Diagnostic, TypeCheckResult, VirtualProject};
use crate::batch::error::{CorsaError, CorsaResult};
use crate::batch::executor::diagnostics::{
    DiagnosticMapper, dedup_diagnostics, relative_module_resolves_on_disk, should_skip_diagnostic,
    should_skip_original_diagnostic,
};
use vize_carton::path::canonicalize_non_verbatim;
use vize_carton::{FxHashMap, profile};
use vize_carton::{String, cstr};

pub(super) fn check_with_cli(
    corsa_path: &Path,
    project: &VirtualProject,
) -> CorsaResult<TypeCheckResult> {
    let config_path = project.virtual_root().join("tsconfig.json");
    run_cli_for_config(corsa_path, project, &config_path, Some(available_threads()))
}

fn available_threads() -> usize {
    std::thread::available_parallelism()
        .map(std::num::NonZero::get)
        .unwrap_or(1)
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
pub(super) fn check_with_cli_sharded(
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
    // Split the machine's checker budget across the concurrent programs.
    let checkers = (available_threads() / config_paths.len()).max(4);
    let results = profile!("canon.corsa.cli.sharded", {
        std::thread::scope(|scope| {
            let handles: Vec<_> = config_paths
                .iter()
                .map(|config_path| {
                    scope.spawn(move || {
                        run_cli_for_config(corsa_path, project, config_path, Some(checkers))
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
        merged.diagnostics.extend(
            result
                .diagnostics
                .into_iter()
                .filter(|diagnostic| owners.get(&diagnostic.file).copied().unwrap_or(0) == index),
        );
    }
    Ok(merged)
}

/// Pick the shard count for a project when the caller did not request one.
/// Corsa's checker pool uses ~4 cores per process; sharding only pays off
/// once there are enough Vue files to amortize each extra program's fixed
/// parse/bind cost.
pub(super) fn auto_server_count(project: &VirtualProject) -> usize {
    let vue_files = project
        .virtual_files_sorted()
        .iter()
        .filter(|file| is_vue_original(&file.original_path))
        .count();
    let threads = std::thread::available_parallelism()
        .map(std::num::NonZero::get)
        .unwrap_or(1);
    (threads / 4).min(vue_files / 64).clamp(1, 8)
}

struct ShardPlan<'a> {
    /// Virtual paths to include per shard (owned Vue files plus every shared
    /// file).
    shards: Vec<Vec<&'a Path>>,
    /// Original path -> owning shard for partitioned Vue files; files absent
    /// from the map (shared sources, project-level anchors) belong to shard 0.
    owners: FxHashMap<PathBuf, usize>,
}

/// Partition the project's source files into shard programs along the
/// connected components of their import graph. Files in different components
/// never load each other, so component-aligned shards check disjoint code and
/// duplicate no work; interconnected projects collapse into one big component
/// and degrade to a single, unsharded run instead of paying N near-full
/// programs. Only ambient `.d.ts` files and sources carrying module/global
/// declarations stay visible to every shard, since they affect the whole
/// program without being imported.
fn partition_virtual_files(project: &VirtualProject, servers: usize) -> ShardPlan<'_> {
    let files = project.virtual_files_sorted();
    let mut partitioned: Vec<&super::super::VirtualFile> = Vec::new();
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
    let alias_prefixes = project.path_alias_prefixes();
    let mut components = UnionFind::new(partitioned.len());
    let mut coupling_keys: FxHashMap<String, usize> = FxHashMap::default();
    let mut workspace_packages: FxHashMap<String, bool> = FxHashMap::default();
    for (index, file) in partitioned.iter().enumerate() {
        for specifier in import_specifiers(&file.content) {
            if specifier.starts_with("./") || specifier.starts_with("../") {
                let Some(base) = file.virtual_path.parent() else {
                    continue;
                };
                let target = normalize_join(base, specifier);
                if let Some(target_index) = resolve_virtual_import(&target, &index_by_virtual) {
                    components.union(index, target_index);
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
                workspace_source_package(project.project_root(), specifier, &mut workspace_packages)
            {
                let key = cstr!("workspace:{package}");
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
    for index in 0..partitioned.len() {
        component_files
            .entry(components.find(index))
            .or_default()
            .push(index);
    }
    let weight = |file_indices: &[usize]| -> usize {
        file_indices
            .iter()
            .map(|&index| partitioned[index].content.len())
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
        let bin = bins
            .iter_mut()
            .min_by_key(|(bin_weight, _)| *bin_weight)
            .expect("at least one shard bin");
        bin.0 += weight(&group);
        bin.1.extend(group);
    }
    let largest = bins.iter().map(|(bin_weight, _)| *bin_weight).max();
    // Wall time tracks the heaviest shard; below ~25% savings the duplicated
    // per-program work on shared and ambient sources outweighs the win.
    if largest.unwrap_or(0) * 4 >= total_weight * 3 {
        return no_sharding;
    }

    let mut shards: Vec<Vec<&Path>> = Vec::with_capacity(bins.len());
    let mut owners = FxHashMap::default();
    for (shard_index, (_, file_indices)) in bins.into_iter().enumerate() {
        let mut include = shared.clone();
        for file_index in file_indices {
            let file = partitioned[file_index];
            include.push(file.virtual_path.as_path());
            owners.insert(file.original_path.clone(), shard_index);
        }
        shards.push(include);
    }

    ShardPlan { shards, owners }
}

/// Quoted module specifiers in generated virtual TS: `from '<spec>'`,
/// `import('<spec>')`, `import '<spec>'`, `require('<spec>')`. A lexical scan
/// is enough here — the result only feeds the shard cost model.
fn import_specifiers(content: &str) -> Vec<&str> {
    let mut specifiers = Vec::new();
    for token in ["from ", "import(", "import ", "require("] {
        for (at, _) in content.match_indices(token) {
            let rest = content[at + token.len()..].trim_start();
            let Some(quote) = rest.chars().next().filter(|ch| matches!(ch, '\'' | '"')) else {
                continue;
            };
            let rest = &rest[1..];
            let Some(end) = rest.find(quote) else {
                continue;
            };
            specifiers.push(&rest[..end]);
        }
    }
    specifiers
}

/// Whether a bare specifier resolves to a workspace package whose source is
/// symlinked into `node_modules` (pnpm/yarn workspaces): importing it drags
/// real project source into the program, so its importers are cost-coupled.
/// Regular npm packages resolve inside `node_modules` and are ambient cost
/// every program pays anyway. Results are cached per package root.
fn workspace_source_package<'spec>(
    project_root: &Path,
    specifier: &'spec str,
    cache: &mut FxHashMap<String, bool>,
) -> Option<&'spec str> {
    let mut segments = specifier.splitn(3, '/');
    let first = segments.next()?;
    let package_end = if first.starts_with('@') {
        first.len() + 1 + segments.next()?.len()
    } else {
        first.len()
    };
    let package = &specifier[..package_end];

    if let Some(&is_workspace) = cache.get(package) {
        return is_workspace.then_some(package);
    }

    let mut is_workspace = false;
    let mut dir = Some(project_root);
    while let Some(current) = dir {
        let candidate = current.join("node_modules").join(package);
        if let Ok(metadata) = std::fs::symlink_metadata(&candidate) {
            if metadata.file_type().is_symlink()
                && let Ok(target) = std::fs::canonicalize(&candidate)
            {
                is_workspace = !target
                    .components()
                    .any(|component| component.as_os_str() == "node_modules");
            }
            break;
        }
        dir = current.parent();
    }

    cache.insert(String::from(package), is_workspace);
    is_workspace.then_some(package)
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

struct UnionFind {
    parent: Vec<usize>,
}

impl UnionFind {
    fn new(size: usize) -> Self {
        Self {
            parent: (0..size).collect(),
        }
    }

    fn find(&mut self, node: usize) -> usize {
        let mut root = node;
        while self.parent[root] != root {
            root = self.parent[root];
        }
        let mut current = node;
        while self.parent[current] != root {
            let next = self.parent[current];
            self.parent[current] = root;
            current = next;
        }
        root
    }

    fn union(&mut self, left: usize, right: usize) {
        let left_root = self.find(left);
        let right_root = self.find(right);
        if left_root != right_root {
            self.parent[right_root] = left_root;
        }
    }
}

fn is_vue_original(path: &Path) -> bool {
    path.extension().is_some_and(|extension| extension == "vue")
}

fn is_ambient_declaration(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".d.ts"))
}

fn declares_program_wide_types(content: &str) -> bool {
    content.contains("declare module") || content.contains("declare global")
}

/// Resolve a normalized relative import target against the registered virtual
/// files, trying the extension candidates TypeScript would.
fn resolve_virtual_import(
    target: &Path,
    index_by_virtual: &FxHashMap<&Path, usize>,
) -> Option<usize> {
    if let Some(&index) = index_by_virtual.get(target) {
        return Some(index);
    }
    let target_str = target.to_string_lossy();
    for suffix in [".ts", ".tsx", ".d.ts", "/index.ts", "/index.tsx"] {
        let candidate = PathBuf::from(cstr!("{target_str}{suffix}").as_str());
        if let Some(&index) = index_by_virtual.get(candidate.as_path()) {
            return Some(index);
        }
    }
    None
}

fn run_cli_for_config(
    corsa_path: &Path,
    project: &VirtualProject,
    config_path: &Path,
    checkers: Option<usize>,
) -> CorsaResult<TypeCheckResult> {
    let output = profile!("canon.corsa.cli.command", {
        let mut command = Command::new(corsa_path);
        command.current_dir(project.virtual_root());
        // Corsa's checker pool defaults to four workers; size it to the share
        // of the machine this program gets so wide machines are not idle.
        if let Some(checkers) = checkers {
            command.arg("--checkers").arg(cstr!("{checkers}").as_str());
        }
        command
            .arg("--pretty")
            .arg("false")
            .arg("--project")
            .arg(config_path)
            .output()
    })?;
    let diagnostics = profile!(
        "canon.corsa.cli.parse",
        parse_output_diagnostics(&output, project)
    );

    // An older runtime without `--checkers` support rejects the whole
    // invocation with TS5023; retry once without the option.
    if checkers.is_some()
        && !output.status.success()
        && diagnostics.iter().any(|diagnostic| {
            diagnostic.code == Some(5023) && diagnostic.message.contains("checkers")
        })
    {
        return run_cli_for_config(corsa_path, project, config_path, None);
    }

    let success = output.status.success()
        && diagnostics
            .iter()
            .all(|diagnostic| diagnostic.severity != 1);

    // A non-zero exit is a runner failure only when the output carries no
    // diagnostic-shaped lines at all (bad invocation, crash, missing CLI
    // support). Recognizable diagnostics whose every entry was suppressed or
    // failed source mapping still prove the CLI ran the project; falling back
    // to the per-file project-session API there costs orders of magnitude
    // more wall time for the same answer.
    if !output.status.success()
        && diagnostics.is_empty()
        && !output_contains_diagnostic_lines(&output)
    {
        return Err(CorsaError::CorsaExecution {
            exit_code: output.status.code().unwrap_or(-1),
            message: output_message(&output),
        });
    }

    Ok(TypeCheckResult {
        exit_code: output.status.code().unwrap_or(if success { 0 } else { 1 }),
        success,
        diagnostics,
    })
}

fn parse_output_diagnostics(output: &Output, project: &VirtualProject) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut mapper = DiagnosticMapper::new(project);
    #[allow(clippy::disallowed_types)]
    let stdout = std::string::String::from_utf8_lossy(&output.stdout);
    parse_cli_diagnostics(stdout.as_ref(), project, &mut mapper, &mut diagnostics);
    #[allow(clippy::disallowed_types)]
    let stderr = std::string::String::from_utf8_lossy(&output.stderr);
    parse_cli_diagnostics(stderr.as_ref(), project, &mut mapper, &mut diagnostics);
    // A single template error surfaces twice — the dynamic prop binding it sits
    // on is generated at two virtual positions that map back to the same source
    // attribute span (#1389). Collapse exact duplicates at the collection point.
    dedup_diagnostics(diagnostics)
}

fn parse_cli_diagnostics(
    output: &str,
    project: &VirtualProject,
    mapper: &mut DiagnosticMapper<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for line in output.lines() {
        if let Some(diagnostic) = parse_cli_diagnostic_line(line, project, mapper) {
            diagnostics.push(diagnostic);
            continue;
        }
        // Project-level diagnostics carry no file position (`error TS2688:
        // Cannot find type definition file for 'x'.`). They are real,
        // user-actionable problems — tsc and vue-tsc report them and the
        // runtime may skip the semantic pass because of them — so they are
        // attributed to the project's tsconfig instead of being dropped.
        if let Some(diagnostic) = parse_global_diagnostic_line(line, project) {
            diagnostics.push(diagnostic);
            continue;
        }
        if is_cli_diagnostic_line(line) {
            continue;
        }

        let Some(last) = diagnostics.last_mut() else {
            continue;
        };
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        last.message.push('\n');
        last.message.push_str(line);
    }
}

/// Parse a file-less project-level diagnostic such as
/// `error TS2688: Cannot find type definition file for 'vite/client'.`
fn parse_global_diagnostic_line(line: &str, project: &VirtualProject) -> Option<Diagnostic> {
    let (severity, rest) = line.split_once(' ')?;
    let severity = match severity {
        "error" => 1,
        "warning" => 2,
        "info" => 3,
        _ => return None,
    };
    let (code, message) = rest.split_once(": ")?;
    let code = code.strip_prefix("TS")?.parse::<u32>().ok()?;
    if should_skip_diagnostic(Some(code), message) {
        return None;
    }

    Some(Diagnostic {
        file: project.project_diagnostics_anchor(),
        line: 0,
        column: 0,
        message: message.into(),
        code: Some(code),
        severity,
        block_type: None,
    })
}

fn parse_cli_diagnostic_line(
    line: &str,
    project: &VirtualProject,
    mapper: &mut DiagnosticMapper<'_>,
) -> Option<Diagnostic> {
    let (prefix, suffix) = line.split_once("): ")?;
    let open = prefix.rfind('(')?;
    let path = &prefix[..open];
    let position = &prefix[open + 1..];
    let (line, column) = position.split_once(',')?;
    let line = line.parse::<u32>().ok()?.saturating_sub(1);
    let column = column.parse::<u32>().ok()?.saturating_sub(1);

    let (severity, rest) = suffix.split_once(' ')?;
    let severity = match severity {
        "error" => 1,
        "warning" => 2,
        "info" => 3,
        _ => return None,
    };
    let (code, message) = rest.split_once(": ")?;
    let code = code
        .strip_prefix("TS")
        .and_then(|code| code.parse::<u32>().ok());
    if should_skip_diagnostic(code, message) {
        return None;
    }
    if code == Some(6133) && !mapper.preserves_unused_diagnostics() {
        return None;
    }

    let virtual_path = normalize_cli_path(path, project.virtual_root());
    let original = mapper.map_to_original(&virtual_path, line, column)?;
    if should_skip_original_diagnostic(code, &original) {
        return None;
    }

    // Suppress the false `TS2307` raised for a relative import of a sibling that
    // exists on disk but sits outside an explicit file subset. See the matching
    // check in `diagnostics::map_lsp_diagnostic`.
    if code == Some(2307) && relative_module_resolves_on_disk(message, &original.path) {
        return None;
    }

    Some(Diagnostic {
        file: original.path,
        line: original.line,
        column: original.column,
        message: message.into(),
        code,
        severity,
        block_type: original.block_type,
    })
}

fn output_contains_diagnostic_lines(output: &Output) -> bool {
    [&output.stdout, &output.stderr].into_iter().any(|stream| {
        #[allow(clippy::disallowed_types)]
        let text = std::string::String::from_utf8_lossy(stream);
        text.lines()
            .any(|line| is_cli_diagnostic_line(line) || is_global_diagnostic_line(line))
    })
}

/// Whether `line` is a file-less project-level diagnostic such as
/// `error TS2688: Cannot find type definition file for 'vite/client'.`
fn is_global_diagnostic_line(line: &str) -> bool {
    let Some(rest) = line
        .strip_prefix("error ")
        .or_else(|| line.strip_prefix("warning "))
        .or_else(|| line.strip_prefix("info "))
    else {
        return false;
    };
    let Some(code) = rest.strip_prefix("TS") else {
        return false;
    };
    let digits = code.bytes().take_while(u8::is_ascii_digit).count();
    digits > 0 && code[digits..].starts_with(':')
}

fn is_cli_diagnostic_line(line: &str) -> bool {
    let Some((prefix, suffix)) = line.split_once("): ") else {
        return false;
    };
    let Some(open) = prefix.rfind('(') else {
        return false;
    };
    let position = &prefix[open + 1..];
    let Some((line, column)) = position.split_once(',') else {
        return false;
    };
    if line.parse::<u32>().is_err() || column.parse::<u32>().is_err() {
        return false;
    }

    matches!(
        suffix.split_once(' ').map(|(severity, _)| severity),
        Some("error" | "warning" | "info")
    )
}

fn normalize_cli_path(path: &str, virtual_root: &Path) -> PathBuf {
    let path = PathBuf::from(path);
    let path = if path.is_absolute() {
        normalize_path_lexically(path.as_path())
    } else {
        normalize_path_lexically(virtual_root.join(path).as_path())
    };

    if path.exists() {
        let canonical_path = canonicalize_non_verbatim(path.as_path());
        let canonical_root = canonicalize_non_verbatim(virtual_root);
        if let Ok(relative) = canonical_path.strip_prefix(canonical_root.as_path()) {
            return virtual_root.join(relative);
        }
        canonical_path
    } else {
        path
    }
}

fn normalize_path_lexically(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                if !normalized.pop() && !normalized.has_root() {
                    normalized.push(component.as_os_str());
                }
            }
            _ => normalized.push(component.as_os_str()),
        }
    }
    normalized
}

fn output_message(output: &Output) -> String {
    #[allow(clippy::disallowed_types)]
    let stderr = std::string::String::from_utf8_lossy(&output.stderr);
    #[allow(clippy::disallowed_types)]
    let stdout = std::string::String::from_utf8_lossy(&output.stdout);
    let stderr = stderr.trim();
    let stdout = stdout.trim();
    if stderr.is_empty() {
        return stdout.to_owned().into();
    }
    if stdout.is_empty() {
        return stderr.to_owned().into();
    }
    cstr!("{}\n{}", stderr, stdout)
}

#[cfg(test)]
mod tests;
