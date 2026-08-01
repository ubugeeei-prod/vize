//! `mixins:` / `extends:` members incorporated into the Options API `__VizeThis`.
//!
//! Without this, `__VizeThis` only lists the component's own
//! `props`/`data`/`computed`/`methods`/`inject`/`setup` names, so a
//! `methods`/`computed` body that legitimately calls a member contributed by a
//! mixin reports `TS2339 Property 'x' does not exist on type '__VizeThis'` on
//! code `vue-tsc` accepts (#3609).
//!
//! Only a *component constructor* contributes. `__VizeInheritedInstance`
//! yields `any` for anything that is not `abstract new (...args) => I`, and
//! `__VizeInheritedMembers` maps that `any` back to the empty object through
//! the `__VizeIsAny` guard. That is what keeps the untyped legacy surface
//! exactly where it is: a plain-JavaScript mixin is an options *object*
//! (`{ methods: { … } }`) or degrades to `any` outright, so it contributes no
//! keys and every diagnostic it used to raise still raises. Intersecting the
//! raw type instead would collapse `__VizeThis` to `any` for an untyped mixin
//! and silently drop every genuine finding in the file.
//!
//! The neutral element is a named empty interface on purpose. TypeScript drops
//! `{}` / `unknown` while building the intersection and then prints the
//! expanded own-member object in diagnostics. Keeping a named neutral member
//! preserves the `__VizeThis` alias and the pinned legacy messages.

use std::ops::Range;

use oxc_ast::ast::{Expression, ObjectExpression};
use oxc_span::GetSpan;

use super::super::options_api::option_expression_property;
use super::super::options_api_support::is_safe_value_identifier;
use vize_carton::{String, append};

/// Type-level helpers backing the inherited-member aliases.
pub(super) const INHERITED_MEMBER_HELPERS: &str = concat!(
    "  type __VizeInheritedInstance<T> = T extends abstract new (...args: any[]) => infer __I ? __I : any;\n",
    "  interface __VizeNoInheritedMembers {}\n",
    "  type __VizeInheritedMembers<T, __I = __VizeInheritedInstance<T>> = [__VizeIsAny<__I>] extends [true] ? __VizeNoInheritedMembers : __I;\n",
);

/// One `mixins:` element (or the `extends:` value) whose instance members
/// `__VizeThis` inherits.
#[derive(Debug)]
pub(super) struct InheritedComponent {
    /// The authored reference, reproduced verbatim as a `typeof` operand.
    pub(super) reference: String,
    /// Script-relative range of the authored expression it was derived from.
    pub(super) src: Range<usize>,
}

/// Collect every `mixins:` element and the `extends:` value that can be named
/// by a `typeof` query.
///
/// An inline options object (`mixins: [{ methods: { … } }]`) is skipped here:
/// it has no name to query, and Croquis already merges its local members into
/// the direct `__VizeThis` shape.
pub(super) fn collect_inherited_components(
    options: &ObjectExpression<'_>,
    inherited: &mut Vec<InheritedComponent>,
) {
    if let Some(mixins) = option_expression_property(options, "mixins")
        && let Expression::ArrayExpression(array) = unwrap_expression(mixins)
    {
        for element in &array.elements {
            if let Some(expression) = element.as_expression() {
                push_inherited(expression, inherited);
            }
        }
    }
    if let Some(extends) = option_expression_property(options, "extends") {
        push_inherited(extends, inherited);
    }
}

fn push_inherited(expression: &Expression<'_>, inherited: &mut Vec<InheritedComponent>) {
    let Some(reference) = reference_text(expression) else {
        return;
    };
    let span = expression.span();
    inherited.push(InheritedComponent {
        reference,
        src: span.start as usize..span.end as usize,
    });
}

/// The authored text of a static value reference (`base`, `shared.base`), or
/// `None` when the expression is not something `typeof` can name.
fn reference_text(expression: &Expression<'_>) -> Option<String> {
    match expression {
        Expression::Identifier(identifier) => {
            let name = identifier.name.as_str();
            is_safe_value_identifier(name).then(|| String::from(name))
        }
        Expression::StaticMemberExpression(member) => {
            let mut text = reference_text(&member.object)?;
            let property = member.property.name.as_str();
            if !is_safe_value_identifier(property) {
                return None;
            }
            append!(text, ".{property}");
            Some(text)
        }
        Expression::ParenthesizedExpression(value) => reference_text(&value.expression),
        Expression::TSAsExpression(value) => reference_text(&value.expression),
        Expression::TSSatisfiesExpression(value) => reference_text(&value.expression),
        Expression::TSNonNullExpression(value) => reference_text(&value.expression),
        _ => None,
    }
}

fn unwrap_expression<'a, 'b>(expression: &'a Expression<'b>) -> &'a Expression<'b> {
    match expression {
        Expression::ParenthesizedExpression(value) => unwrap_expression(&value.expression),
        Expression::TSAsExpression(value) => unwrap_expression(&value.expression),
        Expression::TSSatisfiesExpression(value) => unwrap_expression(&value.expression),
        Expression::TSNonNullExpression(value) => unwrap_expression(&value.expression),
        _ => expression,
    }
}
