//! Workspace graph files that invalidate a cached package route.
//!
//! These files are triggers only. The resolver never reads dependency choices
//! from a lockfile or workspace config; the installed links and package
//! manifests remain the resolution authority.

use std::path::{Path, PathBuf};

const GRAPH_INPUTS: &[&str] = &[
    "pnpm-lock.yaml",
    "pnpm-workspace.yaml",
    "package-lock.json",
    "npm-shrinkwrap.json",
    "yarn.lock",
    "bun.lock",
    "bun.lockb",
    ".pnp.cjs",
    ".pnp.data.json",
];

pub(super) fn collect(start: &Path, out: &mut Vec<PathBuf>) {
    for ancestor in start.ancestors() {
        out.extend(GRAPH_INPUTS.iter().map(|name| ancestor.join(name)));
    }
}

pub(super) fn is_large_lockfile(path: &Path) -> bool {
    path.file_name().is_some_and(|name| {
        matches!(
            name.to_str(),
            Some(
                "pnpm-lock.yaml"
                    | "package-lock.json"
                    | "npm-shrinkwrap.json"
                    | "yarn.lock"
                    | "bun.lock"
                    | "bun.lockb"
                    | ".pnp.data.json"
            )
        )
    })
}
