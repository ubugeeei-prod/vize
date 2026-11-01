//! Direct inline-function classification for generic component props.

use oxc_allocator::Allocator;
use oxc_ast::ast::{ArrowFunctionExpression, Expression, Function};
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::SourceType;
use oxc_syntax::scope::ScopeFlags;
use vize_carton::FxHashMap;
use vize_relief::{ExpressionNode, PropNode, RootNode, TemplateChildNode};

use super::expression_scanner::{has_top_level_comma, skip_js_trivia};
use super::handler_shape::inline_callback_event_argument;

/// Syntax facts for the opt-in legacy external-component route. Reuse the
/// retained expression AST once per lookup domain; modern generation never
/// walks this table. Equal expression contents have equal callback syntax.
#[derive(Default)]
pub(super) struct LegacyInlineCallbacks<'template> {
    values: FxHashMap<&'template str, bool>,
}

impl<'template> LegacyInlineCallbacks<'template> {
    pub(super) fn from_root(root: Option<&RootNode<'template>>, enabled: bool) -> Self {
        let mut facts = Self::default();
        if enabled && let Some(root) = root {
            facts.collect(&root.children);
        }
        facts
    }

    pub(super) fn contains(&self, value: &str) -> bool {
        let value = value.trim();
        if !may_contain_function(value) {
            return false;
        }
        self.values.get(value).copied().unwrap_or_else(|| {
            if !vize_l0::expression_guard::expression_is_safe_to_parse(value) {
                return super::component_prop_checker::contains_inline_function_prop_value(value);
            }
            let allocator = Allocator::new();
            match Parser::new(&allocator, value, SourceType::ts()).parse_expression() {
                Ok(expression) => contains_function(&expression),
                // Keep the preexisting recovery behavior; this classifier
                // must not grant a new exemption to invalid expressions.
                Err(_) => super::component_prop_checker::contains_inline_function_prop_value(value),
            }
        })
    }

    fn collect(&mut self, children: &[TemplateChildNode<'template>]) {
        for child in children {
            match child {
                TemplateChildNode::Element(element) => {
                    for prop in &element.props {
                        let PropNode::Directive(directive) = prop else {
                            continue;
                        };
                        if !matches!(directive.name, "bind" | "model") {
                            continue;
                        }
                        let Some(ExpressionNode::Simple(simple)) = directive.exp.as_ref() else {
                            continue;
                        };
                        let value = simple.content.trim();
                        if !may_contain_function(value) || self.values.contains_key(value) {
                            continue;
                        }
                        let callback = simple
                            .js_ast
                            .and_then(|ast| ast.as_expression())
                            .filter(|retained| retained.raw == simple.content)
                            .map(|retained| contains_function(retained.ast))
                            .unwrap_or_else(|| {
                                super::component_prop_checker::contains_inline_function_prop_value(
                                    value,
                                )
                            });
                        self.values.insert(value, callback);
                    }
                    self.collect(&element.children);
                }
                TemplateChildNode::If(node) => {
                    for branch in &node.branches {
                        self.collect(&branch.children);
                    }
                }
                TemplateChildNode::For(node) => self.collect(&node.children),
                _ => {}
            }
        }
    }
}

fn may_contain_function(value: &str) -> bool {
    value.contains('(') || value.contains("=>") || value.contains("function")
}

fn contains_function(expression: &Expression<'_>) -> bool {
    let mut visitor = InlineFunctions(false);
    visitor.visit_expression(expression);
    visitor.0
}

struct InlineFunctions(bool);
impl<'a> Visit<'a> for InlineFunctions {
    fn visit_function(&mut self, _: &Function<'a>, _: ScopeFlags) {
        self.0 = true;
    }
    fn visit_arrow_function_expression(&mut self, _: &ArrowFunctionExpression<'a>) {
        self.0 = true;
    }
}

/// Whether the value itself is a callback, rather than an expression that only
/// contains one (for example `items.map((item) => item.id)`).
pub(crate) fn is_direct_inline_function_prop_value(value: &str) -> bool {
    inline_callback_event_argument(value).is_some() || parsed_direct_inline_function(value)
}

/// Fall back to the TypeScript parser only for callback-shaped values the
/// allocation-free scanner cannot settle. JavaScript's regex/division lexical
/// goal depends on full grammar context, so duplicating it here would make
/// rare valid defaults less reliable than the parser already used by Canon.
fn parsed_direct_inline_function(value: &str) -> bool {
    let value = value.trim();
    if !value.contains("=>") || !looks_like_inline_callback_start(value) {
        return false;
    }
    let allocator = Allocator::new();
    let Ok(expression) = Parser::new(&allocator, value, SourceType::ts()).parse_expression() else {
        return false;
    };
    expression_is_inline_function(&expression)
}

fn looks_like_inline_callback_start(value: &str) -> bool {
    if value.starts_with(['(', '<'])
        || value.starts_with("async")
        || value.contains('?')
        || has_top_level_comma(value)
    {
        return true;
    }
    let mut end = 0;
    for (index, ch) in value.char_indices() {
        if index == 0 {
            if !(ch == '_' || ch == '$' || ch.is_alphabetic()) {
                return false;
            }
        } else if !(ch == '_' || ch == '$' || ch.is_alphanumeric()) {
            break;
        }
        end = index + ch.len_utf8();
    }
    let after_identifier = skip_js_trivia(value, end);
    value
        .get(after_identifier..)
        .is_some_and(|rest| rest.starts_with("=>"))
}

fn expression_is_inline_function(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_) => true,
        Expression::ParenthesizedExpression(parenthesized) => {
            expression_is_inline_function(&parenthesized.expression)
        }
        Expression::TSAsExpression(assertion) => {
            expression_is_inline_function(&assertion.expression)
        }
        Expression::TSSatisfiesExpression(assertion) => {
            expression_is_inline_function(&assertion.expression)
        }
        Expression::TSTypeAssertion(assertion) => {
            expression_is_inline_function(&assertion.expression)
        }
        Expression::TSNonNullExpression(non_null) => {
            expression_is_inline_function(&non_null.expression)
        }
        Expression::ConditionalExpression(conditional) => {
            expression_is_inline_function(&conditional.consequent)
                && expression_is_inline_function(&conditional.alternate)
        }
        Expression::SequenceExpression(sequence) => sequence
            .expressions
            .last()
            .is_some_and(expression_is_inline_function),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::{LegacyInlineCallbacks, is_direct_inline_function_prop_value};

    #[test]
    fn distinguishes_direct_callbacks_from_nested_callbacks() {
        let direct = [
            "(fn: (x: string) => string) => fn('x')",
            r"(pattern = /* default */ /\)/, value) /* callback */ => value",
            "(pattern = foo++ / bar, value) => value",
            "(pattern = foo! / bar, value) => value",
            r"(fn = () => { if (ok) /\)/.test(value) }, value) => value",
            "value /* callback */ => value",
            "ok ? (value) => value : (value) => value",
            "(0, (value) => value)",
            "void 0, (value) => value",
        ];
        for value in direct {
            assert!(
                is_direct_inline_function_prop_value(value),
                "expected a direct callback: {value}"
            );
        }

        let nested = [
            "jobs.map((job) => job.id)",
            "[() => true]",
            "{ apply: (value) => value }",
            "ok ? (value) => value : fallback",
            "(0, jobs.map((job) => job.id))",
            "void 0, jobs.map((job) => job.id)",
            "/=>/.test(value)",
        ];
        for value in nested {
            assert!(
                !is_direct_inline_function_prop_value(value),
                "expected only a nested callback: {value}"
            );
        }
    }
    #[test]
    fn legacy_callback_syntax_uses_functions_instead_of_text_markers() {
        let facts = LegacyInlineCallbacks::default();
        let values = [
            "{ format(value) { return !!value } }",
            "class { format(value) { return value } }",
            "[(value) => !!value]",
            "{ get format() { return 1 } }",
            "'a=>b'",
            "/=>/.test(value)",
            "items.map(format)",
            "{ format: 1 }",
            "{ format(value) {",
        ];
        assert_eq!(
            values.map(|value| facts.contains(value)),
            [true, true, true, true, false, false, false, false, false]
        );
    }

    #[test]
    fn legacy_facts_reuse_retained_expressions() {
        let allocator = vize_carton::Allocator::new();
        let (root, errors) = vize_armature::parse(
            &allocator,
            r#"<Comp :method="{ format(value) { return !!value } }" :count="'a=>b'" />"#,
        );
        assert_eq!(errors.len(), 0);
        let facts = LegacyInlineCallbacks::from_root(Some(&root), true);
        assert_eq!(facts.values.len(), 2);
        assert_eq!(
            [
                facts.contains("{ format(value) { return !!value } }"),
                facts.contains("'a=>b'")
            ],
            [true, false]
        );
        let disabled = LegacyInlineCallbacks::from_root(Some(&root), false);
        assert_eq!(disabled.values.len(), 0);
    }
}
