//! Preserve the standalone native renderer admission that predates authored modules.

use oxc_ast::ast::{ArrowFunctionExpression, Function, IdentifierReference, Statement};
use oxc_ast_visit::Visit;
use oxc_semantic::{Scoping, SemanticBuilder};
use oxc_span::{GetSpan, Span};

use super::JsxComponent;
use crate::JsxLang;

pub(super) fn admitted(
    components: &[JsxComponent],
    spans: &[(u32, u32)],
    source: &str,
    lang: JsxLang,
) -> bool {
    let allocator = oxc_allocator::Allocator::default();
    let parsed = crate::parse_module(&allocator, source, lang);
    if parsed.program.body.iter().any(|statement| {
        matches!(
            statement,
            Statement::ImportDeclaration(_)
                | Statement::ExportNamedDeclaration(_)
                | Statement::ExportDefaultDeclaration(_)
                | Statement::ExportAllDeclaration(_)
        )
    }) {
        return false;
    }
    let mut functions = FunctionOwners(Vec::new());
    functions.visit_program(&parsed.program);
    if spans.iter().any(|&(start, end)| {
        !functions
            .0
            .iter()
            .any(|owner| owner.start <= start && end <= owner.end)
    }) {
        return false;
    }
    let scoping = SemanticBuilder::new()
        .build(&parsed.program)
        .semantic
        .into_scoping();
    let mut captures = Captures {
        scoping: &scoping,
        components,
        spans,
        missing: false,
    };
    captures.visit_program(&parsed.program);
    !captures.missing
}

struct FunctionOwners(Vec<Span>);

impl<'a> Visit<'a> for FunctionOwners {
    fn visit_function(&mut self, function: &Function<'a>, flags: oxc_syntax::scope::ScopeFlags) {
        self.0.push(function.span);
        oxc_ast_visit::walk::walk_function(self, function, flags);
    }

    fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'a>) {
        self.0.push(arrow.span);
        oxc_ast_visit::walk::walk_arrow_function_expression(self, arrow);
    }
}

struct Captures<'a> {
    scoping: &'a Scoping,
    components: &'a [JsxComponent],
    spans: &'a [(u32, u32)],
    missing: bool,
}

impl<'a> Visit<'a> for Captures<'_> {
    fn visit_statement(&mut self, statement: &Statement<'a>) {
        let span = statement.span();
        if self
            .components
            .iter()
            .filter_map(JsxComponent::component_setup)
            .any(|setup| setup.setup_start <= span.start && span.end <= setup.setup_end)
        {
            self.missing = true;
        }
        oxc_ast_visit::walk::walk_statement(self, statement);
    }

    fn visit_identifier_reference(&mut self, identifier: &IdentifierReference<'a>) {
        let Some(&(root_start, root_end)) = self
            .spans
            .iter()
            .find(|&&(start, end)| start <= identifier.span.start && identifier.span.end <= end)
        else {
            return;
        };
        let Some(reference) = identifier.reference_id.get() else {
            return;
        };
        let Some(symbol) = self.scoping.get_reference(reference).symbol_id() else {
            return;
        };
        let binding = self.scoping.symbol_span(symbol);
        if !(root_start <= binding.start && binding.end <= root_end) {
            self.missing = true;
        }
    }
}
