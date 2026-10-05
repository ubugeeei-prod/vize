//! Original JS/TS declaration spans; no generated module or type query.

use oxc_ast::ast::{BindingPattern, Declaration, Expression, Statement, VariableDeclarationKind};
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};
use tower_lsp::lsp_types::{DocumentSymbol, SymbolKind};
use vize_atelier_sfc::SfcScriptBlock;
use vize_l0::line_index::LineIndex;

mod members;

pub(super) fn children(
    block: &SfcScriptBlock<'_>,
    index: &LineIndex<'_>,
) -> Option<Vec<DocumentSymbol>> {
    let allocator = oxc_allocator::Allocator::default();
    let source_type = match block.lang.as_deref() {
        Some("tsx") => SourceType::tsx(),
        Some("jsx") => SourceType::jsx(),
        Some("ts") => SourceType::ts(),
        _ => SourceType::mjs(),
    };
    let parsed = Parser::new(&allocator, &block.content, source_type).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return None;
    }
    let mut symbols = Vec::new();
    for statement in &parsed.program.body {
        let declaration = match statement {
            Statement::ExportNamedDeclaration(export) => export.declaration.as_ref(),
            other => other.as_declaration(),
        };
        if let Some(declaration) = declaration {
            collect(declaration, index, block.loc.start, &mut symbols);
        }
    }
    (!symbols.is_empty()).then_some(symbols)
}

fn collect(
    declaration: &Declaration<'_>,
    index: &LineIndex<'_>,
    base: usize,
    out: &mut Vec<DocumentSymbol>,
) {
    match declaration {
        Declaration::VariableDeclaration(declaration) => {
            for binding in &declaration.declarations {
                let init = binding.init.as_ref().map(members::unwrapped);
                let kind = match init {
                    Some(
                        Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_),
                    ) => SymbolKind::FUNCTION,
                    Some(Expression::ObjectExpression(_)) => SymbolKind::OBJECT,
                    _ if declaration.kind == VariableDeclarationKind::Const => SymbolKind::CONSTANT,
                    _ => SymbolKind::VARIABLE,
                };
                let children = init
                    .map(|value| members::object_children(value, index, base))
                    .unwrap_or_default();
                bindings(
                    &binding.id,
                    declaration.span,
                    kind,
                    index,
                    base,
                    &children,
                    out,
                );
            }
        }
        Declaration::FunctionDeclaration(function) => {
            if let Some(id) = &function.id {
                out.push(super::symbol(
                    id.name.as_str(),
                    SymbolKind::FUNCTION,
                    index,
                    base,
                    function.span,
                    id.span,
                    Vec::new(),
                ));
            }
        }
        Declaration::ClassDeclaration(class) => {
            if let Some(id) = &class.id {
                out.push(super::symbol(
                    id.name.as_str(),
                    SymbolKind::CLASS,
                    index,
                    base,
                    class.span,
                    id.span,
                    members::class_children(class, index, base),
                ));
            }
        }
        Declaration::TSInterfaceDeclaration(interface) => {
            out.push(super::symbol(
                interface.id.name.as_str(),
                SymbolKind::INTERFACE,
                index,
                base,
                interface.span,
                interface.id.span,
                Vec::new(),
            ));
        }
        Declaration::TSTypeAliasDeclaration(alias) => {
            out.push(super::symbol(
                alias.id.name.as_str(),
                SymbolKind::TYPE_PARAMETER,
                index,
                base,
                alias.span,
                alias.id.span,
                Vec::new(),
            ));
        }
        _ => {}
    }
}

fn bindings(
    pattern: &BindingPattern<'_>,
    declaration: Span,
    kind: SymbolKind,
    index: &LineIndex<'_>,
    base: usize,
    children: &[DocumentSymbol],
    out: &mut Vec<DocumentSymbol>,
) {
    match pattern {
        BindingPattern::BindingIdentifier(id) => out.push(super::symbol(
            id.name.as_str(),
            kind,
            index,
            base,
            declaration,
            id.span,
            children.to_vec(),
        )),
        BindingPattern::ObjectPattern(object) => {
            for property in &object.properties {
                bindings(&property.value, declaration, kind, index, base, &[], out);
            }
            if let Some(rest) = &object.rest {
                bindings(&rest.argument, declaration, kind, index, base, &[], out);
            }
        }
        BindingPattern::ArrayPattern(array) => {
            for item in array.elements.iter().flatten() {
                bindings(item, declaration, kind, index, base, &[], out);
            }
            if let Some(rest) = &array.rest {
                bindings(&rest.argument, declaration, kind, index, base, &[], out);
            }
        }
        BindingPattern::AssignmentPattern(assignment) => {
            bindings(&assignment.left, declaration, kind, index, base, &[], out)
        }
    }
}
