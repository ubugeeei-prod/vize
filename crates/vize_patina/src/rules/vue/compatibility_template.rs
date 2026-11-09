//! Compatibility rules for JavaScript retained inside Vue templates.
//!
//! The core-extension rules share one retained-AST walker. This is important:
//! scanning expression text would mistake strings and comments for executable
//! syntax, while reparsing every enabled rule would multiply parser cost.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::ir::ByteRange;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use oxc_ast::ast::{
    ArrayExpression, ArrayExpressionElement, ArrayPattern, BinaryExpression,
    ComputedMemberExpression, ConditionalExpression, Expression, IdentifierReference,
    NumericLiteral, ObjectExpression, ObjectPattern, ObjectPropertyKind, PropertyKey,
    StaticMemberExpression, UnaryExpression,
};
use oxc_ast_visit::{Visit, walk};
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::{BinaryOperator, UnaryOperator};
use vize_l0::{String, cstr, is_native_tag};
use vize_l1::markup::entity::{EntityContext, decode_one};
use vize_relief::{
    DirectiveNode, ElementNode, ExpressionNode, InterpolationNode, PropNode, TemplateChildNode,
};

macro_rules! meta {
    ($name:literal, $description:literal) => {
        RuleMeta {
            name: $name,
            description: $description,
            category: RuleCategory::Recommended,
            fixable: false,
            default_severity: Severity::Warning,
        }
    };
}

macro_rules! core_rule {
    ($type:ident, $meta:ident, $name:literal, $description:literal, $kind:ident) => {
        static $meta: RuleMeta = meta!($name, $description);
        #[doc = $description]
        pub struct $type;

        impl Rule for $type {
            fn meta(&self) -> &'static RuleMeta {
                &$meta
            }

            fn check_interpolation<'a>(
                &self,
                ctx: &mut LintContext<'a>,
                interpolation: &InterpolationNode<'a>,
            ) {
                check_expression(
                    ctx,
                    &interpolation.content,
                    CoreKind::$kind,
                    EntityContext::Text,
                );
            }

            fn check_directive<'a>(
                &self,
                ctx: &mut LintContext<'a>,
                _element: &ElementNode<'a>,
                directive: &DirectiveNode<'a>,
            ) {
                if let Some(expression) = &directive.exp {
                    check_expression(ctx, expression, CoreKind::$kind, EntityContext::Attribute);
                }
            }
        }
    };
}

core_rule!(
    Camelcase,
    CAMELCASE_META,
    "vue/camelcase",
    "Require camelCase identifiers in template expressions",
    Camelcase
);
core_rule!(
    DotNotation,
    DOT_NOTATION_META,
    "vue/dot-notation",
    "Prefer dot notation in template expressions",
    DotNotation
);
core_rule!(
    Eqeqeq,
    EQEQEQ_META,
    "vue/eqeqeq",
    "Require strict equality in template expressions",
    Eqeqeq
);
core_rule!(
    NoConsole,
    NO_CONSOLE_META,
    "vue/no-console",
    "Disallow console access in template expressions",
    NoConsole
);
core_rule!(
    NoConstantCondition,
    NO_CONSTANT_CONDITION_META,
    "vue/no-constant-condition",
    "Disallow constant conditions in template expressions",
    NoConstantCondition
);
core_rule!(
    NoEmptyPattern,
    NO_EMPTY_PATTERN_META,
    "vue/no-empty-pattern",
    "Disallow empty destructuring patterns in template expressions",
    NoEmptyPattern
);
core_rule!(
    NoImplicitCoercion,
    NO_IMPLICIT_COERCION_META,
    "vue/no-implicit-coercion",
    "Disallow implicit coercion in template expressions",
    NoImplicitCoercion
);
core_rule!(
    NoIrregularWhitespace,
    NO_IRREGULAR_WHITESPACE_META,
    "vue/no-irregular-whitespace",
    "Disallow irregular whitespace in template expressions",
    NoIrregularWhitespace
);
core_rule!(
    NoLossOfPrecision,
    NO_LOSS_OF_PRECISION_META,
    "vue/no-loss-of-precision",
    "Disallow numeric literals that lose precision",
    NoLossOfPrecision
);
core_rule!(
    NoNegatedCondition,
    NO_NEGATED_CONDITION_META,
    "vue/no-negated-condition",
    "Disallow negated ternary conditions in template expressions",
    NoNegatedCondition
);
core_rule!(
    NoSparseArrays,
    NO_SPARSE_ARRAYS_META,
    "vue/no-sparse-arrays",
    "Disallow sparse arrays in template expressions",
    NoSparseArrays
);
core_rule!(
    NoUselessConcat,
    NO_USELESS_CONCAT_META,
    "vue/no-useless-concat",
    "Disallow useless string concatenation in template expressions",
    NoUselessConcat
);
core_rule!(
    ObjectShorthand,
    OBJECT_SHORTHAND_META,
    "vue/object-shorthand",
    "Require object shorthand in template expressions",
    ObjectShorthand
);
core_rule!(
    PreferTemplate,
    PREFER_TEMPLATE_META,
    "vue/prefer-template",
    "Prefer template literals in template expressions",
    PreferTemplate
);
core_rule!(
    QuoteProps,
    QUOTE_PROPS_META,
    "vue/quote-props",
    "Disallow unnecessary quoted object properties in template expressions",
    QuoteProps
);

static NO_RESTRICTED_SYNTAX_META: RuleMeta = meta!(
    "vue/no-restricted-syntax",
    "Disallow configured syntax in template expressions"
);

/// Restrict selected OXC AST node names in template expressions.
#[derive(Default)]
pub struct NoRestrictedSyntax {
    selectors: Vec<String>,
}

impl NoRestrictedSyntax {
    /// Create a rule from simple ESTree node-type selectors.
    pub fn new(selectors: Vec<String>) -> Self {
        Self { selectors }
    }
}

impl Rule for NoRestrictedSyntax {
    fn meta(&self) -> &'static RuleMeta {
        &NO_RESTRICTED_SYNTAX_META
    }

    fn check_interpolation<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        interpolation: &InterpolationNode<'a>,
    ) {
        check_restricted_expression(
            ctx,
            &interpolation.content,
            &self.selectors,
            EntityContext::Text,
        );
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        _element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        if let Some(expression) = &directive.exp {
            check_restricted_expression(ctx, expression, &self.selectors, EntityContext::Attribute);
        }
    }
}

#[derive(Clone, Copy)]
enum CoreKind {
    Camelcase,
    DotNotation,
    Eqeqeq,
    NoConsole,
    NoConstantCondition,
    NoEmptyPattern,
    NoImplicitCoercion,
    NoIrregularWhitespace,
    NoLossOfPrecision,
    NoNegatedCondition,
    NoSparseArrays,
    NoUselessConcat,
    ObjectShorthand,
    PreferTemplate,
    QuoteProps,
}

impl CoreKind {
    fn message(self) -> &'static str {
        match self {
            Self::Camelcase => "Identifier is not in camel case",
            Self::DotNotation => "Use dot notation instead of a computed property",
            Self::Eqeqeq => "Use strict equality operators",
            Self::NoConsole => "Unexpected console access in a template expression",
            Self::NoConstantCondition => "Unexpected constant condition",
            Self::NoEmptyPattern => "Unexpected empty destructuring pattern",
            Self::NoImplicitCoercion => "Use an explicit type conversion",
            Self::NoIrregularWhitespace => "Unexpected irregular whitespace",
            Self::NoLossOfPrecision => "This numeric literal loses precision",
            Self::NoNegatedCondition => "Unexpected negated ternary condition",
            Self::NoSparseArrays => "Unexpected sparse array",
            Self::NoUselessConcat => "Unexpected concatenation of literal strings",
            Self::ObjectShorthand => "Use object property shorthand",
            Self::PreferTemplate => "Use a template literal instead of string concatenation",
            Self::QuoteProps => "Unnecessarily quoted object property",
        }
    }
}

fn check_expression(
    ctx: &mut LintContext<'_>,
    expression: &ExpressionNode<'_>,
    kind: CoreKind,
    entity_context: EntityContext,
) {
    let ExpressionNode::Simple(simple) = expression else {
        return;
    };
    let Some(origin) = Origin::new(ctx.source, simple.content, simple.loc.span, entity_context)
    else {
        return;
    };

    if matches!(kind, CoreKind::NoIrregularWhitespace) {
        for (index, character) in simple.content.char_indices() {
            if is_irregular_whitespace(character)
                && let Some(range) = origin.range(Span::new(
                    index as u32,
                    (index + character.len_utf8()) as u32,
                ))
            {
                ctx.warn_at_with_help(kind.message(), range, compatibility_help());
            }
        }
        return;
    }

    let Some(retained) = simple
        .js_ast
        .filter(|retained| retained.raw == simple.content)
    else {
        return;
    };
    let Some(retained) = retained.as_expression() else {
        return;
    };
    let mut visitor = CoreVisitor { ctx, origin, kind };
    visitor.visit_expression(retained.ast);
}

fn compatibility_help() -> &'static str {
    "Move this JavaScript expression to a script binding or rewrite it using the recommended syntax"
}

struct CoreVisitor<'ctx, 'source> {
    ctx: &'ctx mut LintContext<'source>,
    origin: Origin,
    kind: CoreKind,
}

impl CoreVisitor<'_, '_> {
    fn report(&mut self, span: Span) {
        if let Some(range) = self.origin.range(span) {
            self.ctx
                .warn_at_with_help(self.kind.message(), range, compatibility_help());
        }
    }
}

impl<'a> Visit<'a> for CoreVisitor<'_, '_> {
    fn visit_identifier_reference(&mut self, identifier: &IdentifierReference<'a>) {
        if matches!(self.kind, CoreKind::Camelcase) && has_internal_underscore(&identifier.name) {
            self.report(identifier.span);
        }
        walk::walk_identifier_reference(self, identifier);
    }

    fn visit_computed_member_expression(&mut self, member: &ComputedMemberExpression<'a>) {
        if matches!(self.kind, CoreKind::DotNotation)
            && member
                .static_property_name()
                .is_some_and(|property| is_identifier_name(property.as_str()))
        {
            self.report(member.expression.span());
        }
        walk::walk_computed_member_expression(self, member);
    }

    fn visit_static_member_expression(&mut self, member: &StaticMemberExpression<'a>) {
        if matches!(self.kind, CoreKind::NoConsole)
            && matches!(
                member.object.get_inner_expression(),
                Expression::Identifier(identifier) if identifier.name == "console"
            )
        {
            self.report(member.span);
        }
        walk::walk_static_member_expression(self, member);
    }

    fn visit_binary_expression(&mut self, binary: &BinaryExpression<'a>) {
        match self.kind {
            CoreKind::Eqeqeq
                if matches!(
                    binary.operator,
                    BinaryOperator::Equality | BinaryOperator::Inequality
                ) =>
            {
                self.report(binary.span);
            }
            CoreKind::NoImplicitCoercion
                if binary.operator == BinaryOperator::Addition
                    && (is_empty_string(&binary.left) || is_empty_string(&binary.right)) =>
            {
                self.report(binary.span);
            }
            CoreKind::NoUselessConcat
                if binary.operator == BinaryOperator::Addition
                    && is_string_literal(&binary.left)
                    && is_string_literal(&binary.right) =>
            {
                self.report(binary.span);
            }
            CoreKind::PreferTemplate
                if binary.operator == BinaryOperator::Addition
                    && (is_string_literal(&binary.left) || is_string_literal(&binary.right)) =>
            {
                self.report(binary.span);
            }
            _ => {}
        }
        walk::walk_binary_expression(self, binary);
    }

    fn visit_conditional_expression(&mut self, conditional: &ConditionalExpression<'a>) {
        match self.kind {
            CoreKind::NoConstantCondition if is_constant(&conditional.test) => {
                self.report(conditional.test.span());
            }
            CoreKind::NoNegatedCondition
                if matches!(
                    conditional.test.get_inner_expression(),
                    Expression::UnaryExpression(unary)
                        if unary.operator == UnaryOperator::LogicalNot
                ) =>
            {
                self.report(conditional.test.span());
            }
            _ => {}
        }
        walk::walk_conditional_expression(self, conditional);
    }

    fn visit_array_expression(&mut self, array: &ArrayExpression<'a>) {
        if matches!(self.kind, CoreKind::NoSparseArrays)
            && array
                .elements
                .iter()
                .any(|element| matches!(element, ArrayExpressionElement::Elision(_)))
        {
            self.report(array.span);
        }
        walk::walk_array_expression(self, array);
    }

    fn visit_object_pattern(&mut self, object: &ObjectPattern<'a>) {
        if matches!(self.kind, CoreKind::NoEmptyPattern) && object.is_empty() {
            self.report(object.span);
        }
        walk::walk_object_pattern(self, object);
    }

    fn visit_array_pattern(&mut self, array: &ArrayPattern<'a>) {
        if matches!(self.kind, CoreKind::NoEmptyPattern) && array.is_empty() {
            self.report(array.span);
        }
        walk::walk_array_pattern(self, array);
    }

    fn visit_unary_expression(&mut self, unary: &UnaryExpression<'a>) {
        if matches!(self.kind, CoreKind::NoImplicitCoercion)
            && (unary.operator == UnaryOperator::UnaryPlus
                || (unary.operator == UnaryOperator::LogicalNot
                    && matches!(
                        unary.argument.get_inner_expression(),
                        Expression::UnaryExpression(inner)
                            if inner.operator == UnaryOperator::LogicalNot
                    )))
        {
            self.report(unary.span);
        }
        walk::walk_unary_expression(self, unary);
    }

    fn visit_numeric_literal(&mut self, literal: &NumericLiteral<'a>) {
        if matches!(self.kind, CoreKind::NoLossOfPrecision)
            && numeric_literal_may_lose_precision(literal)
        {
            self.report(literal.span);
        }
        walk::walk_numeric_literal(self, literal);
    }

    fn visit_object_expression(&mut self, object: &ObjectExpression<'a>) {
        for entry in &object.properties {
            let ObjectPropertyKind::ObjectProperty(property) = entry else {
                continue;
            };
            match self.kind {
                CoreKind::ObjectShorthand
                    if !property.shorthand
                        && !property.method
                        && property_name(&property.key).is_some_and(|key| {
                            matches!(
                                property.value.get_inner_expression(),
                                Expression::Identifier(value) if value.name == key
                            )
                        }) =>
                {
                    self.report(property.span);
                }
                CoreKind::QuoteProps
                    if matches!(&property.key, PropertyKey::StringLiteral(literal)
                        if is_identifier_name(literal.value.as_str())) =>
                {
                    self.report(property.key.span());
                }
                _ => {}
            }
        }
        walk::walk_object_expression(self, object);
    }
}

fn has_internal_underscore(name: &str) -> bool {
    name.trim_matches('_').contains('_')
}

fn is_identifier_name(name: &str) -> bool {
    let mut chars = name.chars();
    chars
        .next()
        .is_some_and(|character| character == '_' || character == '$' || character.is_alphabetic())
        && chars
            .all(|character| character == '_' || character == '$' || character.is_alphanumeric())
        && !oxc_syntax::keyword::is_reserved_keyword(name)
}

fn is_empty_string(expression: &Expression<'_>) -> bool {
    matches!(expression.get_inner_expression(), Expression::StringLiteral(literal) if literal.value.is_empty())
}

fn is_string_literal(expression: &Expression<'_>) -> bool {
    match expression.get_inner_expression() {
        Expression::StringLiteral(_) => true,
        Expression::TemplateLiteral(template) => template.expressions.is_empty(),
        _ => false,
    }
}

fn is_constant(expression: &Expression<'_>) -> bool {
    matches!(
        expression.get_inner_expression(),
        Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::StringLiteral(_)
    )
}

fn numeric_literal_may_lose_precision(literal: &NumericLiteral<'_>) -> bool {
    let Some(raw) = literal.raw.as_ref().map(|raw| raw.as_str()) else {
        return false;
    };
    let significant_digits = raw
        .bytes()
        .filter(|byte| byte.is_ascii_digit())
        .skip_while(|byte| *byte == b'0')
        .count();
    significant_digits > 15
}

fn property_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::Identifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::StringLiteral(literal) => Some(literal.value.as_str()),
        _ => None,
    }
}

fn is_irregular_whitespace(character: char) -> bool {
    matches!(
        character,
        '\u{000B}' | '\u{000C}' | '\u{00A0}' | '\u{0085}' | '\u{1680}' | '\u{180E}' | '\u{2000}'
            ..='\u{200B}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{202F}'
                | '\u{205F}'
                | '\u{3000}'
                | '\u{FEFF}'
    )
}

fn check_restricted_expression(
    ctx: &mut LintContext<'_>,
    expression: &ExpressionNode<'_>,
    selectors: &[String],
    entity_context: EntityContext,
) {
    if selectors.is_empty() {
        return;
    }
    let ExpressionNode::Simple(simple) = expression else {
        return;
    };
    let Some(origin) = Origin::new(ctx.source, simple.content, simple.loc.span, entity_context)
    else {
        return;
    };
    let Some(retained) = simple
        .js_ast
        .filter(|retained| retained.raw == simple.content)
        .and_then(|retained| retained.as_expression())
    else {
        return;
    };
    let mut visitor = RestrictedVisitor {
        ctx,
        origin,
        selectors,
    };
    visitor.visit_expression(retained.ast);
}

struct RestrictedVisitor<'ctx, 'source, 'selectors> {
    ctx: &'ctx mut LintContext<'source>,
    origin: Origin,
    selectors: &'selectors [String],
}

impl RestrictedVisitor<'_, '_, '_> {
    fn selected(&self, node_type: &str) -> bool {
        self.selectors.iter().any(|selector| selector == node_type)
    }

    fn report(&mut self, node_type: &str, span: Span) {
        if self.selected(node_type)
            && let Some(range) = self.origin.range(span)
        {
            self.ctx.warn_at_with_help(
                cstr!(
                    "Use of `{}` is restricted in template expressions",
                    node_type
                ),
                range,
                "Rewrite the expression without the restricted syntax",
            );
        }
    }
}

impl<'a> Visit<'a> for RestrictedVisitor<'_, '_, '_> {
    fn visit_call_expression(&mut self, call: &oxc_ast::ast::CallExpression<'a>) {
        self.report("CallExpression", call.span);
        walk::walk_call_expression(self, call);
    }

    fn visit_binary_expression(&mut self, binary: &BinaryExpression<'a>) {
        self.report("BinaryExpression", binary.span);
        walk::walk_binary_expression(self, binary);
    }

    fn visit_conditional_expression(&mut self, conditional: &ConditionalExpression<'a>) {
        self.report("ConditionalExpression", conditional.span);
        walk::walk_conditional_expression(self, conditional);
    }

    fn visit_object_expression(&mut self, object: &ObjectExpression<'a>) {
        self.report("ObjectExpression", object.span);
        walk::walk_object_expression(self, object);
    }

    fn visit_array_expression(&mut self, array: &ArrayExpression<'a>) {
        self.report("ArrayExpression", array.span);
        walk::walk_array_expression(self, array);
    }

    fn visit_unary_expression(&mut self, unary: &UnaryExpression<'a>) {
        self.report("UnaryExpression", unary.span);
        walk::walk_unary_expression(self, unary);
    }
}

static NO_LITERALS_IN_TEMPLATE_META: RuleMeta = meta!(
    "vue/no-literals-in-template",
    "Disallow object, array and function literals in v-bind"
);

/// `vue/no-literals-in-template`.
pub struct NoLiteralsInTemplate;

impl Rule for NoLiteralsInTemplate {
    fn meta(&self) -> &'static RuleMeta {
        &NO_LITERALS_IN_TEMPLATE_META
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        _element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        if directive.name != "bind"
            || static_argument(directive)
                .is_some_and(|argument| matches!(argument, "class" | "style"))
        {
            return;
        }
        let Some(ExpressionNode::Simple(expression)) = &directive.exp else {
            return;
        };
        let Some(parsed) = expression.js_ast.and_then(|ast| ast.as_expression()) else {
            return;
        };
        if matches!(
            parsed.ast.get_inner_expression(),
            Expression::ObjectExpression(_)
                | Expression::ArrayExpression(_)
                | Expression::FunctionExpression(_)
                | Expression::ArrowFunctionExpression(_)
        ) {
            ctx.warn_with_help(
                "Unexpected literal in a template binding",
                &expression.loc,
                "Move the literal to a script binding so Vue can reuse the same value",
            );
        }
    }
}

static NO_RESTRICTED_V_BIND_META: RuleMeta = meta!(
    "vue/no-restricted-v-bind",
    "Disallow restricted v-bind arguments"
);

/// `vue/no-restricted-v-bind`; the default rejects `:v-*` attributes.
pub struct NoRestrictedVBind;

impl Rule for NoRestrictedVBind {
    fn meta(&self) -> &'static RuleMeta {
        &NO_RESTRICTED_V_BIND_META
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        _element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        if directive.name == "bind"
            && static_argument(directive).is_some_and(|argument| argument.starts_with("v-"))
        {
            ctx.warn_with_help(
                "Using `:v-*` is not allowed",
                &directive.loc,
                "Remove `:` and use the attribute as a directive",
            );
        }
    }
}

static PREFER_V_MODEL_META: RuleMeta = meta!(
    "vue/prefer-v-model",
    "Prefer v-model over matching bind and update listeners"
);

/// `vue/prefer-v-model`.
pub struct PreferVModel;

impl Rule for PreferVModel {
    fn meta(&self) -> &'static RuleMeta {
        &PREFER_V_MODEL_META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        if is_native_tag(element.tag) {
            return;
        }
        for bind in element.props.iter().filter_map(|prop| match prop {
            PropNode::Directive(directive)
                if directive.name == "bind" && directive.modifiers.is_empty() =>
            {
                Some(directive)
            }
            _ => None,
        }) {
            let Some(prop_name) = static_argument(bind) else {
                continue;
            };
            let Some(bind_expression) = simple_expression(bind.exp.as_ref()) else {
                continue;
            };
            let update_name = cstr!("update:{}", prop_name);
            let matching = element.props.iter().find_map(|prop| match prop {
                PropNode::Directive(directive)
                    if directive.name == "on"
                        && directive.modifiers.is_empty()
                        && static_argument(directive) == Some(update_name.as_str()) =>
                {
                    Some(directive)
                }
                _ => None,
            });
            let Some(handler) =
                matching.and_then(|directive| simple_expression(directive.exp.as_ref()))
            else {
                continue;
            };
            if mirrors_assignment(bind_expression, handler) {
                ctx.warn_with_help(
                    cstr!(
                        "Prefer `v-model` over `:{}` and `@update:{}`",
                        prop_name,
                        prop_name
                    ),
                    &bind.loc,
                    "Replace the matching bind and update listener with v-model",
                );
            }
        }
    }
}

static V_IF_ELSE_KEY_META: RuleMeta = meta!(
    "vue/v-if-else-key",
    "Require keys on repeated conditional components"
);

/// `vue/v-if-else-key`.
pub struct VIfElseKey;

impl Rule for VIfElseKey {
    fn meta(&self) -> &'static RuleMeta {
        &V_IF_ELSE_KEY_META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        check_conditional_siblings(ctx, &element.children);
    }
}

fn static_argument<'a>(directive: &'a DirectiveNode<'a>) -> Option<&'a str> {
    match &directive.arg {
        Some(ExpressionNode::Simple(argument)) if argument.is_static => Some(argument.content),
        _ => None,
    }
}

fn simple_expression<'a>(expression: Option<&'a ExpressionNode<'a>>) -> Option<&'a str> {
    match expression {
        Some(ExpressionNode::Simple(expression)) => Some(expression.content),
        _ => None,
    }
}

fn mirrors_assignment(binding: &str, handler: &str) -> bool {
    let binding = compact_js(binding);
    let handler = compact_js(handler);
    handler == cstr!("{}=$event", binding)
        || handler.split_once("=>").is_some_and(|(parameter, body)| {
            let parameter = parameter.trim_matches(['(', ')']);
            body == cstr!("{}={}", binding, parameter)
        })
}

fn compact_js(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn has_key(element: &ElementNode<'_>) -> bool {
    element.props.iter().any(|prop| match prop {
        PropNode::Attribute(attribute) => attribute.name == "key",
        PropNode::Directive(directive) => {
            directive.name == "bind" && static_argument(directive) == Some("key")
        }
    })
}

fn check_conditional_siblings(ctx: &mut LintContext<'_>, children: &[TemplateChildNode<'_>]) {
    let elements: Vec<_> = children
        .iter()
        .filter_map(|child| match child {
            TemplateChildNode::Element(element)
                if !is_native_tag(element.tag)
                    && !matches!(element.tag, "component" | "slot" | "template")
                    && has_conditional_directive(element) =>
            {
                Some(element.as_ref())
            }
            _ => None,
        })
        .collect();
    for element in elements.iter().filter(|element| !has_key(element)) {
        if elements
            .iter()
            .filter(|sibling| sibling.tag == element.tag)
            .count()
            > 1
        {
            ctx.warn_with_help(
                cstr!(
                    "Conditionally rendered repeated component `{}` should have a key",
                    element.tag
                ),
                &element.loc,
                "Add a unique `key` or `:key` to each conditional branch",
            );
        }
    }
}

fn has_conditional_directive(element: &ElementNode<'_>) -> bool {
    element.props.iter().any(|prop| {
        matches!(prop, PropNode::Directive(directive)
            if matches!(directive.name, "if" | "else-if" | "else"))
    })
}

pub(crate) fn rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(Camelcase),
        Box::new(DotNotation),
        Box::new(Eqeqeq),
        Box::new(NoConsole),
        Box::new(NoConstantCondition),
        Box::new(NoEmptyPattern),
        Box::new(NoImplicitCoercion),
        Box::new(NoIrregularWhitespace),
        Box::new(NoLossOfPrecision),
        Box::new(NoNegatedCondition),
        Box::new(NoSparseArrays),
        Box::new(NoUselessConcat),
        Box::new(ObjectShorthand),
        Box::new(PreferTemplate),
        Box::new(QuoteProps),
        Box::new(NoRestrictedSyntax::default()),
        Box::new(NoLiteralsInTemplate),
        Box::new(NoRestrictedVBind),
        Box::new(PreferVModel),
        Box::new(VIfElseKey),
    ]
}

struct Origin {
    start: u32,
    offsets: Option<Vec<u32>>,
}

impl Origin {
    fn new(
        source: &str,
        content: &str,
        span: vize_l0::Span,
        context: EntityContext,
    ) -> Option<Self> {
        let authored = source.get(span.start as usize..span.end as usize)?;
        if authored == content {
            return Some(Self {
                start: span.start,
                offsets: None,
            });
        }
        let mut decoded = String::default();
        let mut offsets = vec![0];
        let mut at = 0;
        while let Some(rest) = authored.get(at..)
            && let Some(first) = rest.chars().next()
        {
            let length = if let Some((value, length)) = decode_one(rest.as_bytes(), context) {
                value.for_each(|character| {
                    decoded.push(character);
                    offsets.extend(std::iter::repeat_n(at as u32, character.len_utf8()));
                });
                offsets.pop();
                length
            } else {
                decoded.push(first);
                offsets.extend(std::iter::repeat_n(at as u32, first.len_utf8() - 1));
                first.len_utf8()
            };
            at += length;
            offsets.push(at as u32);
        }
        (decoded == content).then_some(Self {
            start: span.start,
            offsets: Some(offsets),
        })
    }

    fn range(&self, span: Span) -> Option<ByteRange> {
        let offset = |position: u32| match &self.offsets {
            Some(offsets) => offsets.get(position as usize).copied(),
            None => Some(position),
        };
        Some(ByteRange {
            start: self.start.checked_add(offset(span.start)?)?,
            end: self.start.checked_add(offset(span.end)?)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linter::{LintResult, Linter};
    use crate::rule::RuleRegistry;

    fn lint(rule: Box<dyn Rule>, template: &str) -> LintResult {
        let mut registry = RuleRegistry::new();
        registry.register(rule);
        Linter::with_registry(registry).lint_template(template, "Component.vue")
    }

    #[test]
    fn core_rules_read_retained_expression_ast() {
        let eqeqeq = lint(Box::new(Eqeqeq), "<p>{{ value == null }}</p>");
        assert_eq!(eqeqeq.warning_count, 1);
        let sparse = lint(Box::new(NoSparseArrays), "<p :data=\"[1,,2]\" />");
        assert_eq!(sparse.warning_count, 1);
        let console = lint(Box::new(NoConsole), "<p>{{ console.log(value) }}</p>");
        assert_eq!(console.warning_count, 1);
    }

    #[test]
    fn template_specific_rules_report_compatibility_cases() {
        let literals = lint(
            Box::new(NoLiteralsInTemplate),
            "<Widget :options=\"{ eager: true }\" />",
        );
        assert_eq!(literals.warning_count, 1);
        let restricted = lint(Box::new(NoRestrictedVBind), "<div :v-test=\"value\" />");
        assert_eq!(restricted.warning_count, 1);
        let model = lint(
            Box::new(PreferVModel),
            "<Widget :model-value=\"value\" @update:model-value=\"value = $event\" />",
        );
        assert_eq!(model.warning_count, 1);
    }

    #[test]
    fn repeated_conditional_components_require_keys() {
        let result = lint(
            Box::new(VIfElseKey),
            "<section><Widget v-if=\"first\" /><Widget v-else /></section>",
        );
        assert_eq!(result.warning_count, 2);
    }
}
