//! Import rewriter for transforming .vue imports to .vue.ts.

use oxc_allocator::Allocator;
use oxc_ast::ast::{Expression, Statement};
use oxc_ast_visit::Visit;
use oxc_ast_visit::walk;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::String;
use vize_carton::ToCompactString;
use vize_carton::cstr;

#[derive(Debug, Clone)]
pub struct OffsetAdjustment {
    pub original_offset: u32,
    pub adjustment: i32,
}

#[derive(Debug)]
pub struct RewriteResult {
    pub code: String,
    pub source_map: ImportSourceMap,
}

#[derive(Debug, Default)]
pub struct ImportSourceMap {
    adjustments: Vec<OffsetAdjustment>,
}

impl ImportSourceMap {
    pub fn new(adjustments: Vec<OffsetAdjustment>) -> Self {
        Self { adjustments }
    }

    pub fn empty() -> Self {
        Self::default()
    }

    pub fn get_original_offset(&self, virtual_offset: u32) -> u32 {
        let mut cumulative: i32 = 0;
        for adj in &self.adjustments {
            let adjusted = (adj.original_offset as i32 + cumulative) as u32;
            if virtual_offset < adjusted {
                break;
            }
            cumulative += adj.adjustment;
        }
        (virtual_offset as i32 - cumulative) as u32
    }

    pub fn get_virtual_offset(&self, original_offset: u32) -> u32 {
        let mut cumulative: i32 = 0;
        for adj in &self.adjustments {
            if original_offset < adj.original_offset {
                break;
            }
            cumulative += adj.adjustment;
        }
        (original_offset as i32 + cumulative) as u32
    }
}

pub struct ImportRewriter;

impl ImportRewriter {
    pub fn new() -> Self {
        Self
    }

    pub fn rewrite(&self, source: &str, source_type: SourceType) -> RewriteResult {
        if !source.contains(".vue") {
            return RewriteResult {
                code: source.to_compact_string(),
                source_map: ImportSourceMap::empty(),
            };
        }

        self.rewrite_with(source, source_type, |path| {
            self.rewrite_module_specifier(path)
        })
    }

    pub fn rewrite_for_virtual_project(
        &self,
        source: &str,
        source_type: SourceType,
        roots: (&std::path::Path, &std::path::Path),
    ) -> RewriteResult {
        let project_root = roots.0.to_string_lossy();
        if !source.contains(".vue") && !source.contains(project_root.as_ref()) {
            return RewriteResult {
                code: source.to_compact_string(),
                source_map: ImportSourceMap::empty(),
            };
        }

        self.rewrite_with(source, source_type, |path| {
            self.rewrite_virtual_project_specifier(path, roots)
        })
    }

    pub fn rewrite_declaration_specifiers(
        &self,
        source: &str,
        source_type: SourceType,
    ) -> RewriteResult {
        if !source.contains(".vue.ts") {
            return RewriteResult {
                code: source.to_compact_string(),
                source_map: ImportSourceMap::empty(),
            };
        }

        self.rewrite_with(source, source_type, |path| {
            self.rewrite_declaration_specifier(path)
        })
    }

    fn rewrite_with<F>(
        &self,
        source: &str,
        source_type: SourceType,
        rewrite_specifier: F,
    ) -> RewriteResult
    where
        F: Fn(&str) -> Option<String>,
    {
        let allocator = Allocator::default();
        let parser = Parser::new(&allocator, source, source_type);
        let result = parser.parse();

        let mut rewrites: Vec<(u32, u32, String)> = Vec::new();

        for stmt in &result.program.body {
            match stmt {
                Statement::ImportDeclaration(decl) => {
                    if let Some(rewrite) = rewrite_specifier(&decl.source.value) {
                        rewrites.push((
                            decl.source.span.start + 1, // +1 to skip opening quote
                            decl.source.span.end - 1,   // -1 to skip closing quote
                            rewrite,
                        ));
                    }
                }
                Statement::ExportNamedDeclaration(decl) => {
                    if let Some(source) = &decl.source
                        && let Some(rewrite) = rewrite_specifier(&source.value)
                    {
                        rewrites.push((source.span.start + 1, source.span.end - 1, rewrite));
                    }
                }
                Statement::ExportAllDeclaration(decl) => {
                    if let Some(rewrite) = rewrite_specifier(&decl.source.value) {
                        rewrites.push((
                            decl.source.span.start + 1,
                            decl.source.span.end - 1,
                            rewrite,
                        ));
                    }
                }
                _ => {}
            }
        }

        let mut collector = DynamicImportCollector::new();
        collector.visit_program(&result.program);
        for (start, end, path) in collector.imports {
            if let Some(rewrite) = rewrite_specifier(&path) {
                rewrites.push((start, end, rewrite));
            }
        }

        rewrites.sort_by_key(|rewrite| std::cmp::Reverse(rewrite.0));

        let mut output = source.to_compact_string();
        let mut adjustments = Vec::new();

        for (start, end, new_path) in rewrites {
            let original_len = (end - start) as i32;
            let new_len = new_path.len() as i32;

            output.replace_range(start as usize..end as usize, new_path.as_str());

            adjustments.push(OffsetAdjustment {
                original_offset: start,
                adjustment: new_len - original_len,
            });
        }

        adjustments.reverse();

        RewriteResult {
            code: output,
            source_map: ImportSourceMap::new(adjustments),
        }
    }

    pub fn collect_relative_vue_specifiers(
        &self,
        source: &str,
        source_type: SourceType,
    ) -> Vec<String> {
        if !source.contains(".vue") {
            return Vec::new();
        }

        let allocator = Allocator::default();
        let parser = Parser::new(&allocator, source, source_type);
        let result = parser.parse();

        let mut specifiers: Vec<String> = Vec::new();
        let mut push = |path: &str| {
            if path.ends_with(".vue") && (path.starts_with("./") || path.starts_with("../")) {
                let candidate = path.to_compact_string();
                if !specifiers.iter().any(|s| s.as_str() == candidate.as_str()) {
                    specifiers.push(candidate);
                }
            }
        };

        for stmt in &result.program.body {
            match stmt {
                Statement::ImportDeclaration(decl) => push(&decl.source.value),
                Statement::ExportNamedDeclaration(decl) => {
                    if let Some(source) = &decl.source {
                        push(&source.value);
                    }
                }
                Statement::ExportAllDeclaration(decl) => push(&decl.source.value),
                _ => {}
            }
        }

        let mut collector = DynamicImportCollector::new();
        collector.visit_program(&result.program);
        for (_, _, path) in collector.imports {
            push(&path);
        }

        specifiers
    }

    fn rewrite_module_specifier(&self, path: &str) -> Option<String> {
        if path.ends_with(".vue") {
            Some(cstr!("{path}.ts"))
        } else {
            None
        }
    }

    fn rewrite_virtual_project_specifier(
        &self,
        path: &str,
        roots: (&std::path::Path, &std::path::Path),
    ) -> Option<String> {
        let candidate = std::path::Path::new(path);
        if candidate.is_absolute()
            && let Ok(relative) = candidate.strip_prefix(roots.0)
            && is_rewritable_project_specifier(relative)
        {
            let mut rewritten = cstr!("{}", roots.1.join(relative).display());
            if path.ends_with(".vue") {
                rewritten.push_str(".ts");
            }
            return Some(rewritten);
        }
        if path.ends_with(".vue") {
            Some(cstr!("{path}.ts"))
        } else {
            None
        }
    }

    fn rewrite_declaration_specifier(&self, path: &str) -> Option<String> {
        if path.ends_with(".vue.ts") {
            return path
                .strip_suffix(".ts")
                .map(|value| value.to_compact_string());
        }
        None
    }
}

fn is_rewritable_project_specifier(path: &std::path::Path) -> bool {
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

impl Default for ImportRewriter {
    fn default() -> Self {
        Self::new()
    }
}

struct DynamicImportCollector {
    imports: Vec<(u32, u32, String)>,
}

impl DynamicImportCollector {
    fn new() -> Self {
        Self {
            imports: Vec::new(),
        }
    }
}

impl<'a> Visit<'a> for DynamicImportCollector {
    fn visit_import_expression(&mut self, expr: &oxc_ast::ast::ImportExpression<'a>) {
        if let Expression::StringLiteral(lit) = &expr.source {
            self.imports.push((
                lit.span.start + 1,
                lit.span.end - 1,
                lit.value.as_str().into(),
            ));
        }
        walk::walk_import_expression(self, expr);
    }
}
