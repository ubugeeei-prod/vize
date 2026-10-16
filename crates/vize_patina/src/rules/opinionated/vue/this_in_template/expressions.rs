//! Inspect retained JavaScript, not string-literal contents or whole expressions.

use super::origin::Origin;
use crate::context::LintContext;
use oxc_allocator::Allocator;
use oxc_ast::ast::{ComputedMemberExpression, Expression, StaticMemberExpression};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::{ParseOptions, Parser};
use oxc_span::{GetSpan, SourceType};
use vize_l0::expression_guard::{expression_is_safe_to_parse, is_expression_trailing_trivia};
use vize_l1::markup::entity::EntityContext;
use vize_relief::ExpressionNode;

pub(super) fn check(
    ctx: &mut LintContext<'_>,
    expression: &ExpressionNode<'_>,
    directive: Option<&str>,
    entity_context: EntityContext,
) {
    let ExpressionNode::Simple(node) = expression else {
        return;
    };
    if !node.content.contains("this") {
        return;
    }
    let Some(origin) = Origin::new(ctx.source, node.content, node.loc.span, entity_context) else {
        return;
    };
    let mut visitor = ThisMembers {
        ctx,
        origin,
        parse_offset: 0,
    };
    if let Some(retained) = node.js_ast
        && retained.raw == node.content
        && let Some(expression) = retained.as_expression()
    {
        visitor.visit_expression(expression.ast);
        return;
    }
    if let Some(retained) = node.js_ast
        && retained.raw == node.content
        && let Some(Ok(parameters)) = retained.as_slot_parameters()
    {
        visitor.visit_formal_parameters(parameters);
        return;
    }
    if directive == Some("for") {
        check_v_for(&mut visitor, node.content);
        return;
    }
    // Multi-statement event bodies do not have an ordinary retained expression.
    // Parse their original bytes directly, allowing the handler's return goal,
    // so spans need no synthetic-wrapper adjustment. Other refused goals stay
    // refused, including pathological nesting and invalid ordinary expressions.
    if directive != Some("on") || !expression_is_safe_to_parse(node.content) {
        return;
    }
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, node.content, SourceType::ts())
        .with_options(ParseOptions {
            allow_return_outside_function: true,
            ..ParseOptions::default()
        })
        .parse();
    if !parsed.panicked && parsed.diagnostics.is_empty() {
        visitor.visit_program(&parsed.program);
    }
}

struct ThisMembers<'ctx, 'source> {
    ctx: &'ctx mut LintContext<'source>,
    origin: Origin,
    parse_offset: u32,
}

fn check_v_for(visitor: &mut ThisMembers<'_, '_>, source: &str) {
    if !expression_is_safe_to_parse(source) {
        return;
    }
    let (bindings, iterable) = vize_croquis::drawer::parse_v_for_expression(source);
    if bindings.is_empty() {
        return;
    }
    let trimmed = source.trim_end();
    let Some(prefix) = trimmed.strip_suffix(iterable.as_str()) else {
        return;
    };
    let allocator = Allocator::default();
    let iterable_offset = prefix.len() as u32;
    // The same original binding goal can contain defaults (`{x=this.x}`),
    // which are executable expressions too. Keep their original offsets.
    let prefix = prefix.trim_end();
    let Some(aliases) = prefix
        .strip_suffix("in")
        .or_else(|| prefix.strip_suffix("of"))
    else {
        return;
    };
    let aliases = aliases.trim();
    let aliases = aliases
        .strip_prefix('(')
        .and_then(|text| text.strip_suffix(')'))
        .unwrap_or(aliases);
    if aliases.contains("this")
        && let Ok(parameters) =
            Parser::new(&allocator, aliases, SourceType::ts()).parse_slot_parameters()
        && aliases
            .get(parameters.span().end as usize..)
            .is_some_and(is_expression_trailing_trivia)
    {
        // `aliases` is a checked subslice of the complete original expression.
        visitor.parse_offset = (aliases.as_ptr().addr() - source.as_ptr().addr()) as u32;
        visitor.visit_formal_parameters(&parameters);
    }
    if let Ok(expression) =
        Parser::new(&allocator, iterable.as_str(), SourceType::ts()).parse_expression()
        && iterable
            .get(expression.span().end as usize..)
            .is_some_and(is_expression_trailing_trivia)
    {
        visitor.parse_offset = iterable_offset;
        visitor.visit_expression(&expression);
    }
}

impl ThisMembers<'_, '_> {
    fn check_member(&mut self, object: &Expression<'_>, property: Option<&str>) {
        let Expression::ThisExpression(this) = object.without_parentheses() else {
            return;
        };
        let Some(property) = property else {
            return;
        };
        // Removing `this` must leave an identifier that resolves to the same
        // component property, rather than a scoped alias or a reserved word.
        if !plain_identifier(property) || self.ctx.is_v_for_var(property) {
            return;
        }
        let Some(range) = self.origin.range(oxc_span::Span::new(
            this.span.start + self.parse_offset,
            this.span.end + self.parse_offset,
        )) else {
            return;
        };
        self.ctx.warn_at_with_help(
            self.ctx.t("vue/this-in-template.message"),
            range,
            self.ctx.t("vue/this-in-template.help"),
        );
    }
}

impl<'a> Visit<'a> for ThisMembers<'_, '_> {
    fn visit_static_member_expression(&mut self, member: &StaticMemberExpression<'a>) {
        self.check_member(&member.object, Some(member.property.name.as_str()));
        walk::walk_static_member_expression(self, member);
    }

    fn visit_computed_member_expression(&mut self, member: &ComputedMemberExpression<'a>) {
        let property = member.static_property_name();
        self.check_member(&member.object, property.as_ref().map(|name| name.as_str()));
        walk::walk_computed_member_expression(self, member);
    }
}

fn plain_identifier(name: &str) -> bool {
    let mut bytes = name.bytes();
    bytes.next().is_some_and(|byte| byte.is_ascii_alphabetic() || matches!(byte, b'_' | b'$'))
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$'))
        && !oxc_syntax::keyword::is_reserved_keyword(name)
        // Match the reference rule's legacy/future keyword exclusions too.
        && !matches!(
            name,
            "abstract" | "arguments" | "boolean" | "byte" | "char" | "double" | "eval"
                | "final" | "float" | "goto" | "int" | "long" | "native" | "short"
                | "synchronized" | "throws" | "transient" | "volatile"
        )
}
