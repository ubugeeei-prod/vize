//! A Vue setup method cannot inherit an authored arrow's enclosing receiver.

use oxc_ast::ast::{
    Function, IdentifierReference, NewTarget, Program, PropertyDefinition, StaticBlock,
    ThisExpression,
};
use oxc_ast_visit::Visit;
use oxc_span::{GetSpan, Span};

use super::{JsxComponent, JsxDiagnostic, error};

pub(super) fn check(
    program: &Program<'_>,
    components: &[JsxComponent],
) -> Result<(), JsxDiagnostic> {
    if !components.iter().any(|c| c.component_setup().is_some()) {
        return Ok(());
    }
    let spans: Vec<_> = components
        .iter()
        .filter_map(JsxComponent::component_setup)
        .map(|setup| Span::new(setup.declaration_start, setup.declaration_end))
        .collect();
    check_receivers(
        program,
        &spans,
        "Block-body JSX component setup cannot preserve lexical this, arguments or new.target; use a function declaration or expression-bodied component",
    )
}

pub(super) fn check_spans(program: &Program<'_>, spans: &[Span]) -> Result<(), JsxDiagnostic> {
    check_receivers(
        program,
        spans,
        "Vapor/SSR component setup cannot preserve the authored lexical this, arguments or new.target in this context",
    )
}

fn check_receivers(
    program: &Program<'_>,
    spans: &[Span],
    message: &str,
) -> Result<(), JsxDiagnostic> {
    let mut visitor = Contexts {
        spans,
        missing: None,
        receivers: Vec::new(),
    };
    visitor.visit_program(program);
    if let Some((start, end)) = visitor.missing {
        return Err(error(start, end, message));
    }
    Ok(())
}

struct Contexts<'a> {
    spans: &'a [Span],
    missing: Option<(u32, u32)>,
    receivers: Vec<Span>,
}

impl Contexts<'_> {
    fn record(&mut self, start: u32, end: u32) {
        if self.spans.iter().any(|setup| {
            setup.start <= start
                && end <= setup.end
                && self
                    .receivers
                    .last()
                    .is_none_or(|receiver| receiver.start < setup.start || setup.end < receiver.end)
        }) {
            self.missing = Some((start, end));
        }
    }
}

impl<'a> Visit<'a> for Contexts<'_> {
    fn visit_function(&mut self, function: &Function<'a>, flags: oxc_syntax::scope::ScopeFlags) {
        // A retained normal function owns its context; a nested component
        // wrapper still needs to diagnose captures from that enclosing context.
        self.receivers.push(function.span);
        oxc_ast_visit::walk::walk_function(self, function, flags);
        self.receivers.pop();
    }

    fn visit_property_definition(&mut self, property: &PropertyDefinition<'a>) {
        // Decorators and computed keys use the surrounding receiver. Field
        // initializer values bind the class instance (or the static class).
        self.visit_decorators(&property.decorators);
        self.visit_property_key(&property.key);
        if let Some(value) = &property.value {
            self.receivers.push(value.span());
            self.visit_expression(value);
            self.receivers.pop();
        }
    }

    fn visit_static_block(&mut self, block: &StaticBlock<'a>) {
        self.receivers.push(block.span);
        oxc_ast_visit::walk::walk_static_block(self, block);
        self.receivers.pop();
    }

    fn visit_this_expression(&mut self, expression: &ThisExpression) {
        self.record(expression.span.start, expression.span.end);
    }

    fn visit_identifier_reference(&mut self, identifier: &IdentifierReference<'a>) {
        if identifier.name == "arguments" {
            self.record(identifier.span.start, identifier.span.end);
        }
    }

    fn visit_new_target(&mut self, target: &NewTarget) {
        self.record(target.span.start, target.span.end);
    }
}
