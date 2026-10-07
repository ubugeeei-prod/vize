use oxc_ast::ast::{
    CallExpression, ExportAllDeclaration, ExportNamedDeclaration, Expression, ImportDeclaration,
    ImportExpression, StringLiteral, TSExternalModuleReference, TSImportType,
};
use oxc_ast_visit::{Visit, walk};
use vize_l0::String;

#[derive(Default)]
pub(super) struct Specifiers(pub(super) Vec<(u32, u32, String)>);

impl Specifiers {
    fn push(&mut self, literal: &StringLiteral<'_>) {
        self.0.push((
            literal.span.start + 1,
            literal.span.end - 1,
            literal.value.as_str().into(),
        ));
    }
}

impl<'a> Visit<'a> for Specifiers {
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
