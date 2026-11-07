//! Rewriting of `.vue.ts` shim specifiers to their `.vue.tsx` sources before
//! declaration emit.

use std::path::Path;

use crate::batch::import_rewriter::{ImportSourceMap, RewriteResult};
use oxc_allocator::Allocator;
use oxc_ast::ast::{
    CallExpression, ExportAllDeclaration, ExportNamedDeclaration, Expression, ImportDeclaration,
    ImportExpression, StringLiteral, TSExternalModuleReference, TSImportType,
    TSModuleDeclarationName,
};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{String, ToCompactString, cstr};

pub(super) fn is_script_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| matches!(extension, "ts" | "tsx" | "mts" | "cts"))
}

pub(super) fn rewrite_tsx_vue_shim_specifiers(
    source: &str,
    source_type: SourceType,
    source_dir: &Path,
    serialize_setup_scoped_props: bool,
) -> Option<RewriteResult> {
    let mut rewrites = if source.contains(".vue.ts") {
        let allocator = Allocator::default();
        let parser = Parser::new(&allocator, source, source_type);
        let result = parser.parse();
        let mut collector = ModuleSpecifierCollector::default();
        collector.visit_program(&result.program);
        collector
            .specifiers
            .into_iter()
            .filter_map(|(start, end, path)| {
                rewrite_tsx_vue_shim_specifier(path.as_str(), source_dir)
                    .map(|rewrite| (start, end, rewrite))
            })
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    if serialize_setup_scoped_props && let Some(start) = source.find(SETUP_SCOPED_PROPS_INPUT) {
        rewrites.push((
            start as u32,
            (start + SETUP_SCOPED_PROPS_INPUT.len()) as u32,
            SETUP_SCOPED_PROPS_DECLARATION.into(),
        ));
    }
    if rewrites.is_empty() {
        return None;
    }

    rewrites.sort_by_key(|rewrite| std::cmp::Reverse(rewrite.0));
    let mut output = source.to_compact_string();
    let mut source_map = ImportSourceMap::for_rewrites(rewrites.len());
    for (start, end, new_path) in rewrites {
        output.replace_range(start as usize..end as usize, new_path.as_str());
        source_map.record_replacement(start, end, new_path.len() as u32);
    }
    source_map.finish_descending_replacements();
    Some(RewriteResult {
        code: output,
        source_map,
    })
}

const SETUP_SCOPED_PROPS_INPUT: &str =
    "  const __vize_setup_props = undefined as unknown as $SetupProps;";

// A setup-local interface cannot be named from __setup's declaration return
// type (TS4060). Declaration input alone uses this distributive homomorphic
// projection so TypeScript serializes each branch without changing the normal
// checking surface, diagnostics, or editor projections.
const SETUP_SCOPED_PROPS_DECLARATION: &str = "  const __vize_setup_props = undefined as unknown as $IsAny<$SetupProps> extends true ? $SetupProps : unknown extends $SetupProps ? $SetupProps : $SetupProps extends infer $PropBranch ? $PropBranch extends (...args: any[]) => any ? $PropBranch : { [K in keyof $PropBranch]: $PropBranch[K] } : never;";

fn rewrite_tsx_vue_shim_specifier(path: &str, source_dir: &Path) -> Option<String> {
    if !(path.starts_with("./") || path.starts_with("../")) {
        return None;
    }
    let _ = path.strip_suffix(".vue.ts")?;
    let shim_path = source_dir.join(path);
    let shim_name = shim_path.file_name()?.to_str()?;
    let tsx_path = shim_path.with_file_name(cstr!("{shim_name}x").as_str());
    tsx_path.is_file().then(|| cstr!("{path}x"))
}

#[derive(Default)]
struct ModuleSpecifierCollector {
    specifiers: Vec<(u32, u32, String)>,
}

impl ModuleSpecifierCollector {
    fn push(&mut self, lit: &StringLiteral<'_>) {
        self.specifiers.push((
            lit.span.start + 1,
            lit.span.end - 1,
            lit.value.as_str().into(),
        ));
    }
}

impl<'a> Visit<'a> for ModuleSpecifierCollector {
    fn visit_import_declaration(&mut self, decl: &ImportDeclaration<'a>) {
        self.push(&decl.source);
        walk::walk_import_declaration(self, decl);
    }

    fn visit_export_named_declaration(&mut self, decl: &ExportNamedDeclaration<'a>) {
        if let Some(source) = &decl.source {
            self.push(source);
        }
        walk::walk_export_named_declaration(self, decl);
    }

    fn visit_export_all_declaration(&mut self, decl: &ExportAllDeclaration<'a>) {
        self.push(&decl.source);
        walk::walk_export_all_declaration(self, decl);
    }

    fn visit_import_expression(&mut self, expr: &ImportExpression<'a>) {
        if let Expression::StringLiteral(lit) = &expr.source {
            self.push(lit);
        }
        walk::walk_import_expression(self, expr);
    }

    fn visit_call_expression(&mut self, expr: &CallExpression<'a>) {
        if let Some(lit) = expr.common_js_require() {
            self.push(lit);
        }
        walk::walk_call_expression(self, expr);
    }

    fn visit_ts_import_type(&mut self, import_type: &TSImportType<'a>) {
        self.push(&import_type.source);
        walk::walk_ts_import_type(self, import_type);
    }

    fn visit_ts_external_module_reference(&mut self, reference: &TSExternalModuleReference<'a>) {
        self.push(&reference.expression);
        walk::walk_ts_external_module_reference(self, reference);
    }

    fn visit_ts_module_declaration_name(&mut self, name: &TSModuleDeclarationName<'a>) {
        if let TSModuleDeclarationName::StringLiteral(lit) = name {
            self.push(lit);
        }
        walk::walk_ts_module_declaration_name(self, name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn setup_scoped_props_are_serialized_only_for_declaration_input() {
        let source = cstr!("before\n{SETUP_SCOPED_PROPS_INPUT}\nafter\n");
        assert!(
            rewrite_tsx_vue_shim_specifiers(&source, SourceType::ts(), Path::new("."), false)
                .is_none()
        );

        let rewritten =
            rewrite_tsx_vue_shim_specifiers(&source, SourceType::ts(), Path::new("."), true)
                .expect("declaration input should serialize setup-local props");

        let original_start = source.find(SETUP_SCOPED_PROPS_INPUT).unwrap() as u32;
        let generated_start = rewritten.code.find(SETUP_SCOPED_PROPS_DECLARATION).unwrap() as u32;
        assert_eq!(
            rewritten
                .source_map
                .get_original_offset(generated_start + 16),
            original_start
        );
        assert_eq!(
            rewritten
                .source_map
                .get_original_offset(rewritten.code.find("after").unwrap() as u32),
            source.find("after").unwrap() as u32
        );
    }
}
