//! Template-expression projection through the S4 emission document (P4-5b).
//!
//! Each interpolation and `v-bind` value is emitted by the JS [`ExprDialect`]
//! into one [`EmitDocument`]. The document's links are the [`ProjectionMapping`]
//! rows, so the projection has no mapping model of its own. [`ExprRef::Opaque`]
//! keeps the pessimal answers: nothing is enumerated, it is not constant, it
//! is emitted verbatim, and every inner range maps to the whole expression.

use oxc_ast::ast::{ArrowFunctionExpression, Class, Expression, Function, IdentifierReference};
use oxc_ast_visit::Visit;
use oxc_ast_visit::walk::{walk_arrow_function_expression, walk_class, walk_function};
use oxc_syntax::scope::ScopeFlags;
use vize_atelier_core::codegen::document::EmitDocument;
use vize_carton::{Allocator, Span, String};
use vize_s1_to_s2::lower;
use vize_s2::expr::{ExprDialect, ExprRef};
use vize_s2::op::{BindingOp, Op, Region};

use crate::virtual_ts::ProjectionMapping;

/// Project `source`'s template expressions into mapping rows.
pub fn project_template_expressions(source: &str) -> ProjectionMapping {
    let allocator = Allocator::new();
    let (tree, errors) = vize_s1::parse(&allocator, source);
    let lowered = lower(&allocator, &tree, &errors);
    let mut document = EmitDocument::new(true);
    project_region(&mut document, &JsDialect, &lowered.root);
    ProjectionMapping::from_emit_document(&document)
}

fn project_region<D: ExprDialect>(document: &mut EmitDocument, dialect: &D, region: &Region<'_>) {
    for op in &region.ops {
        project_op(document, dialect, op);
    }
}

fn project_op<D: ExprDialect>(document: &mut EmitDocument, dialect: &D, op: &Op<'_>) {
    match op {
        Op::Element(element) => {
            project_bindings(document, dialect, &element.bindings);
            project_region(document, dialect, &element.children);
        }
        Op::Component(component) => {
            project_bindings(document, dialect, &component.bindings);
            project_region(document, dialect, &component.children);
        }
        Op::Interpolation(interpolation) => {
            project_expr(document, dialect, interpolation.expression);
        }
        Op::If(if_op) => {
            for branch in &if_op.branches {
                project_region(document, dialect, &branch.region);
            }
        }
        Op::For(for_op) => project_region(document, dialect, &for_op.region),
        Op::Slot(slot) => {
            project_bindings(document, dialect, &slot.bindings);
            project_region(document, dialect, &slot.fallback);
        }
        Op::Text(_) | Op::Comment(_) => {}
    }
}

/// `ui.bind` values (`:title="name"`). The row is the expression, not the
/// directive.
fn project_bindings<D: ExprDialect>(
    document: &mut EmitDocument,
    dialect: &D,
    bindings: &[BindingOp<'_>],
) {
    for binding in bindings {
        if let BindingOp::Bind(bind) = binding
            && let Some(value) = bind.value
        {
            project_expr(document, dialect, value);
        }
    }
}

/// Emit one expression and record the authored range [`ExprDialect::map_span`]
/// answers for the emitted text.
fn project_expr<D: ExprDialect>(document: &mut EmitDocument, dialect: &D, expr: ExprRef<'_>) {
    let mut generated = String::default();
    if dialect.emit(expr, &mut generated).is_err() || generated.is_empty() {
        return;
    }
    let inner_end = u32::try_from(generated.len()).unwrap_or(u32::MAX);
    let authored = dialect.map_span(expr, Span::new(0, inner_end));
    if authored.is_empty() {
        return;
    }
    if !document.is_empty() {
        document.push_str("\n");
    }
    document.push_linked(&generated, authored);
}

/// The JS expression dialect. Opaque answers are the pessimal laws and are
/// not refined from the text.
struct JsDialect;

impl ExprDialect for JsDialect {
    fn enumerate_bindings(&self, expr: ExprRef<'_>, each: &mut dyn FnMut(&str)) {
        let ExprRef::Js(js) = expr else {
            return;
        };
        let mut names = Names { each, depth: 0 };
        names.visit_expression(js.ast);
    }

    fn bindings_are_exact(&self, expr: ExprRef<'_>) -> bool {
        let ExprRef::Js(js) = expr else {
            return false;
        };
        let mut probe = ScopeProbe { found: false };
        probe.visit_expression(js.ast);
        !probe.found
    }

    fn is_constant(&self, expr: ExprRef<'_>) -> bool {
        match expr {
            ExprRef::Js(js) => is_constant_expr(js.ast),
            ExprRef::Opaque(_) | ExprRef::Filter(_) | ExprRef::Foreign(_) => false,
        }
    }

    fn map_span(&self, expr: ExprRef<'_>, inner: Span) -> Span {
        let span = expr.span();
        // An opaque expression has no structure to localize inside.
        if matches!(expr, ExprRef::Opaque(_)) {
            return span;
        }
        let Some(len) = u32::try_from(expr.source().len()).ok() else {
            return span;
        };
        if len == span.len() && inner.start <= inner.end && inner.end <= len {
            Span::new(span.start + inner.start, span.start + inner.end)
        } else {
            span
        }
    }

    fn emit(
        &self,
        expr: ExprRef<'_>,
        out: &mut dyn core::fmt::Write,
    ) -> Result<(), core::fmt::Error> {
        match expr {
            ExprRef::Js(js) => out.write_str(js.source),
            ExprRef::Filter(filter) => out.write_str(filter.source),
            // Pessimal law 5: verbatim, or refusal. Never a fixed-up spelling.
            ExprRef::Opaque(opaque) => out.write_str(opaque.source),
            ExprRef::Foreign(_) => Err(core::fmt::Error),
        }
    }
}

fn is_constant_expr(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::StringLiteral(_)
        | Expression::BigIntLiteral(_) => true,
        Expression::TemplateLiteral(template) => template.expressions.is_empty(),
        Expression::ParenthesizedExpression(inner) => is_constant_expr(&inner.expression),
        _ => false,
    }
}

struct Names<'a> {
    each: &'a mut dyn FnMut(&str),
    depth: u32,
}

impl Visit<'_> for Names<'_> {
    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'_>) {
        if self.depth == 0 {
            (self.each)(ident.name.as_str());
        }
    }

    fn visit_function(&mut self, function: &Function<'_>, flags: ScopeFlags) {
        self.depth += 1;
        walk_function(self, function, flags);
        self.depth -= 1;
    }

    fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'_>) {
        self.depth += 1;
        walk_arrow_function_expression(self, arrow);
        self.depth -= 1;
    }

    fn visit_class(&mut self, class: &Class<'_>) {
        self.depth += 1;
        walk_class(self, class);
        self.depth -= 1;
    }
}

struct ScopeProbe {
    found: bool,
}

impl Visit<'_> for ScopeProbe {
    fn visit_function(&mut self, function: &Function<'_>, flags: ScopeFlags) {
        self.found = true;
        walk_function(self, function, flags);
    }

    fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'_>) {
        self.found = true;
        walk_arrow_function_expression(self, arrow);
    }

    fn visit_class(&mut self, class: &Class<'_>) {
        self.found = true;
        walk_class(self, class);
    }
}

#[cfg(test)]
mod tests {
    use super::project_template_expressions;
    use crate::virtual_ts::VizeMapping;

    #[test]
    fn projects_count_interpolation_onto_its_expression_span() {
        let source = "{{ count }}";
        let start = source.find("count").unwrap();
        let end = start + "count".len();
        let mapping = project_template_expressions(source);
        assert_eq!(mapping.spans(), &[VizeMapping::new(0..5, start..end)]);
        assert_eq!(
            mapping.diagnostic_range_to_authored(0, 5),
            Some((start, end))
        );
    }

    #[test]
    fn projects_title_bind_onto_its_expression_span() {
        let source = r#"<div :title="name"></div>"#;
        let start = source.find("name").unwrap();
        let end = start + "name".len();
        let mapping = project_template_expressions(source);
        assert_eq!(mapping.spans(), &[VizeMapping::new(0..4, start..end)]);
        assert_eq!(
            mapping.diagnostic_range_to_authored(0, 4),
            Some((start, end))
        );
    }
}
