//! AST visitor collecting every module specifier literal the rewriter may
//! replace, with the span of the literal's *contents* (quotes excluded).

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    CallExpression, ExportAllDeclaration, ExportNamedDeclaration, Expression, ImportAttributeKey,
    ImportDeclaration, ImportExpression, ObjectExpression, ObjectPropertyKind, PropertyKey,
    StringLiteral, TSExternalModuleReference, TSImportType, TSModuleDeclarationName, WithClause,
};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::String;

pub(super) fn collect_specifier_occurrences(
    source: &str,
    source_type: SourceType,
) -> Vec<(String, crate::PackageResolutionMode)> {
    let allocator = Allocator::default();
    let result = Parser::new(&allocator, source, source_type).parse();
    let mut collector = ModuleSpecifierCollector::new();
    collector.visit_program(&result.program);
    let mut occurrences = Vec::new();
    for (_, _, path, mode) in collector.specifiers {
        if !occurrences.contains(&(path.clone(), mode)) {
            occurrences.push((path, mode));
        }
    }
    occurrences
}

pub(super) struct ModuleSpecifierCollector {
    pub(super) specifiers: Vec<(u32, u32, String, crate::PackageResolutionMode)>,
}

impl ModuleSpecifierCollector {
    pub(super) fn new() -> Self {
        Self {
            specifiers: Vec::new(),
        }
    }

    fn push(&mut self, start: u32, end: u32, specifier: &str, mode: crate::PackageResolutionMode) {
        self.specifiers
            .push((start + 1, end - 1, specifier.into(), mode));
    }

    fn push_literal(&mut self, lit: &StringLiteral<'_>, mode: crate::PackageResolutionMode) {
        self.push(lit.span.start, lit.span.end, lit.value.as_str(), mode);
    }
}

impl<'a> Visit<'a> for ModuleSpecifierCollector {
    fn visit_import_declaration(&mut self, decl: &ImportDeclaration<'a>) {
        self.push_literal(
            &decl.source,
            mode_from_with_clause(decl.with_clause.as_deref()),
        );
        walk::walk_import_declaration(self, decl);
    }

    fn visit_export_named_declaration(&mut self, decl: &ExportNamedDeclaration<'a>) {
        if let Some(source) = &decl.source {
            self.push_literal(source, mode_from_with_clause(decl.with_clause.as_deref()));
        }
        walk::walk_export_named_declaration(self, decl);
    }

    fn visit_export_all_declaration(&mut self, decl: &ExportAllDeclaration<'a>) {
        self.push_literal(
            &decl.source,
            mode_from_with_clause(decl.with_clause.as_deref()),
        );
        walk::walk_export_all_declaration(self, decl);
    }

    fn visit_import_expression(&mut self, expr: &ImportExpression<'a>) {
        if let Expression::StringLiteral(lit) = &expr.source {
            self.push_literal(lit, mode_from_options(expr.options.as_ref()));
        }
        walk::walk_import_expression(self, expr);
    }

    fn visit_call_expression(&mut self, expr: &CallExpression<'a>) {
        if let Some(lit) = expr.common_js_require() {
            self.push(
                lit.span.start,
                lit.span.end,
                lit.value.as_str(),
                crate::PackageResolutionMode::Require,
            );
        }
        walk::walk_call_expression(self, expr);
    }

    fn visit_ts_import_type(&mut self, import_type: &TSImportType<'a>) {
        self.push_literal(
            &import_type.source,
            import_type
                .options
                .as_deref()
                .and_then(mode_from_options_object)
                .unwrap_or(crate::PackageResolutionMode::Contextual),
        );
        walk::walk_ts_import_type(self, import_type);
    }

    fn visit_ts_external_module_reference(&mut self, reference: &TSExternalModuleReference<'a>) {
        self.push_literal(&reference.expression, crate::PackageResolutionMode::Require);
        walk::walk_ts_external_module_reference(self, reference);
    }

    fn visit_ts_module_declaration_name(&mut self, name: &TSModuleDeclarationName<'a>) {
        if let TSModuleDeclarationName::StringLiteral(lit) = name {
            self.push_literal(lit, crate::PackageResolutionMode::Contextual);
        }
        walk::walk_ts_module_declaration_name(self, name);
    }
}

fn mode_from_with_clause(clause: Option<&WithClause<'_>>) -> crate::PackageResolutionMode {
    clause
        .into_iter()
        .flat_map(|clause| &clause.with_entries)
        .find_map(|attribute| {
            let key = match &attribute.key {
                ImportAttributeKey::Identifier(key) => key.name.as_str(),
                ImportAttributeKey::StringLiteral(key) => key.value.as_str(),
            };
            (key == "resolution-mode")
                .then(|| {
                    crate::PackageResolutionMode::from_explicit_attribute(&attribute.value.value)
                })
                .flatten()
        })
        .unwrap_or(crate::PackageResolutionMode::Contextual)
}

fn mode_from_options(options: Option<&Expression<'_>>) -> crate::PackageResolutionMode {
    options
        .and_then(expression_object)
        .and_then(mode_from_options_object)
        .unwrap_or(crate::PackageResolutionMode::Import)
}

fn mode_from_options_object(
    options: &ObjectExpression<'_>,
) -> Option<crate::PackageResolutionMode> {
    let attributes = expression_object(
        object_property(options, "with").or_else(|| object_property(options, "assert"))?,
    )?;
    let Expression::StringLiteral(value) = object_property(attributes, "resolution-mode")? else {
        return None;
    };
    crate::PackageResolutionMode::from_explicit_attribute(&value.value)
}

fn expression_object<'a>(expression: &'a Expression<'a>) -> Option<&'a ObjectExpression<'a>> {
    let Expression::ObjectExpression(object) = expression else {
        return None;
    };
    Some(object)
}

fn object_property<'a>(object: &'a ObjectExpression<'a>, name: &str) -> Option<&'a Expression<'a>> {
    object.properties.iter().find_map(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        let key = match &property.key {
            PropertyKey::StaticIdentifier(key) => key.name.as_str(),
            PropertyKey::StringLiteral(key) => key.value.as_str(),
            _ => return None,
        };
        (key == name).then_some(&property.value)
    })
}
