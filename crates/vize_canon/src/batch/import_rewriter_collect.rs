//! AST visitor collecting every module specifier literal the rewriter may
//! replace, with the span of the literal's *contents* (quotes excluded).

use oxc_ast::ast::{
    CallExpression, ExportAllDeclaration, ExportNamedDeclaration, Expression, ImportDeclaration,
    ImportExpression, StringLiteral, TSExternalModuleReference, TSImportType,
    TSModuleDeclarationName,
};
use oxc_ast_visit::{Visit, walk};
use vize_carton::String;

pub(super) struct ModuleSpecifierCollector {
    pub(super) specifiers: Vec<(u32, u32, String)>,
}

impl ModuleSpecifierCollector {
    pub(super) fn new() -> Self {
        Self {
            specifiers: Vec::new(),
        }
    }

    fn push(&mut self, start: u32, end: u32, specifier: &str) {
        self.specifiers.push((start + 1, end - 1, specifier.into()));
    }

    fn push_literal(&mut self, lit: &StringLiteral<'_>) {
        self.push(lit.span.start, lit.span.end, lit.value.as_str());
    }
}

impl<'a> Visit<'a> for ModuleSpecifierCollector {
    fn visit_import_declaration(&mut self, decl: &ImportDeclaration<'a>) {
        self.push_literal(&decl.source);
        walk::walk_import_declaration(self, decl);
    }

    fn visit_export_named_declaration(&mut self, decl: &ExportNamedDeclaration<'a>) {
        if let Some(source) = &decl.source {
            self.push_literal(source);
        }
        walk::walk_export_named_declaration(self, decl);
    }

    fn visit_export_all_declaration(&mut self, decl: &ExportAllDeclaration<'a>) {
        self.push_literal(&decl.source);
        walk::walk_export_all_declaration(self, decl);
    }

    fn visit_import_expression(&mut self, expr: &ImportExpression<'a>) {
        if let Expression::StringLiteral(lit) = &expr.source {
            self.push_literal(lit);
        }
        walk::walk_import_expression(self, expr);
    }

    fn visit_call_expression(&mut self, expr: &CallExpression<'a>) {
        if let Some(lit) = expr.common_js_require() {
            self.push(lit.span.start, lit.span.end, lit.value.as_str());
        }
        walk::walk_call_expression(self, expr);
    }

    fn visit_ts_import_type(&mut self, import_type: &TSImportType<'a>) {
        self.push_literal(&import_type.source);
        walk::walk_ts_import_type(self, import_type);
    }

    fn visit_ts_external_module_reference(&mut self, reference: &TSExternalModuleReference<'a>) {
        self.push_literal(&reference.expression);
        walk::walk_ts_external_module_reference(self, reference);
    }

    fn visit_ts_module_declaration_name(&mut self, name: &TSModuleDeclarationName<'a>) {
        if let TSModuleDeclarationName::StringLiteral(lit) = name {
            self.push_literal(lit);
        }
        walk::walk_ts_module_declaration_name(self, name);
    }
}
