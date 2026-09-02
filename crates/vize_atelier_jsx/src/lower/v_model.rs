//! `v-model` attribute lowering: the three spellings `@vue/babel-plugin-jsx`
//! accepts, and the write-back target validation they share.
//!
//! JSX attribute names cannot contain `.`, so `@vue/babel-plugin-jsx` encodes
//! v-model's arg and modifiers two ways, both of which Vize accepts:
//!
//! - underscore suffixes — `v-model_lazy={x}`, `v-model:foo_trim={x}`
//! - an array value — `v-model={[x, 'arg', ['trim']]}`
//!
//! Split out of `attr.rs` so the generic attribute classification stays readable
//! and both files stay inside the per-file source budget.

use oxc_ast::ast::{
    ArrayExpressionElement, Expression, JSXAttribute, JSXAttributeName, JSXAttributeValue,
};
use oxc_span::GetSpan;
use vize_relief::SourceLocation;
use vize_relief::{DirectiveNode, PropNode, SimpleExpressionNode};
use vize_s0::ToCompactString;

use super::Lowerer;
use crate::diagnostics::JsxDiagnostic;

/// Result of parsing babel-plugin-jsx's positional `v-model` array form.
pub(crate) enum ModelArrayLowering<'a> {
    /// The array was lowered into a model directive.
    Lowered(PropNode<'a>),
    /// The array shape was not recognizable and may use the generic fallback.
    Unrecognized,
    /// The array was recognized but rejected with a diagnostic.
    Rejected,
}

impl<'a, 'm, 's: 'a> Lowerer<'a, 'm, 's> {
    /// Reject a `v-model` whose bound target cannot be assigned to.
    ///
    /// `v-model` compiles to `$event => (target = $event)`, so a target that is
    /// not a *place* expression — an object literal, a call, a literal — makes
    /// the emitted module fail to parse. `@vue/babel-plugin-jsx` rejects the same
    /// inputs (`Property left of AssignmentExpression expected node to be of a
    /// type ["LVal","OptionalMemberExpression"]`), so this is the compatible
    /// behavior as well as the correct one (#3420).
    ///
    /// Returns `true` when a diagnostic was reported and the attribute must be
    /// dropped. A `v-model` with no value at all is left alone: the core
    /// transform already reports "v-model is missing expression."
    pub(crate) fn reject_unassignable_model_target(&mut self, attr: &JSXAttribute<'_>) -> bool {
        let Some(target) = self.model_target_expr(attr) else {
            return false;
        };
        if is_assignable_target(target) {
            return false;
        }
        let span = target.span();
        let source = self.mapper().slice(span);
        self.report(JsxDiagnostic::error(
            format_args!(
                "v-model target `{source}` cannot be assigned to; \
                 v-model needs a variable or property reference."
            )
            .to_compact_string(),
            span.start,
            span.end,
        ));
        true
    }

    /// The expression a `v-model` attribute writes back to, for any of the three
    /// spellings: `v-model={x}`, `v-model_lazy={x}`, and the array form
    /// `v-model={[x, 'arg', ['mod']]}` (whose first element is the target).
    ///
    /// `None` for attributes that are not `v-model`, and for a `v-model` with no
    /// expression value — those are handled elsewhere.
    fn model_target_expr<'e>(&self, attr: &'e JSXAttribute<'e>) -> Option<&'e Expression<'e>> {
        let raw_name = match &attr.name {
            JSXAttributeName::NamespacedName(named) => self
                .mapper()
                .slice(named.namespace.span())
                .strip_prefix("v-")?,
            JSXAttributeName::Identifier(id) => {
                self.mapper().slice(id.span()).strip_prefix("v-")?
            }
        };
        let base = match split_underscore_model_modifiers(raw_name) {
            Some((name, _)) => name,
            None => raw_name,
        };
        if base != "model" {
            return None;
        }

        let JSXAttributeValue::ExpressionContainer(container) = attr.value.as_ref()? else {
            return None;
        };
        match &container.expression {
            // The array form's first element is the target; the rest are the arg
            // and modifiers, which are not assigned to.
            oxc_ast::ast::JSXExpression::ArrayExpression(array) => {
                array.elements.first().and_then(|e| e.as_expression())
            }
            expression => expression.as_expression(),
        }
    }

    /// The `[value, ...]` array literal backing a `v-model={[...]}` attribute,
    /// or `None` when the value is not an array-literal expression container.
    pub(crate) fn array_literal_value<'e>(
        &self,
        value: Option<&'e JSXAttributeValue<'e>>,
    ) -> Option<&'e oxc_ast::ast::ArrayExpression<'e>> {
        let JSXAttributeValue::ExpressionContainer(container) = value? else {
            return None;
        };
        match &container.expression {
            oxc_ast::ast::JSXExpression::ArrayExpression(array) => Some(array),
            _ => None,
        }
    }

    /// Lower a `v-model={[value, argString?, modifiersArray?]}` array into a
    /// `model` directive. Layout (per babel-plugin-jsx):
    ///   - element 0: the model expression (required)  -> `exp`
    ///   - a trailing array-literal element: modifiers  -> `directive.modifiers`
    ///   - an intermediate element: arg  -> `directive.arg`
    ///
    /// Returns [`ModelArrayLowering::Unrecognized`] for shapes we cannot
    /// confidently destructure (e.g. empty array, spread/hole elements).
    ///
    /// A *dynamic* argument (anything but a string literal) needs computed prop
    /// and update-listener keys, which only components support, so it is
    /// accepted when `allow_dynamic_arg` says the caller is lowering a component
    /// in opt-in Babel VDOM mode and rejected everywhere else rather than being
    /// silently discarded (#3466).
    pub(crate) fn lower_model_array(
        &mut self,
        array: &oxc_ast::ast::ArrayExpression<'_>,
        loc: &SourceLocation,
        allow_dynamic_arg: bool,
    ) -> ModelArrayLowering<'a> {
        // Collect the plain (non-hole, non-spread) expression elements.
        let mut elems: std::vec::Vec<&Expression<'_>> = std::vec::Vec::new();
        for element in &array.elements {
            match element {
                ArrayExpressionElement::SpreadElement(_) | ArrayExpressionElement::Elision(_) => {
                    return ModelArrayLowering::Unrecognized;
                }
                _ => match element.as_expression() {
                    Some(expr) => elems.push(expr),
                    None => return ModelArrayLowering::Unrecognized,
                },
            }
        }

        let Some(value_expr) = elems.first().copied() else {
            return ModelArrayLowering::Unrecognized;
        };

        // A trailing array-literal element is the modifiers list. Its index marks
        // the boundary so a middle string element can be recognized as the arg.
        let modifiers_idx = match elems.last() {
            Some(Expression::ArrayExpression(_)) if elems.len() >= 2 => Some(elems.len() - 1),
            _ => None,
        };

        // An intermediate element (index 1, before any modifiers array) is the
        // arg: the bound prop name for component v-model.
        let arg = if modifiers_idx == Some(1) {
            None
        } else {
            match elems.get(1).copied() {
                Some(expression @ Expression::StringLiteral(_)) => Some(expression),
                Some(expression) if allow_dynamic_arg => Some(expression),
                Some(expression) => {
                    let span = expression.span();
                    let source = self.mapper().slice(span);
                    self.reject_at(
                        span,
                        format_args!(
                            "v-model argument `{source}` must be a string literal; dynamic \
                             arguments are not supported."
                        ),
                    );
                    return ModelArrayLowering::Rejected;
                }
                None => None,
            }
        };

        let mut directive = DirectiveNode::new(self.bump(), "model", loc.clone());
        directive.exp = Some(self.dyn_expr(value_expr.span()));
        if let Some(argument) = arg {
            directive.arg = Some(match argument {
                Expression::StringLiteral(string) => {
                    self.static_expr(self.bump().alloc_str(string.value.as_str()), string.span)
                }
                expression => self.dyn_expr(expression.span()),
            });
        }

        if let Some(idx) = modifiers_idx
            && let Expression::ArrayExpression(modifiers) = elems[idx]
        {
            for element in &modifiers.elements {
                let Some(Expression::StringLiteral(s)) = element.as_expression() else {
                    continue;
                };
                directive.modifiers.push(SimpleExpressionNode::new(
                    self.bump().alloc_str(s.value.as_str()),
                    false,
                    loc.clone(),
                ));
            }
        }

        ModelArrayLowering::Lowered(PropNode::Directive(self.boxed(directive)))
    }
}

/// Whether an expression can appear on the left of an assignment.
///
/// `v-model` writes back with `target = $event`, so anything that is not a
/// *place* expression would emit a module that does not parse. The accepted set
/// mirrors `@vue/babel-plugin-jsx`'s (`LVal` plus optional member access):
/// identifiers, member access of any form, and private fields. Type-only
/// wrappers (`x as T`, `x!`, `<T>x`, `x satisfies T`) and parentheses are
/// transparent — they are stripped by codegen, so the underlying target decides.
///
/// Note the deliberate omission of array/object destructuring patterns: they are
/// valid `LVal`s in general JavaScript, but `v-model` assigns a single event
/// value, so destructuring one is never meaningful here.
pub(crate) fn is_assignable_target(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::Identifier(_)
        | Expression::StaticMemberExpression(_)
        | Expression::ComputedMemberExpression(_)
        | Expression::PrivateFieldExpression(_) => true,
        Expression::ParenthesizedExpression(inner) => is_assignable_target(&inner.expression),
        Expression::TSAsExpression(inner) => is_assignable_target(&inner.expression),
        Expression::TSSatisfiesExpression(inner) => is_assignable_target(&inner.expression),
        Expression::TSNonNullExpression(inner) => is_assignable_target(&inner.expression),
        Expression::TSTypeAssertion(inner) => is_assignable_target(&inner.expression),
        // `a?.b = x` is not legal JavaScript, but babel accepts an
        // OptionalMemberExpression as a v-model target, so matching it here keeps
        // the diagnostic from firing where babel stays quiet.
        Expression::ChainExpression(_) => true,
        _ => false,
    }
}

/// Split a babel-plugin-jsx `v-model_<mod>(_<mod>)*` attribute name (already
/// stripped of its `v-` prefix) into `("model", [modifiers...])`.
///
/// JSX attribute names cannot contain `.`, so babel-plugin-jsx encodes v-model
/// modifiers as `_`-joined suffixes: `model_lazy`, `model_number_lazy`. The
/// recognized standard modifiers are `lazy` / `trim` / `number`; any further
/// `_`-separated segments are passed through verbatim (custom modifiers).
///
/// Returns `None` for names that are not `model` with at least one suffix (so
/// bare `model` and unrelated directives keep their default behavior).
pub(crate) fn split_underscore_model_modifiers(
    name: &str,
) -> Option<(&'static str, std::vec::Vec<&str>)> {
    let rest = name.strip_prefix("model_")?;
    if rest.is_empty() {
        return None;
    }
    let mods: std::vec::Vec<&str> = rest.split('_').filter(|s| !s.is_empty()).collect();
    if mods.is_empty() {
        return None;
    }
    Some(("model", mods))
}

/// Split Babel's namespaced argument/modifier spelling:
/// `v-model:argument_trim` -> `("argument", ["trim"])`.
pub(crate) fn split_model_arg_modifiers(name: &str) -> Option<(&str, std::vec::Vec<&str>)> {
    let (arg, rest) = name.split_once('_')?;
    if arg.is_empty() || rest.is_empty() {
        return None;
    }
    let modifiers: std::vec::Vec<_> = rest.split('_').collect();
    if modifiers.iter().any(|modifier| modifier.is_empty()) {
        return None;
    }
    Some((arg, modifiers))
}
