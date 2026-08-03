//! Resolves the program selected by the original `check` invocation.

use std::path::{Path, PathBuf};

use super::resolve::{resolve_project_root, resolve_tsconfig_path};

/// Resolve before collecting transitive imports: imports outside the package
/// may widen the virtual mirror, but must not select a different `tsconfig`.
pub(super) fn resolve_invocation_program(
    effective_tsconfig: Option<&Path>,
    cwd: &Path,
) -> (PathBuf, Option<PathBuf>) {
    let project_root = resolve_project_root(effective_tsconfig, cwd, &[]);
    let tsconfig_path = resolve_tsconfig_path(effective_tsconfig, cwd, &project_root, &[]);
    (project_root, tsconfig_path)
}
