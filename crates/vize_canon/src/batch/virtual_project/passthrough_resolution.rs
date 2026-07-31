//! TypeScript-compatible probing for relative modules copied into the mirror.

use std::path::{Path, PathBuf};

use vize_carton::cstr;

const PASSTHROUGH_EXTENSIONS: &[&str] = &["js", "jsx", "mjs", "cjs", "json"];
const TYPESCRIPT_EXTENSIONS: &[&str] = &["ts", "tsx", "d.ts", "js", "jsx"];
const PACKAGE_ENTRY_FIELDS: &[&str] = &["types", "typings", "main"];

pub(super) fn resolve_relative_passthrough_module(dir: &Path, specifier: &str) -> Option<PathBuf> {
    let base = dir.join(specifier);
    if let Some((stripped, extensions)) = explicit_typescript_substitution(&base, specifier) {
        if let Some(path) = first_existing_with_extensions(&stripped, extensions) {
            return Some(path);
        }
        if let Some(path) = first_existing_with_extensions(&base, TYPESCRIPT_EXTENSIONS) {
            return Some(path);
        }
        return first_existing_directory_module(&base, TYPESCRIPT_EXTENSIONS);
    }

    if specifier_has_passthrough_extension(specifier) && base.is_file() {
        return Some(normalize_existing_path(&base));
    }
    first_existing_with_extensions(&base, PASSTHROUGH_EXTENSIONS)
        .or_else(|| first_existing_directory_module(&base, PASSTHROUGH_EXTENSIONS))
}

fn explicit_typescript_substitution<'a>(
    base: &Path,
    specifier: &str,
) -> Option<(PathBuf, &'a [&'a str])> {
    let (suffix, extensions): (&str, &[&str]) = if specifier.ends_with(".vue.tsx") {
        (".tsx", &["tsx", "ts", "d.ts", "jsx", "js"])
    } else if specifier.ends_with(".vue.ts") {
        (".ts", &["ts", "tsx", "d.ts", "js", "jsx"])
    } else {
        return None;
    };
    let name = base.file_name()?.to_str()?;
    Some((base.with_file_name(name.strip_suffix(suffix)?), extensions))
}

fn first_existing_with_extensions(base: &Path, extensions: &[&str]) -> Option<PathBuf> {
    extensions
        .iter()
        .map(|extension| append_extension(base, extension))
        .find(|candidate| candidate.is_file())
        .map(|candidate| normalize_existing_path(&candidate))
}

fn first_existing_index(base: &Path, extensions: &[&str]) -> Option<PathBuf> {
    extensions
        .iter()
        .map(|extension| base.join(cstr!("index.{extension}").as_str()))
        .find(|candidate| candidate.is_file())
        .map(|candidate| normalize_existing_path(&candidate))
}

fn first_existing_directory_module(base: &Path, extensions: &[&str]) -> Option<PathBuf> {
    package_entry(base, extensions).or_else(|| first_existing_index(base, extensions))
}

fn package_entry(base: &Path, extensions: &[&str]) -> Option<PathBuf> {
    let manifest = std::fs::read_to_string(base.join("package.json")).ok()?;
    let package: serde_json::Value = serde_json::from_str(&manifest).ok()?;
    for field in PACKAGE_ENTRY_FIELDS {
        let Some(relative) = package.get(field).and_then(serde_json::Value::as_str) else {
            continue;
        };
        let target = base.join(relative);
        if let Some(resolved) = resolve_package_target(&target, extensions) {
            return Some(resolved);
        }
    }
    None
}

fn resolve_package_target(target: &Path, extensions: &[&str]) -> Option<PathBuf> {
    let substituted = target
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| {
            [".js", ".jsx"]
                .into_iter()
                .find_map(|suffix| name.strip_suffix(suffix))
        })
        .map(|name| target.with_file_name(name))
        .and_then(|base| first_existing_with_extensions(&base, extensions));
    substituted
        .or_else(|| target.is_file().then(|| normalize_existing_path(target)))
        .or_else(|| first_existing_with_extensions(target, extensions))
        .or_else(|| first_existing_index(target, extensions))
}

fn append_extension(base: &Path, extension: &str) -> PathBuf {
    match base.file_name().and_then(|name| name.to_str()) {
        Some(name) => base.with_file_name(cstr!("{name}.{extension}")),
        None => base.to_path_buf(),
    }
}

fn specifier_has_passthrough_extension(specifier: &str) -> bool {
    PASSTHROUGH_EXTENSIONS
        .iter()
        .any(|extension| specifier.ends_with(cstr!(".{extension}").as_str()))
}

fn normalize_existing_path(path: &Path) -> PathBuf {
    vize_carton::path::canonicalize_non_verbatim(path)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::resolve_relative_passthrough_module;

    #[test]
    fn explicit_vue_ts_uses_typescript_candidate_order() {
        let case = tempfile::tempdir().expect("temp project");
        let root = &std::fs::canonicalize(case.path()).unwrap();
        for path in [
            "Direct.vue.ts",
            "Direct.vue.tsx",
            "Direct.vue.d.ts",
            "Direct.vue.js",
            "Direct.vue.jsx",
            "Full.vue.ts.d.ts",
            "Directory.vue.ts/index.ts",
            "Packaged.vue.ts/types.d.ts",
            "Main.vue.ts/entry.d.ts",
            "Main.vue.ts/entry.js",
        ] {
            let path = root.join(path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, "export {}\n").unwrap();
        }
        std::fs::write(
            root.join("Packaged.vue.ts/package.json"),
            r#"{"types":"./types.d.ts"}"#,
        )
        .unwrap();
        std::fs::write(
            root.join("Main.vue.ts/package.json"),
            r#"{"main":"./entry.js"}"#,
        )
        .unwrap();

        let resolve = |specifier| {
            resolve_relative_passthrough_module(root, specifier)
                .unwrap()
                .strip_prefix(root)
                .unwrap()
                .to_path_buf()
        };
        assert_eq!(resolve("./Direct.vue.ts"), Path::new("Direct.vue.ts"));
        assert_eq!(resolve("./Full.vue.ts"), Path::new("Full.vue.ts.d.ts"));
        assert_eq!(
            resolve("./Directory.vue.ts"),
            Path::new("Directory.vue.ts/index.ts")
        );
        assert_eq!(
            resolve("./Packaged.vue.ts"),
            Path::new("Packaged.vue.ts/types.d.ts")
        );
        assert_eq!(
            resolve("./Main.vue.ts"),
            Path::new("Main.vue.ts/entry.d.ts")
        );
    }

    #[test]
    fn explicit_vue_tsx_prefers_tsx_then_ts() {
        let case = tempfile::tempdir().expect("temp project");
        let root = &std::fs::canonicalize(case.path()).unwrap();
        std::fs::write(root.join("Component.vue.ts"), "export {}\n").unwrap();
        std::fs::write(root.join("Component.vue.tsx"), "export {}\n").unwrap();

        let resolved = resolve_relative_passthrough_module(root, "./Component.vue.tsx").unwrap();
        assert!(resolved.ends_with("Component.vue.tsx"));
    }
}
