//! Distinguishes an authored `.vue.ts` import from an SFC mirror collision.

use std::path::Path;

use super::virtual_rewrite::append_extension;

/// Whether an explicit `.vue.ts`/`.vue.tsx` specifier is proven to resolve only
/// because Vize materializes the sibling `.vue` file at that exact path.
pub(super) fn unresolved_authored_vue_ts_collides_with_sfc(
    specifier: &str,
    source_dir: Option<&Path>,
) -> bool {
    let suffix = if specifier.ends_with(".vue.tsx") {
        ".tsx"
    } else if specifier.ends_with(".vue.ts") {
        ".ts"
    } else {
        return false;
    };

    let relative = specifier.starts_with("./") || specifier.starts_with("../");
    let path = Path::new(specifier);
    let candidate = if path.is_absolute() {
        path.to_path_buf()
    } else if relative {
        let Some(source_dir) = source_dir else {
            return false;
        };
        source_dir.join(path)
    } else {
        // Bare and aliased paths need the project resolver. Absence cannot be
        // established from the authored directory alone.
        return false;
    };

    let Some(name) = candidate.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    let Some(sfc_name) = name.strip_suffix(suffix) else {
        return false;
    };
    let sfc_path = candidate.with_file_name(sfc_name);
    if !sfc_path.is_file() {
        return false;
    }

    // TypeScript first substitutes the explicit TS/TSX suffix, then appends
    // the standard candidate set to the full spelling, and finally tries the
    // full spelling as a directory. Poison only when every native candidate is
    // proven absent; permission/metadata errors stay conservatively untouched.
    let stripped_extensions: &[&str] = if suffix == ".tsx" {
        &[".tsx", ".ts", ".d.ts", ".jsx", ".js"]
    } else {
        &[".ts", ".tsx", ".d.ts", ".js", ".jsx"]
    };
    stripped_extensions
        .iter()
        .all(|extension| path_is_proven_absent(&append_extension(&sfc_path, extension)))
        && [".ts", ".tsx", ".d.ts", ".js", ".jsx"]
            .iter()
            .all(|extension| path_is_proven_absent(&append_extension(&candidate, extension)))
        && path_is_proven_absent(&candidate)
}

fn path_is_proven_absent(path: &Path) -> bool {
    matches!(path.try_exists(), Ok(false))
}
