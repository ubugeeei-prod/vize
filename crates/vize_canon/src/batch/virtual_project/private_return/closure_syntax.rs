//! Strict declaration/source facts only; these do not resolve or authenticate a closure.

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    BindingIdentifier, CallExpression, ExportAllDeclaration, ExportNamedDeclaration, Expression,
    ImportDeclaration, ImportExpression, StringLiteral, TSExternalModuleReference, TSImportType,
    TSModuleDeclarationName, TemplateLiteral,
};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::String;

use super::syntax::MOVED_NAMES;

#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct ClosureSyntaxFacts {
    pub(super) moved_name_binding: bool,
    pub(super) module_specifiers: Vec<String>,
    pub(super) module_declarations: Vec<String>,
    pub(super) opaque_module_reference: bool,
    pub(super) reference_directives: bool,
}

/// TS/TSX inputs use one complete parse; broken/recovered trees supply no prefix.
/// JS/JSX JSDoc imports/types are not exposed by this AST, so those languages
/// are unsupported rather than an apparently complete empty reference vector.
pub(super) fn inspect(source: &str, source_type: SourceType) -> Option<ClosureSyntaxFacts> {
    if !source_type.is_typescript() {
        return None;
    }
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, source_type).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return None;
    }
    let mut collector = Collector::default();
    collector.visit_program(&parsed.program);
    // Callback pre-order differs from literal order for nested calls/types.
    // Preserve source order and duplicates, like the existing import collector.
    collector.specifiers.sort_by_key(|(offset, _)| *offset);
    collector.declarations.sort_by_key(|(offset, _)| *offset);
    Some(ClosureSyntaxFacts {
        moved_name_binding: collector.moved_name_binding,
        module_specifiers: collector
            .specifiers
            .into_iter()
            .map(|(_, text)| text)
            .collect(),
        module_declarations: collector
            .declarations
            .into_iter()
            .map(|(_, text)| text)
            .collect(),
        opaque_module_reference: collector.opaque_module_reference,
        reference_directives: reference_marker(source),
    })
}

/// This is deliberately only a refusal marker, not directive parsing or
/// resolution. A line inside a multiline string may conservatively refuse.
fn reference_marker(source: &str) -> bool {
    source
        .split(['\n', '\r', '\u{2028}', '\u{2029}'])
        .any(|line| {
            line.trim_start_matches(|character: char| {
                character.is_whitespace() || character == '\u{feff}'
            })
            .strip_prefix("///")
            .is_some_and(|comment| {
                comment
                    .trim_start_matches(|character: char| {
                        character.is_whitespace() || character == '\u{feff}'
                    })
                    .starts_with("<reference")
            })
        })
}

#[derive(Default)]
struct Collector {
    moved_name_binding: bool,
    opaque_module_reference: bool,
    specifiers: Vec<(u32, String)>,
    declarations: Vec<(u32, String)>,
}

impl Collector {
    fn literal(&mut self, literal: &StringLiteral<'_>) {
        self.specifiers
            .push((literal.span.start, String::from(literal.value.as_str())));
    }

    fn expression(&mut self, expression: &Expression<'_>) {
        match expression.get_inner_expression() {
            Expression::StringLiteral(literal) => self.literal(literal),
            Expression::TemplateLiteral(template) => self.template(template),
            _ => self.opaque_module_reference = true,
        }
    }

    fn template(&mut self, template: &TemplateLiteral<'_>) {
        let value = template
            .expressions
            .is_empty()
            .then(|| template.quasis.first())
            .flatten()
            .and_then(|quasi| quasi.value.cooked.as_ref());
        if let Some(value) = value {
            self.specifiers
                .push((template.span.start, String::from(value.as_str())));
        } else {
            self.opaque_module_reference = true;
        }
    }
}

impl<'a> Visit<'a> for Collector {
    fn visit_binding_identifier(&mut self, node: &BindingIdentifier<'a>) {
        self.moved_name_binding |= MOVED_NAMES.contains(&node.name.as_str());
    }

    fn visit_import_declaration(&mut self, node: &ImportDeclaration<'a>) {
        self.literal(&node.source);
        walk::walk_import_declaration(self, node);
    }

    fn visit_export_named_declaration(&mut self, node: &ExportNamedDeclaration<'a>) {
        if let Some(source) = &node.source {
            self.literal(source);
        }
        walk::walk_export_named_declaration(self, node);
    }

    fn visit_export_all_declaration(&mut self, node: &ExportAllDeclaration<'a>) {
        self.literal(&node.source);
        walk::walk_export_all_declaration(self, node);
    }

    fn visit_ts_import_type(&mut self, node: &TSImportType<'a>) {
        self.literal(&node.source);
        walk::walk_ts_import_type(self, node);
    }

    fn visit_ts_external_module_reference(&mut self, node: &TSExternalModuleReference<'a>) {
        self.literal(&node.expression);
        walk::walk_ts_external_module_reference(self, node);
    }

    fn visit_ts_module_declaration_name(&mut self, node: &TSModuleDeclarationName<'a>) {
        if let TSModuleDeclarationName::StringLiteral(literal) = node {
            self.declarations
                .push((literal.span.start, String::from(literal.value.as_str())));
        }
        walk::walk_ts_module_declaration_name(self, node);
    }

    fn visit_import_expression(&mut self, node: &ImportExpression<'a>) {
        self.expression(&node.source);
        walk::walk_import_expression(self, node);
    }

    fn visit_call_expression(&mut self, node: &CallExpression<'a>) {
        if matches!(node.callee.get_inner_expression(), Expression::Identifier(name) if name.name == "require")
        {
            if node.arguments.len() != 1 {
                self.opaque_module_reference = true;
            }
            if let Some(expression) = node.arguments.first().and_then(|arg| arg.as_expression()) {
                self.expression(expression);
            } else {
                self.opaque_module_reference = true;
            }
        }
        walk::walk_call_expression(self, node);
    }
}

#[cfg(test)]
mod tests;
