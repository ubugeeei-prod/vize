use std::path::{Path, PathBuf};

use oxc_allocator::Allocator;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{FxHashSet, String, cstr};

use super::ModuleSpecifierCollector;

const SOURCE_EXTENSIONS: &[&str] = &[
    ".ts", ".tsx", ".d.ts", ".d.mts", ".d.cts", ".vue", ".mts", ".cts", ".js", ".jsx", ".mjs",
    ".cjs",
];

pub(super) fn absolute_import_needs_virtual_rewrite(path: &Path) -> bool {
    let Some(source_path) = resolve_source_path(path) else {
        return false;
    };
    source_needs_virtual_rewrite(&source_path)
}

fn source_needs_virtual_rewrite(path: &Path) -> bool {
    let mut visited: FxHashSet<PathBuf> = FxHashSet::default();
    let mut queue = vec![path.to_path_buf()];

    while let Some(file) = queue.pop() {
        if !visited.insert(file.clone()) {
            continue;
        }
        if file.extension().and_then(|extension| extension.to_str()) == Some("vue") {
            return true;
        }

        let Ok(source) = std::fs::read_to_string(&file) else {
            continue;
        };
        let Some(dir) = file.parent() else {
            continue;
        };

        for specifier in collect_import_specifiers(&source) {
            let candidate = Path::new(specifier.as_str());
            let resolved = if is_relative_specifier(&specifier) {
                resolve_source_path(&dir.join(specifier.as_str()))
            } else if candidate.is_absolute() {
                resolve_source_path(candidate)
            } else {
                None
            };
            let Some(resolved) = resolved else {
                continue;
            };
            if is_node_modules_path(&resolved) {
                continue;
            }
            if resolved
                .extension()
                .and_then(|extension| extension.to_str())
                == Some("vue")
            {
                return true;
            }
            if !visited.contains(&resolved) {
                queue.push(resolved);
            }
        }
    }

    false
}

fn resolve_source_path(path: &Path) -> Option<PathBuf> {
    if path.is_file() {
        return Some(path.to_path_buf());
    }

    for ext in SOURCE_EXTENSIONS {
        let candidate = append_extension(path, ext);
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    for ext in SOURCE_EXTENSIONS {
        let candidate = path.join(cstr!("index{ext}").as_str());
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    None
}

pub(super) fn append_extension(path: &Path, extension: &str) -> PathBuf {
    match path.file_name().and_then(|name| name.to_str()) {
        Some(name) => path.with_file_name(cstr!("{name}{extension}")),
        None => path.to_path_buf(),
    }
}

pub(super) fn is_rewritable_project_specifier(path: &Path) -> bool {
    if path
        .components()
        .next()
        .is_some_and(|component| component.as_os_str() == "node_modules")
    {
        return false;
    }
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_none_or(|extension| {
            matches!(
                extension,
                "vue" | "ts" | "tsx" | "mts" | "cts" | "js" | "jsx" | "mjs" | "cjs"
            )
        })
}

/// Redirect a relative extensionless specifier whose target is a `.vue` file on
/// disk onto that file's mirror module (`./components/svg` -> `./components/svg.vue.ts`).
///
/// Webpack-era apps spell SFC imports without the extension. TypeScript appends
/// extensions to the specifier and never tries `.vue`, so neither the mirror
/// (which holds `svg.vue.ts`) nor the real tree (which holds `svg.vue`) resolves
/// it: the import silently types as `any` and every prop/event contract at the
/// usage site disappears (#3329). The alias-mapped spelling is served by a
/// trailing `paths` candidate instead (#3300), which `paths` cannot do for
/// relative specifiers.
///
/// Like that alias candidate, this only ever turns a failing resolution into a
/// successful one: a specifier TypeScript already resolves through an ordinary
/// source sibling or directory module keeps its original spelling.
pub(crate) fn rewrite_relative_vue_specifier(specifier: &str, source_dir: &Path) -> Option<String> {
    // `.`/`..` name a directory module, not a stem an extension can be appended
    // to, so only the `./`-prefixed spellings are candidates here.
    if !(specifier.starts_with("./") || specifier.starts_with("../")) {
        return None;
    }
    if specifier_has_extension(specifier) {
        return None;
    }
    let target = source_dir.join(specifier);
    if !append_extension(&target, ".vue").is_file() || resolves_without_vue(&target) {
        return None;
    }
    Some(cstr!("{specifier}.vue.ts"))
}

fn specifier_has_extension(specifier: &str) -> bool {
    Path::new(specifier)
        .extension()
        .is_some_and(|extension| !extension.is_empty())
}

/// Whether TypeScript already resolves `target` without consulting `.vue`,
/// through an ordinary source extension or an `index` directory module.
fn resolves_without_vue(target: &Path) -> bool {
    SOURCE_EXTENSIONS
        .iter()
        .filter(|extension| **extension != ".vue")
        .any(|extension| {
            append_extension(target, extension).is_file()
                || target.join(cstr!("index{extension}").as_str()).is_file()
        })
}

pub(super) fn is_rewritable_vue_specifier(path: &str) -> bool {
    path.ends_with(".vue")
        && (path.starts_with("./")
            || path.starts_with("../")
            || path.starts_with("@/")
            || path.starts_with("~/")
            || Path::new(path).is_absolute())
}

fn is_relative_specifier(specifier: &str) -> bool {
    matches!(specifier, "." | "..") || specifier.starts_with("./") || specifier.starts_with("../")
}

fn is_node_modules_path(path: &Path) -> bool {
    path.components()
        .any(|component| component.as_os_str() == std::ffi::OsStr::new("node_modules"))
}

fn collect_import_specifiers(source: &str) -> Vec<String> {
    let allocator = Allocator::default();
    let parser = Parser::new(&allocator, source, SourceType::tsx());
    let result = parser.parse();
    let mut collector = ModuleSpecifierCollector::new();
    collector.visit_program(&result.program);
    collector
        .specifiers
        .into_iter()
        .map(|(_, _, path)| path)
        .collect()
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use vize_carton::cstr;

    use super::{
        absolute_import_needs_virtual_rewrite, collect_import_specifiers,
        rewrite_relative_vue_specifier,
    };

    fn unique_case_dir(name: &str) -> PathBuf {
        static NEXT_CASE_ID: std::sync::atomic::AtomicUsize =
            std::sync::atomic::AtomicUsize::new(0);
        let case_id = NEXT_CASE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        std::env::temp_dir().join(
            cstr!(
                "vize-import-rewriter-virtual-helper-{name}-{}-{case_id}",
                std::process::id()
            )
            .as_str(),
        )
    }

    fn write(dir: &Path, rel: &str, contents: &str) -> PathBuf {
        let path = dir.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&path, contents).unwrap();
        path
    }

    #[test]
    fn absolute_index_module_declaration_detects_vue_import() {
        let root = unique_case_dir("index-dcts");
        let _ = fs::remove_dir_all(&root);
        write(
            &root,
            "src/feature/index.d.cts",
            "export type Feature = typeof import('../Widget.vue')\n",
        );
        write(&root, "src/Widget.vue", "<template />");

        assert_eq!(
            collect_import_specifiers("export type Feature = typeof import('../Widget.vue')\n"),
            vec!["../Widget.vue".to_string()]
        );
        assert!(absolute_import_needs_virtual_rewrite(
            &root.join("src/feature")
        ));

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn extensionless_relative_sfc_specifiers_target_the_mirror_module() {
        let root = unique_case_dir("relative-vue");
        let _ = fs::remove_dir_all(&root);
        write(&root, "src/components/common/svg.vue", "<template />");
        write(&root, "src/components/header/head.vue", "<template />");
        write(&root, "src/page/home/home.vue", "<template />");
        let src = root.join("src");
        let home = root.join("src/page/home");

        assert_eq!(
            rewrite_relative_vue_specifier("./components/common/svg", &src).as_deref(),
            Some("./components/common/svg.vue.ts")
        );
        assert_eq!(
            rewrite_relative_vue_specifier("../../components/header/head", &home).as_deref(),
            Some("../../components/header/head.vue.ts")
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn specifiers_typescript_already_resolves_keep_their_spelling() {
        let root = unique_case_dir("relative-vue-precedence");
        let _ = fs::remove_dir_all(&root);
        write(&root, "src/Shadowed.vue", "<template />");
        write(&root, "src/Shadowed.ts", "export default 1;\n");
        write(&root, "src/Sibling.vue", "<template />");
        write(&root, "src/Sibling/index.ts", "export default 1;\n");
        write(&root, "src/Plain.vue", "<template />");
        let src = root.join("src");

        // Only a failing resolution may be redirected: an ordinary sibling
        // module and a directory `index` module both keep winning.
        assert_eq!(rewrite_relative_vue_specifier("./Shadowed", &src), None);
        assert_eq!(rewrite_relative_vue_specifier("./Sibling", &src), None);
        // Already-extensioned, bare, and absent specifiers are left alone.
        assert_eq!(rewrite_relative_vue_specifier("./Plain.vue", &src), None);
        assert_eq!(rewrite_relative_vue_specifier("./Plain.ts", &src), None);
        assert_eq!(rewrite_relative_vue_specifier("Plain", &src), None);
        assert_eq!(rewrite_relative_vue_specifier(".", &src), None);
        assert_eq!(rewrite_relative_vue_specifier("./Absent", &src), None);

        let _ = fs::remove_dir_all(&root);
    }
}
