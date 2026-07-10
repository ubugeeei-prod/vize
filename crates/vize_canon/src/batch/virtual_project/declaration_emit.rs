use std::path::Path;

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

use crate::batch::CorsaResult;

use super::{VirtualFile, VirtualProject};

impl VirtualProject {
    pub(super) fn rewrite_tsx_vue_declaration_inputs(&self) -> CorsaResult<()> {
        for file in self.virtual_files_sorted() {
            let path = file.virtual_path.as_path();
            if !is_script_path(path) {
                continue;
            }
            let content = std::fs::read_to_string(path)?;
            let source_type = SourceType::from_path(path).unwrap_or_else(|_| SourceType::ts());
            let source_dir = path.parent().unwrap_or_else(|| Path::new("."));
            let Some(rewritten) =
                rewrite_tsx_vue_shim_specifiers(&content, source_type, source_dir)
            else {
                continue;
            };
            std::fs::write(path, rewritten.as_str())?;
        }
        Ok(())
    }

    pub(super) fn declaration_emit_include_paths(&self) -> Vec<&Path> {
        self.virtual_files
            .values()
            .filter(|file| !self.is_tsx_vue_import_shim(file))
            .map(|file| file.virtual_path.as_path())
            .collect()
    }

    fn is_tsx_vue_import_shim(&self, file: &VirtualFile) -> bool {
        if file.original_path != file.virtual_path {
            return false;
        }
        let Some(file_name) = file.virtual_path.file_name().and_then(|name| name.to_str()) else {
            return false;
        };
        if !file_name.ends_with(".vue.ts") {
            return false;
        }
        let tsx_path = file
            .virtual_path
            .with_file_name(cstr!("{file_name}x").as_str());
        self.virtual_files.contains_key(&tsx_path)
    }
}

fn is_script_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| matches!(extension, "ts" | "tsx" | "mts" | "cts"))
}

fn rewrite_tsx_vue_shim_specifiers(
    source: &str,
    source_type: SourceType,
    source_dir: &Path,
) -> Option<String> {
    if !source.contains(".vue.ts") {
        return None;
    }

    let allocator = Allocator::default();
    let parser = Parser::new(&allocator, source, source_type);
    let result = parser.parse();
    let mut collector = ModuleSpecifierCollector::default();
    collector.visit_program(&result.program);

    let mut rewrites = collector
        .specifiers
        .into_iter()
        .filter_map(|(start, end, path)| {
            rewrite_tsx_vue_shim_specifier(path.as_str(), source_dir)
                .map(|rewrite| (start, end, rewrite))
        })
        .collect::<Vec<_>>();
    if rewrites.is_empty() {
        return None;
    }

    rewrites.sort_by_key(|rewrite| std::cmp::Reverse(rewrite.0));
    let mut output = source.to_compact_string();
    for (start, end, new_path) in rewrites {
        output.replace_range(start as usize..end as usize, new_path.as_str());
    }
    Some(output)
}

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
