//! Filesystem resolution helpers for the editor dependency walk.

use std::path::{Path, PathBuf};

use vize_carton::cstr;

pub(super) fn resolve_relative_script_import(dir: &Path, specifier: &str) -> Option<PathBuf> {
    let base = dir.join(specifier);
    if has_known_script_extension(&base) {
        return known_script_path(&base).then(|| normalize_path(&base));
    }

    for ext in [
        "ts", "tsx", "mts", "cts", "d.ts", "d.mts", "d.cts", "js", "jsx", "mjs", "cjs",
    ] {
        let candidate = append_extension(&base, ext);
        if candidate.exists() {
            return Some(normalize_path(&candidate));
        }
    }
    for name in [
        "index.ts",
        "index.tsx",
        "index.mts",
        "index.cts",
        "index.js",
        "index.jsx",
        "index.mjs",
        "index.cjs",
        "index.d.ts",
        "index.d.mts",
        "index.d.cts",
    ] {
        let candidate = base.join(name);
        if candidate.exists() {
            return Some(normalize_path(&candidate));
        }
    }
    None
}

fn has_known_script_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            matches!(
                extension,
                "ts" | "tsx" | "mts" | "cts" | "js" | "jsx" | "mjs" | "cjs"
            )
        })
}

fn known_script_path(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    path.exists()
        && (name.ends_with(".ts")
            || name.ends_with(".tsx")
            || name.ends_with(".mts")
            || name.ends_with(".cts")
            || name.ends_with(".js")
            || name.ends_with(".jsx")
            || name.ends_with(".mjs")
            || name.ends_with(".cjs"))
}

fn append_extension(path: &Path, extension: &str) -> PathBuf {
    path.file_name().and_then(|name| name.to_str()).map_or_else(
        || path.to_path_buf(),
        |name| path.with_file_name(cstr!("{name}.{extension}")),
    )
}

pub(super) fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                if !normalized.pop() {
                    normalized.push("..");
                }
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}

#[cfg(test)]
mod tests {
    use super::resolve_relative_script_import;

    #[test]
    fn resolves_directory_module_declaration_indices() {
        let project = tempfile::TempDir::new().expect("temp project");
        let src = project.path().join("src");
        std::fs::create_dir_all(src.join("esm")).expect("esm dir");
        std::fs::create_dir_all(src.join("cjs")).expect("cjs dir");

        let esm = src.join("esm").join("index.d.mts");
        let cjs = src.join("cjs").join("index.d.cts");
        let schema = src.join("schema.d.ts");
        std::fs::write(&esm, "export type Value = string;\n").expect("esm dts");
        std::fs::write(&cjs, "export type Value = string;\n").expect("cjs dts");
        std::fs::write(&schema, "export type Schema = { id: string };\n").expect("schema dts");

        assert_eq!(
            resolve_relative_script_import(&src, "./esm").as_deref(),
            Some(esm.as_path())
        );
        assert_eq!(
            resolve_relative_script_import(&src, "./cjs").as_deref(),
            Some(cjs.as_path())
        );
        assert_eq!(
            resolve_relative_script_import(&src, "./schema").as_deref(),
            Some(schema.as_path())
        );
    }

    #[test]
    fn resolves_extensionless_imports_with_dotted_basenames() {
        let project = tempfile::TempDir::new().expect("temp project");
        let src = project.path().join("src");
        std::fs::create_dir_all(&src).expect("src dir");
        let target = src.join("x.use.ts");
        std::fs::write(&target, "export const useX = () => 1;\n").expect("dotted module");

        assert_eq!(
            resolve_relative_script_import(&src, "./x.use").as_deref(),
            Some(target.as_path())
        );
    }
}
