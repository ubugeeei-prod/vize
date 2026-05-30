//! Transitive resolution of relative source imports for explicit `vize check`
//! subsets.
//!
//! `vize check src/App.vue` registers only the requested files in the virtual
//! project, so a relative import like `import { Foo } from './types'` cannot see
//! the sibling's real types and degrades to `any` (or surfaces a false
//! `TS2307`). `tsc`/`vue-tsc` load the whole reachable program instead. This
//! module walks the relative-import graph from the requested files and returns
//! the additional on-disk source files to register, so cross-file types resolve
//! precisely — analogous to the ambient-`.d.ts` pull-in in the runner.

use std::path::{Path, PathBuf};

use oxc_allocator::Allocator;
use oxc_ast::ast::Statement;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{FxHashSet, String, ToCompactString, cstr};

/// Source extensions whose imports carry TypeScript types worth pulling into the
/// virtual project, in module-resolution precedence order.
const RESOLVE_EXTENSIONS: &[&str] = &[".ts", ".tsx", ".d.ts", ".vue", ".mts", ".cts"];

/// Walk the relative-import graph reachable from `roots` and return the extra
/// on-disk source files that should be registered alongside them. The roots
/// themselves are excluded from the result; every returned path is absolute.
pub(super) fn collect_transitive_local_imports(roots: &[PathBuf], cwd: &Path) -> Vec<PathBuf> {
    let mut visited: FxHashSet<PathBuf> = FxHashSet::default();
    let mut queue: Vec<PathBuf> = Vec::new();

    // Seed the visited set with the roots so they are never re-registered.
    for root in roots {
        if let Some(absolute) = absolutize(root, cwd)
            && visited.insert(absolute.clone())
        {
            queue.push(absolute);
        }
    }

    let mut discovered: Vec<PathBuf> = Vec::new();

    while let Some(file) = queue.pop() {
        let Some(dir) = file.parent() else {
            continue;
        };
        let Ok(source) = std::fs::read_to_string(&file) else {
            continue;
        };
        let script = scriptable_source(&file, &source);
        for specifier in extract_relative_specifiers(&script) {
            let Some(resolved) = resolve_relative_import(dir, &specifier) else {
                continue;
            };
            if visited.insert(resolved.clone()) {
                discovered.push(resolved.clone());
                queue.push(resolved);
            }
        }
    }

    discovered
}

/// Resolve `path` against `cwd` and canonicalize it so duplicate registrations
/// of the same file under different spellings collapse.
fn absolutize(path: &Path, cwd: &Path) -> Option<PathBuf> {
    let joined = if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    };
    Some(joined.canonicalize().unwrap_or(joined))
}

/// Extract the TypeScript-analyzable source from a file: the concatenated
/// `<script>` / `<script setup>` blocks of a `.vue` SFC, or the file verbatim.
fn scriptable_source(file: &Path, source: &str) -> String {
    let is_vue = file
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("vue"));
    if !is_vue {
        return source.to_compact_string();
    }

    let Ok(descriptor) = vize_atelier_sfc::parse_sfc(source, Default::default()) else {
        return String::default();
    };
    let mut combined = String::default();
    if let Some(script) = descriptor.script.as_ref() {
        combined.push_str(&script.content);
        combined.push('\n');
    }
    if let Some(script_setup) = descriptor.script_setup.as_ref() {
        combined.push_str(&script_setup.content);
    }
    combined
}

/// Parse `source` as TypeScript and collect the specifiers of its static
/// `import` / `export … from` declarations that are relative (`./`, `../`).
fn extract_relative_specifiers(source: &str) -> Vec<String> {
    let allocator = Allocator::default();
    let source_type = SourceType::default().with_typescript(true).with_jsx(true);
    let parsed = Parser::new(&allocator, source, source_type).parse();

    let mut specifiers = Vec::new();
    for statement in &parsed.program.body {
        let specifier = match statement {
            Statement::ImportDeclaration(decl) => Some(decl.source.value.as_str()),
            Statement::ExportNamedDeclaration(decl) => {
                decl.source.as_ref().map(|s| s.value.as_str())
            }
            Statement::ExportAllDeclaration(decl) => Some(decl.source.value.as_str()),
            _ => None,
        };
        if let Some(specifier) = specifier
            && is_relative_specifier(specifier)
        {
            specifiers.push(specifier.to_compact_string());
        }
    }
    specifiers
}

fn is_relative_specifier(specifier: &str) -> bool {
    specifier.starts_with("./") || specifier.starts_with("../")
}

/// Resolve a relative module specifier against `dir` to an existing on-disk
/// source file, mirroring TypeScript's extension and `index` probing (including
/// the `.js` → `.ts` rewrite used under bundler/Node-ESM resolution).
fn resolve_relative_import(dir: &Path, specifier: &str) -> Option<PathBuf> {
    let base = dir.join(specifier);

    // 1. The specifier already points at an existing source file.
    if has_source_extension(&base) && base.is_file() {
        return canonicalize(&base);
    }

    // 2. A `.js`/`.mjs`/`.cjs` specifier resolving to its `.ts`/`.tsx` sibling.
    if let Some(rewritten) = rewrite_js_to_ts(&base) {
        return Some(rewritten);
    }

    // 3. Append a source extension: `./types` → `./types.ts`.
    for ext in RESOLVE_EXTENSIONS {
        let candidate = append_extension(&base, ext);
        if candidate.is_file() {
            return canonicalize(&candidate);
        }
    }

    // 4. Directory index: `./feature` → `./feature/index.ts`.
    for ext in RESOLVE_EXTENSIONS {
        let candidate = base.join(cstr_index(ext));
        if candidate.is_file() {
            return canonicalize(&candidate);
        }
    }

    None
}

fn rewrite_js_to_ts(base: &Path) -> Option<PathBuf> {
    let name = base.file_name()?.to_str()?;
    let stem = name
        .strip_suffix(".js")
        .or_else(|| name.strip_suffix(".mjs"))
        .or_else(|| name.strip_suffix(".cjs"))?;
    for ext in [".ts", ".tsx", ".d.ts", ".mts", ".cts"] {
        let candidate = base.with_file_name(cstr!("{stem}{ext}"));
        if candidate.is_file() {
            return canonicalize(&candidate);
        }
    }
    None
}

fn has_source_extension(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    RESOLVE_EXTENSIONS
        .iter()
        .any(|ext| name.ends_with(ext) && name.len() > ext.len())
}

/// Append a full extension (e.g. `.d.ts`) to a path's file name without
/// replacing any existing one, so `./a.b` → `./a.b.ts`.
fn append_extension(base: &Path, ext: &str) -> PathBuf {
    match base.file_name().and_then(|name| name.to_str()) {
        Some(name) => base.with_file_name(cstr!("{name}{ext}")),
        None => base.to_path_buf(),
    }
}

fn cstr_index(ext: &str) -> String {
    cstr!("index{ext}")
}

fn canonicalize(path: &Path) -> Option<PathBuf> {
    Some(path.canonicalize().unwrap_or_else(|_| path.to_path_buf()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(dir: &Path, rel: &str, contents: &str) -> PathBuf {
        let path = dir.join(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&path, contents).unwrap();
        path
    }

    #[test]
    fn collects_relative_ts_and_vue_imports_transitively() {
        let root = std::env::temp_dir().join(cstr!("vize-imports-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("src")).unwrap();

        let app = write(
            &root,
            "src/App.vue",
            "<script setup lang=\"ts\">\nimport type { Sibling } from './types'\nimport Child from './Child.vue'\nconst x: Sibling = { a: 1 }\n</script>\n<template><Child /></template>\n",
        );
        let types = write(
            &root,
            "src/types.ts",
            "export interface Sibling { a: number }\n",
        );
        let child = write(
            &root,
            "src/Child.vue",
            "<script setup lang=\"ts\">\nimport { helper } from './nested/util'\n</script>\n<template><div /></template>\n",
        );
        let util = write(&root, "src/nested/util.ts", "export const helper = 1\n");

        let discovered = collect_transitive_local_imports(&[app.clone()], &root);

        let canon = |p: &Path| p.canonicalize().unwrap();
        assert!(discovered.contains(&canon(&types)));
        assert!(discovered.contains(&canon(&child)));
        assert!(discovered.contains(&canon(&util)));
        // The root itself is never re-registered.
        assert!(!discovered.contains(&canon(&app)));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn ignores_bare_and_missing_specifiers() {
        let root = std::env::temp_dir().join(cstr!("vize-imports-bare-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        let entry = write(
            &root,
            "entry.ts",
            "import { ref } from 'vue'\nimport { gone } from './missing'\nexport const a = ref(0)\nvoid gone\n",
        );

        let discovered = collect_transitive_local_imports(&[entry], &root);
        assert!(discovered.is_empty());

        let _ = std::fs::remove_dir_all(&root);
    }
}
