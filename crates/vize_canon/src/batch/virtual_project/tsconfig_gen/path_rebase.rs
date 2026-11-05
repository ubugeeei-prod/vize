//! Declaration mirror rebasing remains in Canon.
use std::path::{Component, Path, PathBuf};
pub(super) use vize_carton::typescript_project::is_url;

/// Rebase a configured `rootDir` onto the virtual mirror.
///
/// The mirror reproduces paths relative to the project root, so a `rootDir` of
/// `./lib` becomes `<virtual_root>/lib`. Relative values are already rebased onto
/// the project root by the shared compiler-option loader, which resolves them against the
/// tsconfig that declares them, so an inherited `rootDir` keeps the base config's
/// meaning. Returns `None` when nothing is configured, or when the configured
/// directory falls outside the project root and therefore has no mirror
/// counterpart — in each case the caller keeps inferring the layout.
pub(super) fn root_dir_into_mirror(
    project_root: &Path,
    virtual_root: &Path,
    configured: Option<&str>,
) -> Option<PathBuf> {
    let configured = Path::new(configured?);
    let absolute = if configured.is_absolute() {
        configured.to_path_buf()
    } else {
        project_root.join(configured)
    };
    let absolute = vize_carton::path::canonicalize_non_verbatim(&absolute);
    let relative = absolute.strip_prefix(project_root).ok()?;
    // `canonicalize` returns the input untouched for a directory that does not
    // exist, so `..` can survive and `strip_prefix` still matches lexically
    // (`<root>/../outside` starts with `<root>`). Such a path has no mirror
    // counterpart.
    if relative
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return None;
    }
    Some(virtual_root.join(relative))
}
