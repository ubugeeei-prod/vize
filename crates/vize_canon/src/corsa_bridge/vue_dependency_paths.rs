//! Filesystem resolution helpers for the editor dependency walk.

use std::path::{Path, PathBuf};

pub(super) fn resolve_relative_script_import(dir: &Path, specifier: &str) -> Option<PathBuf> {
    crate::batch::virtual_project::dependency_scan::resolve_dependency(specifier, dir, dir, &[])
        .filter(|path| has_known_script_extension(path))
        .map(|path| normalize_path(&path))
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
