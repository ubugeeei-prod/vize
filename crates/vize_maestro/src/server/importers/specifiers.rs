//! Static module specifiers that can make an open script depend on a file.

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    CallExpression, ExportAllDeclaration, ExportNamedDeclaration, Expression, ImportDeclaration,
    ImportExpression, StringLiteral, TSExternalModuleReference, TSImportType,
};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_s0::{String, ToCompactString};

pub(super) fn collect(source: &str, source_type: SourceType) -> Vec<String> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, source_type).parse();
    let mut collector = Collector::default();
    collector.visit_program(&parsed.program);
    collector.specifiers
}

#[derive(Default)]
struct Collector {
    specifiers: Vec<String>,
}

impl Collector {
    fn push(&mut self, literal: &StringLiteral<'_>) {
        let value = literal.value.as_str();
        if !self.specifiers.iter().any(|known| known.as_str() == value) {
            self.specifiers.push(value.to_compact_string());
        }
    }
}

impl<'a> Visit<'a> for Collector {
    fn visit_import_declaration(&mut self, declaration: &ImportDeclaration<'a>) {
        self.push(&declaration.source);
        walk::walk_import_declaration(self, declaration);
    }

    fn visit_export_named_declaration(&mut self, declaration: &ExportNamedDeclaration<'a>) {
        if let Some(source) = &declaration.source {
            self.push(source);
        }
        walk::walk_export_named_declaration(self, declaration);
    }

    fn visit_export_all_declaration(&mut self, declaration: &ExportAllDeclaration<'a>) {
        self.push(&declaration.source);
        walk::walk_export_all_declaration(self, declaration);
    }

    fn visit_import_expression(&mut self, expression: &ImportExpression<'a>) {
        if let Expression::StringLiteral(source) = &expression.source {
            self.push(source);
        }
        walk::walk_import_expression(self, expression);
    }

    fn visit_call_expression(&mut self, expression: &CallExpression<'a>) {
        if let Some(source) = expression.common_js_require() {
            self.push(source);
        }
        walk::walk_call_expression(self, expression);
    }

    fn visit_ts_import_type(&mut self, import_type: &TSImportType<'a>) {
        self.push(&import_type.source);
        walk::walk_ts_import_type(self, import_type);
    }

    fn visit_ts_external_module_reference(&mut self, reference: &TSExternalModuleReference<'a>) {
        self.push(&reference.expression);
        walk::walk_ts_external_module_reference(self, reference);
    }
}
