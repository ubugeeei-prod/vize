//! Discovering relative non-TS modules imported by a source file so they can be
//! mirrored into the virtual project, keeping TypeScript's module resolution
//! happy. Includes a lightweight byte-level scanner for `import`/`from`
//! specifiers.

use std::path::{Path, PathBuf};

use vize_carton::{FxHashSet, String as CompactString, ToCompactString, cstr};

use super::{VirtualProject, build::mirrored_virtual_path};

impl VirtualProject {
    pub(super) fn javascript_passthrough_files(&self) -> impl Iterator<Item = &Path> {
        self.passthrough_files
            .keys()
            .filter(|path| is_javascript_passthrough(path))
            .map(PathBuf::as_path)
    }
}

pub(super) fn collect_passthrough_modules(
    path: &Path,
    content: &str,
    project_root: &Path,
    virtual_root: &Path,
) -> Vec<(PathBuf, PathBuf)> {
    let Some(dir) = path.parent() else {
        return Vec::new();
    };

    let mut seen = FxHashSet::default();
    let mut files = Vec::new();
    for specifier in extract_relative_module_specifiers(content) {
        let Some(original_path) = resolve_relative_passthrough_module(dir, &specifier) else {
            continue;
        };
        let Ok(virtual_path) = mirrored_virtual_path(project_root, virtual_root, &original_path)
        else {
            continue;
        };
        if seen.insert(virtual_path.clone()) {
            files.push((virtual_path, original_path));
        }
    }
    files
}

fn extract_relative_module_specifiers(source: &str) -> Vec<CompactString> {
    let bytes = source.as_bytes();
    let len = bytes.len();
    let mut specifiers = Vec::new();
    let mut i = 0;

    while i < len {
        let keyword_len = if matches_keyword(bytes, i, b"from") {
            4
        } else if matches_keyword(bytes, i, b"import") {
            6
        } else {
            i += 1;
            continue;
        };

        let mut j = i + keyword_len;
        while j < len && bytes[j].is_ascii_whitespace() {
            j += 1;
        }
        if j < len && bytes[j] == b'(' {
            j += 1;
            while j < len && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
        }

        if j < len && (bytes[j] == b'"' || bytes[j] == b'\'') {
            let quote = bytes[j];
            let start = j + 1;
            let mut k = start;
            while k < len && bytes[k] != quote {
                k += 1;
            }
            if k < len {
                let specifier = &source[start..k];
                if is_relative_specifier(specifier) {
                    specifiers.push(specifier.to_compact_string());
                }
                i = k + 1;
                continue;
            }
        }

        i += keyword_len;
    }

    specifiers
}

fn matches_keyword(bytes: &[u8], at: usize, keyword: &[u8]) -> bool {
    if at + keyword.len() > bytes.len() || &bytes[at..at + keyword.len()] != keyword {
        return false;
    }
    let before_ok = at == 0 || !is_identifier_byte(bytes[at - 1]);
    let after = at + keyword.len();
    let after_ok = after >= bytes.len() || !is_identifier_byte(bytes[after]);
    before_ok && after_ok
}

fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
}

fn is_relative_specifier(specifier: &str) -> bool {
    specifier.starts_with("./") || specifier.starts_with("../")
}

const PASSTHROUGH_EXTENSIONS: &[&str] = &["js", "jsx", "mjs", "cjs", "json"];

fn resolve_relative_passthrough_module(dir: &Path, specifier: &str) -> Option<PathBuf> {
    let base = dir.join(specifier);

    if specifier_has_passthrough_extension(specifier) && base.is_file() {
        return Some(normalize_existing_path(&base));
    }

    for extension in PASSTHROUGH_EXTENSIONS {
        let candidate = append_extension(&base, extension);
        if candidate.is_file() {
            return Some(normalize_existing_path(&candidate));
        }
    }

    for extension in PASSTHROUGH_EXTENSIONS {
        let candidate = base.join(cstr!("index.{extension}").as_str());
        if candidate.is_file() {
            return Some(normalize_existing_path(&candidate));
        }
    }

    None
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

fn is_javascript_passthrough(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| matches!(extension, "js" | "jsx" | "mjs" | "cjs"))
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::collect_passthrough_modules;
    use vize_carton::cstr;

    fn unique_case_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(cstr!("vize-passthrough-{name}-{}", std::process::id()).as_str())
    }

    fn write(root: &Path, rel: &str, content: &str) -> PathBuf {
        let path = root.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn collects_relative_js_and_json_modules() {
        let case_dir = unique_case_dir("js-json");
        let _ = std::fs::remove_dir_all(&case_dir);
        std::fs::create_dir_all(&case_dir).unwrap();
        let root = vize_carton::path::canonicalize_non_verbatim(&case_dir);
        let entry = write(
            &root,
            "lint/__tests__/rule.spec.ts",
            "import config from '../../.eslintrc.js'\nimport rule from '../rules/no-access-process'\nimport colors from '../../tokens/colors.json'\n",
        );
        write(&root, ".eslintrc.js", "export default {}\n");
        write(
            &root,
            "lint/rules/no-access-process.js",
            "export default {}\n",
        );
        write(&root, "tokens/colors.json", "{}\n");
        let virtual_root = root.join("node_modules/.vize/canon");

        let mut files = collect_passthrough_modules(
            &entry,
            &std::fs::read_to_string(&entry).unwrap(),
            &root,
            &virtual_root,
        );
        files.sort();

        assert_eq!(
            files
                .into_iter()
                .map(|(virtual_path, original_path)| {
                    (
                        virtual_path
                            .strip_prefix(&virtual_root)
                            .unwrap()
                            .to_path_buf(),
                        original_path.strip_prefix(&root).unwrap().to_path_buf(),
                    )
                })
                .collect::<Vec<_>>(),
            vec![
                (PathBuf::from(".eslintrc.js"), PathBuf::from(".eslintrc.js")),
                (
                    PathBuf::from("lint/rules/no-access-process.js"),
                    PathBuf::from("lint/rules/no-access-process.js"),
                ),
                (
                    PathBuf::from("tokens/colors.json"),
                    PathBuf::from("tokens/colors.json"),
                ),
            ]
        );

        let _ = std::fs::remove_dir_all(&case_dir);
    }
}
