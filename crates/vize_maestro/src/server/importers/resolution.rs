use super::comparable_path;
use std::path::{Path, PathBuf};
use vize_canon::{PackageRouteResolver, PackageSourceOptions};
use vize_s0::cstr;

const SCRIPT_EXTENSIONS: &[&str] = &["vue", "ts", "tsx", "js", "jsx", "mts", "cts", "mjs", "cjs"];

pub(super) struct ImportResolution {
    pub(super) dependencies: Vec<PathBuf>,
    pub(super) target_found: bool,
}

pub(super) fn resolve_import(
    importer_dir: &Path,
    specifier: &str,
    package_routes: &mut PackageRouteResolver,
) -> ImportResolution {
    let specifier = if specifier.starts_with('#') {
        specifier
    } else {
        specifier
            .split_once(['?', '#'])
            .map_or(specifier, |(path, _)| path)
    };
    if specifier == "."
        || specifier == ".."
        || specifier.starts_with("./")
        || specifier.starts_with("../")
    {
        let dependencies: Vec<_> = resolve_relative_import(importer_dir, specifier)
            .into_iter()
            .collect();
        return ImportResolution {
            target_found: !dependencies.is_empty(),
            dependencies,
        };
    }

    let lookup = package_routes.lookup(
        importer_dir,
        specifier,
        PackageSourceOptions::new(true, true),
    );
    let (route, dependencies) = lookup.into_parts();
    ImportResolution {
        dependencies,
        target_found: route.is_some(),
    }
}

fn resolve_relative_import(importer_dir: &Path, specifier: &str) -> Option<PathBuf> {
    let joined = importer_dir.join(specifier);
    if matches!(specifier, "." | "..") {
        return SCRIPT_EXTENSIONS
            .iter()
            .map(|extension| joined.join("index").with_extension(extension))
            .find(|candidate| candidate.exists())
            .map(|candidate| comparable_path(&candidate));
    }
    if has_script_extension(Path::new(specifier)) {
        return Some(comparable_path(&joined));
    }
    SCRIPT_EXTENSIONS
        .iter()
        .map(|extension| append_extension(&joined, extension))
        .chain(
            SCRIPT_EXTENSIONS
                .iter()
                .map(|extension| joined.join("index").with_extension(extension)),
        )
        .find(|candidate| candidate.exists())
        .map(|candidate| comparable_path(&candidate))
}

fn has_script_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| SCRIPT_EXTENSIONS.contains(&extension))
}

fn append_extension(path: &Path, extension: &str) -> PathBuf {
    path.file_name().and_then(|name| name.to_str()).map_or_else(
        || path.to_path_buf(),
        |name| path.with_file_name(cstr!("{name}.{extension}")),
    )
}
