//! Distinguishes an authored `.vue.ts` import from an SFC mirror collision.

use std::path::Path;

use super::virtual_rewrite::append_extension;

pub(super) enum VueTsCollision {
    /// No authored candidate exists; use the child-path poison from #3482.
    Unresolved,
    /// TypeScript has an authored candidate that needs a collision-free alias.
    Authored,
}

/// Whether an explicit `.vue.ts`/`.vue.tsx` specifier shares its virtual path
/// with the generated mirror for a sibling SFC.
pub(super) fn authored_vue_ts_collides_with_sfc(
    specifier: &str,
    source_dir: Option<&Path>,
) -> Option<VueTsCollision> {
    let suffix = if specifier.ends_with(".vue.tsx") {
        ".tsx"
    } else if specifier.ends_with(".vue.ts") {
        ".ts"
    } else {
        return None;
    };

    let relative = specifier.starts_with("./") || specifier.starts_with("../");
    let path = Path::new(specifier);
    let candidate = if path.is_absolute() {
        path.to_path_buf()
    } else if relative {
        let source_dir = source_dir?;
        source_dir.join(path)
    } else {
        // Bare and aliased paths need the project resolver. Absence cannot be
        // established from the authored directory alone.
        return None;
    };

    let name = candidate.file_name().and_then(|name| name.to_str())?;
    let sfc_name = name.strip_suffix(suffix)?;
    let sfc_path = candidate.with_file_name(sfc_name);
    if !sfc_path.is_file() {
        return None;
    }

    let stripped_extensions: &[&str] = if suffix == ".tsx" {
        &[".tsx", ".ts", ".d.ts", ".jsx", ".js"]
    } else {
        &[".ts", ".tsx", ".d.ts", ".js", ".jsx"]
    };
    let missing = stripped_extensions
        .iter()
        .all(|extension| path_is_proven_absent(&append_extension(&sfc_path, extension)))
        && [".ts", ".tsx", ".d.ts", ".js", ".jsx"]
            .iter()
            .all(|extension| path_is_proven_absent(&append_extension(&candidate, extension)))
        && path_is_proven_absent(&candidate);
    Some(if missing {
        VueTsCollision::Unresolved
    } else {
        VueTsCollision::Authored
    })
}

fn path_is_proven_absent(path: &Path) -> bool {
    matches!(path.try_exists(), Ok(false))
}
