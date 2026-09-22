//! Project route names for editor completions, read from the Vue Router
//! provider's `route-params` fact group (P4-10a).
//!
//! The host side of the provider contract: Maestro supplies the script
//! modules of the file's project — the provider's declared ambient input,
//! narrowed to modules that can contribute a route (`createRouter` or a
//! `path` key) and bounded so a completion never walks an unbounded tree.

use std::fs;
use std::path::{Path, PathBuf};

use vize_croquis_cf::providers::vue_router::{NamedRoute, RouteParams};
use vize_croquis_cf::providers::{PROJECT_FACTS, ProjectSources};
use vize_davinci::fact::{Demand, FactConsumer, FactGroup, FactManager};
use vize_s0::String;

/// How many ancestors are searched for the project's `package.json`.
const MAX_ANCESTORS: usize = 16;
/// How many script modules one scan reads at most.
const MAX_MODULES: usize = 1024;
/// How deep below the source root a scan descends.
const MAX_DEPTH: usize = 8;
/// Larger files are generated bundles, not route modules.
const MAX_BYTES: u64 = 256 * 1024;
const SCRIPT_EXTENSIONS: &[&str] = &["ts", "tsx", "mts", "cts", "js", "jsx", "mjs", "cjs"];
const SKIPPED_DIRS: &[&str] = &[
    "node_modules",
    "dist",
    ".git",
    ".nuxt",
    ".output",
    "coverage",
];

/// Maestro's route-name completion, a declared consumer of `route-params`.
struct RouteNameCompletion;

impl FactConsumer for RouteNameCompletion {
    const NAME: &'static str = "maestro/route-name-completion";
    const DEMAND: Demand = Demand::NONE.with(RouteParams::ID);
}

/// One completable route: its name and, when static, its path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ProjectRoute {
    pub(super) name: String,
    pub(super) path: Option<String>,
}

/// Every named route of the project `file` belongs to, in name order.
pub(super) fn project_routes(file: &Path) -> Vec<ProjectRoute> {
    let Some(root) = project_root(file) else {
        return Vec::new();
    };
    let source_root = if root.join("src").is_dir() {
        root.join("src")
    } else {
        root.clone()
    };
    let mut paths = Vec::new();
    collect_scripts(&source_root, 0, &mut paths);
    let mut project = ProjectSources::new();
    for path in &paths {
        let Ok(source) = fs::read_to_string(path) else {
            continue;
        };
        if !(source.contains("createRouter") || source.contains("path")) {
            continue;
        }
        let relative = path.strip_prefix(&root).unwrap_or(path);
        project.add(&relative.to_string_lossy(), &source);
    }
    let mut manager = FactManager::new(&PROJECT_FACTS);
    let Ok(view) = manager.prepare::<RouteNameCompletion>(&project) else {
        return Vec::new();
    };
    let Ok(named) = view.get::<RouteParams>() else {
        return Vec::new();
    };
    named
        .iter()
        .map(|(name, route): (&String, &NamedRoute)| ProjectRoute {
            name: name.clone(),
            path: (route.declarations == 1)
                .then(|| route.path.clone())
                .flatten(),
        })
        .collect()
}

fn project_root(file: &Path) -> Option<PathBuf> {
    file.ancestors()
        .skip(1)
        .take(MAX_ANCESTORS)
        .find(|dir| dir.join("package.json").is_file())
        .map(Path::to_path_buf)
}

fn collect_scripts(dir: &Path, depth: usize, out: &mut Vec<PathBuf>) {
    if depth > MAX_DEPTH || out.len() >= MAX_MODULES {
        return;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<PathBuf> = entries.flatten().map(|entry| entry.path()).collect();
    entries.sort();
    for path in entries {
        if out.len() >= MAX_MODULES {
            return;
        }
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if path.is_dir() {
            if !SKIPPED_DIRS.contains(&name) {
                collect_scripts(&path, depth + 1, out);
            }
        } else if path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| SCRIPT_EXTENSIONS.contains(&extension))
            && !name.ends_with(".d.ts")
            && fs::metadata(&path).is_ok_and(|meta| meta.len() <= MAX_BYTES)
        {
            out.push(path);
        }
    }
}
