use std::path::{Path, PathBuf};

use oxc_allocator::Allocator;
use oxc_ast::ast::{ExportNamedDeclaration, ImportDeclarationSpecifier, Statement};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_carton::{FxHashSet, String, ToCompactString};

use crate::parse_sfc;
use crate::types::SfcParseOptions;

use super::ScriptCompileContext;
use super::helpers::is_import_type_only;

const RESOLVE_EXTENSIONS: &[&str] = &[
    ".ts", ".tsx", ".d.ts", ".mts", ".cts", ".js", ".jsx", ".vue",
];
const INDEX_CANDIDATES: &[&str] = &[
    "index.ts",
    "index.tsx",
    "index.d.ts",
    "index.mts",
    "index.cts",
    "index.js",
    "index.jsx",
    "index.vue",
];

impl ScriptCompileContext {
    pub fn collect_imported_types_from_path(&mut self, source: &str, filename: &str) {
        if !source.contains("type") || (!source.contains("import") && !source.contains("export")) {
            return;
        }

        let owned_base = canonicalize_or_original(PathBuf::from(filename))
            .unwrap_or_else(|| PathBuf::from(filename));
        let base_file = owned_base.as_path();
        let Some(base_dir) = base_file.parent() else {
            return;
        };
        if base_dir.as_os_str().is_empty() {
            return;
        }

        let mut visited = FxHashSet::default();
        self.collect_imported_types_recursive(source, base_file, &mut visited);
    }

    fn collect_imported_types_recursive(
        &mut self,
        source: &str,
        current_file: &Path,
        visited: &mut FxHashSet<String>,
    ) {
        let allocator = Allocator::default();
        let source_type = SourceType::from_path("script.ts").unwrap_or_default();
        let ret = Parser::new(&allocator, source, source_type).parse();
        if ret.panicked {
            return;
        }

        for stmt in ret.program.body.iter() {
            match stmt {
                Statement::ImportDeclaration(import_decl) => {
                    if !import_decl.import_kind.is_type()
                        && !is_import_type_only(import_decl, source)
                        && !import_decl.specifiers.as_ref().is_some_and(|specifiers| {
                            specifiers.iter().any(|specifier| match specifier {
                                ImportDeclarationSpecifier::ImportSpecifier(spec) => {
                                    spec.import_kind.is_type()
                                }
                                _ => false,
                            })
                        })
                    {
                        continue;
                    }

                    self.collect_types_from_specifier(
                        import_decl.source.value.as_str(),
                        current_file,
                        visited,
                    );
                }
                Statement::ExportNamedDeclaration(export_decl) => {
                    let Some(ref export_source) = export_decl.source else {
                        continue;
                    };
                    if !is_type_re_export(export_decl, source) {
                        continue;
                    }

                    self.collect_types_from_specifier(
                        export_source.value.as_str(),
                        current_file,
                        visited,
                    );
                }
                Statement::ExportAllDeclaration(export_decl) => {
                    if !is_export_all_type_only(stmt, source) {
                        continue;
                    }

                    self.collect_types_from_specifier(
                        export_decl.source.value.as_str(),
                        current_file,
                        visited,
                    );
                }
                _ => {}
            }
        }
    }

    fn collect_types_from_specifier(
        &mut self,
        specifier: &str,
        current_file: &Path,
        visited: &mut FxHashSet<String>,
    ) {
        let Some(resolved_path) = resolve_import_path(current_file, specifier) else {
            return;
        };

        let key = path_key(&resolved_path);
        if !visited.insert(key) {
            return;
        }

        let Ok(content) = std::fs::read_to_string(&resolved_path) else {
            return;
        };

        if resolved_path.extension().is_some_and(|ext| ext == "vue") {
            self.collect_types_from_vue_file(&resolved_path, &content, visited);
            return;
        }

        self.collect_types_from(&content);
        self.collect_imported_types_recursive(&content, &resolved_path, visited);
    }

    fn collect_types_from_vue_file(
        &mut self,
        path: &Path,
        content: &str,
        visited: &mut FxHashSet<String>,
    ) {
        let Ok(descriptor) = parse_sfc(content, SfcParseOptions::default()) else {
            return;
        };

        if let Some(ref script) = descriptor.script {
            self.collect_types_from(&script.content);
            self.collect_imported_types_recursive(&script.content, path, visited);
        }

        if let Some(ref script_setup) = descriptor.script_setup {
            self.collect_types_from(&script_setup.content);
            self.collect_imported_types_recursive(&script_setup.content, path, visited);
        }
    }
}

fn resolve_import_path(current_file: &Path, specifier: &str) -> Option<PathBuf> {
    if let Some(alias_path) = resolve_at_src_alias(current_file, specifier) {
        return Some(alias_path);
    }

    if !specifier.starts_with('.') && !specifier.starts_with('/') {
        return None;
    }

    let base_dir = current_file.parent()?;
    let candidate = if specifier.starts_with('/') {
        PathBuf::from(specifier)
    } else {
        base_dir.join(specifier)
    };

    resolve_candidate_path(candidate)
}

fn resolve_at_src_alias(current_file: &Path, specifier: &str) -> Option<PathBuf> {
    let rest = specifier.strip_prefix("@/")?;
    let src_dir = current_file
        .parent()?
        .ancestors()
        .find(|path| path.file_name().is_some_and(|name| name == "src"))?;

    resolve_candidate_path(src_dir.join(rest))
}

fn resolve_candidate_path(candidate: PathBuf) -> Option<PathBuf> {
    if candidate.is_file() {
        return canonicalize_or_original(candidate);
    }

    for ext in RESOLVE_EXTENSIONS {
        let mut with_ext = candidate.clone().into_os_string();
        with_ext.push(ext);
        let path = PathBuf::from(with_ext);
        if path.is_file() {
            return canonicalize_or_original(path);
        }
    }

    if candidate.is_dir() {
        for index_name in INDEX_CANDIDATES {
            let path = candidate.join(index_name);
            if path.is_file() {
                return canonicalize_or_original(path);
            }
        }
    }

    None
}

fn canonicalize_or_original(path: PathBuf) -> Option<PathBuf> {
    match path.canonicalize() {
        Ok(canonical) => Some(canonical),
        Err(_) if path.exists() => Some(path),
        Err(_) => None,
    }
}

fn is_type_re_export(export_decl: &ExportNamedDeclaration<'_>, source: &str) -> bool {
    if export_decl.export_kind.is_type() {
        return true;
    }

    if export_decl
        .specifiers
        .iter()
        .any(|specifier| specifier.export_kind.is_type())
    {
        return true;
    }

    let span = export_decl.span;
    let start = span.start as usize;
    let end = span.end as usize;
    if start >= end || end > source.len() {
        return false;
    }

    let raw = source[start..end].trim_start();
    raw.starts_with("export type ")
        || raw.contains("{ type ")
        || raw.contains("{type ")
        || raw.contains(", type ")
}

fn is_export_all_type_only(stmt: &Statement<'_>, source: &str) -> bool {
    let span = stmt.span();
    let start = span.start as usize;
    let end = span.end as usize;
    start < end
        && end <= source.len()
        && source[start..end].trim_start().starts_with("export type ")
}

fn path_key(path: &Path) -> String {
    path.to_string_lossy().as_ref().to_compact_string()
}

#[cfg(test)]
mod tests {
    use super::{is_type_re_export, resolve_at_src_alias, resolve_import_path};
    use oxc_allocator::Allocator;
    use oxc_ast::ast::Statement;
    use oxc_parser::Parser;
    use oxc_span::SourceType;
    use std::path::{Path, PathBuf};

    fn temp_project_dir(test_name: &str) -> PathBuf {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "vize-sfc-external-types-{}-{}-{}",
            std::process::id(),
            test_name,
            nonce
        ))
    }

    #[test]
    fn resolves_at_alias_from_nearest_src_directory() {
        let project = temp_project_dir("at-alias");
        let components = project.join("packages/frontend/src/components");
        std::fs::create_dir_all(&components).unwrap();
        let target = components.join("Base.vue");
        std::fs::write(&target, "").unwrap();

        let current = components.join("Child.vue");
        let resolved = resolve_at_src_alias(&current, "@/components/Base.vue");
        let target = target.canonicalize().unwrap();

        assert_eq!(resolved.as_deref(), Some(target.as_path()));

        let _ = std::fs::remove_dir_all(project);
    }

    #[test]
    fn ignores_at_alias_without_src_ancestor() {
        let current = Path::new("/repo/packages/frontend/components/Child.vue");

        assert!(resolve_at_src_alias(current, "@/components/Base.vue").is_none());
    }

    #[test]
    fn leaves_non_at_alias_specifiers_to_existing_resolution() {
        let current = Path::new("/repo/src/components/Child.vue");

        assert!(resolve_import_path(current, "vue").is_none());
    }

    #[test]
    fn collects_type_reexports_from_vue_files() {
        let project = temp_project_dir("vue-type-reexport");
        let components = project.join("src/components");
        std::fs::create_dir_all(&components).unwrap();
        std::fs::write(
            components.join("Base.vue"),
            r#"<script lang="ts">
export interface BaseProps {
  as?: string;
  asChild?: boolean;
}
</script>"#,
        )
        .unwrap();
        std::fs::write(
            components.join("index.ts"),
            r#"export { type BaseProps } from "./Base.vue";"#,
        )
        .unwrap();

        let parent = components.join("Parent.vue");
        let source = r#"
import type { BaseProps } from "./index";

interface ParentProps extends BaseProps {}

const props = defineProps<ParentProps>();
"#;

        let mut ctx = super::ScriptCompileContext::new(source);
        ctx.collect_imported_types_from_path(source, parent.to_string_lossy().as_ref());
        ctx.analyze();

        assert!(ctx.interfaces.contains_key("BaseProps"));
        assert_eq!(
            ctx.bindings.bindings.get("as"),
            Some(&crate::types::BindingType::Props)
        );
        assert_eq!(
            ctx.bindings.bindings.get("asChild"),
            Some(&crate::types::BindingType::Props)
        );

        let _ = std::fs::remove_dir_all(project);
    }

    #[test]
    fn detects_multiline_mixed_type_reexports() {
        let source = r#"export {
  default as Content,
  type ContentProps,
} from "./Content.vue";
"#;
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
        let Statement::ExportNamedDeclaration(export_decl) = &parsed.program.body[0] else {
            panic!("expected export declaration");
        };

        assert!(is_type_re_export(export_decl, source));
    }

    #[test]
    fn collects_mixed_type_reexports_from_vue_files() {
        let project = temp_project_dir("mixed-vue-type-reexport");
        let components = project.join("src/components");
        std::fs::create_dir_all(&components).unwrap();
        std::fs::write(
            components.join("Content.vue"),
            r#"<script lang="ts">
export interface ContentProps {
  as?: string;
  asChild?: boolean;
}
</script>"#,
        )
        .unwrap();
        std::fs::write(
            components.join("index.ts"),
            r#"export {
  default as Content,
  type ContentProps,
} from "./Content.vue";
"#,
        )
        .unwrap();

        let parent = components.join("Parent.vue");
        let source = r#"
import type { ContentProps } from "./index";

interface ParentProps extends ContentProps {}

const props = defineProps<ParentProps>();
"#;

        let mut ctx = super::ScriptCompileContext::new(source);
        ctx.collect_imported_types_from_path(source, parent.to_string_lossy().as_ref());
        ctx.analyze();

        assert!(ctx.interfaces.contains_key("ContentProps"));
        assert_eq!(
            ctx.bindings.bindings.get("as"),
            Some(&crate::types::BindingType::Props)
        );
        assert_eq!(
            ctx.bindings.bindings.get("asChild"),
            Some(&crate::types::BindingType::Props)
        );

        let _ = std::fs::remove_dir_all(project);
    }
}
