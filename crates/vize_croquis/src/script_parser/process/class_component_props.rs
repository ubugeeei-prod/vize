//! `@Prop` decorator -> prop-contract extraction for class components.
//!
//! A `@Prop`-decorated class member is a real declared prop, exactly like a
//! `props:` entry on an Options API component or a `defineProps` member. Until
//! the contract was recorded here the member was only a `BindingType::Props`
//! *template binding*, so it resolved inside its own component but produced no
//! usage-site contract in a parent: `export type Props` stayed `{}`, a
//! mismatched binding type-checked, and the missing-required-prop surface saw
//! no required props at all (#3298).
//!
//! The decorator argument is a Vue runtime prop declaration — `@Prop(String)`,
//! `@Prop({ type: String, required: true })`, `@Prop({ default: 0 })` — so it
//! is read with the very same helpers the runtime `props: { ... }` object form
//! uses. `required` therefore follows the *runtime* declaration (Vue defaults
//! it to `false`), never the field's `!` definite-assignment assertion, which
//! is an author-side claim rather than a caller-side obligation.
//!
//! The prop *type* prefers the member's TS annotation (`readonly items!:
//! Item[]`) over the runtime ctor (`Array` -> `unknown[]`) because the
//! annotation is strictly more precise and is what the class instance type
//! already resolved template bindings to.
//!
//! Only `@Prop` is recorded. `@PropSync` / `@Model` / `@ModelSync` / `@VModel`
//! rename or pair their prop with a generated computed member, so their
//! contract is not a plain member-name-to-type mapping; they keep resolving
//! through the class instance type as before.

use oxc_ast::ast::{
    Argument, Decorator, Expression, PropertyKey, TSType, TSTypeAnnotation, TSTypeOperatorOperator,
};
use oxc_span::GetSpan;

use vize_carton::CompactString;

use super::super::ScriptParseResult;
use super::super::extract::{
    detect_required_prop, extract_runtime_prop_default, extract_runtime_prop_type,
};
use crate::macros::PropDefinition;

/// Record the prop contract of a `@Prop`-decorated field / accessor.
///
/// `optional` is the member's `?` marker; it makes the prop optional even when
/// the decorator omits `required`, matching how a `defineProps<{ x?: T }>`
/// member is read.
pub(super) fn collect_prop_decorator(
    result: &mut ScriptParseResult,
    key: &PropertyKey<'_>,
    decorators: &[Decorator<'_>],
    type_annotation: Option<&TSTypeAnnotation<'_>>,
    optional: bool,
    source: &str,
) {
    let Some(decorator) = decorators
        .iter()
        .find(|decorator| is_prop_decorator(decorator))
    else {
        return;
    };
    let Some(name) = prop_member_name(key) else {
        return;
    };

    let options = prop_decorator_options(&decorator.expression);
    let declared_type = type_annotation.and_then(|annotation| {
        declared_prop_type_source(&annotation.type_annotation, source).map(CompactString::new)
    });

    let required = !optional && options.is_some_and(detect_required_prop);
    let prop_type = declared_type
        .or_else(|| options.and_then(|options| extract_runtime_prop_type(options, source)));
    let default_value = options.and_then(|options| extract_runtime_prop_default(options, source));

    result.macros.add_prop(PropDefinition {
        name: CompactString::new(name),
        prop_type,
        required,
        default_value,
    });
}

/// Whether a member decorator is `@Prop` / `@Prop(...)`.
///
/// Kept narrower than `is_prop_like_decorator_name` in the binding pass on
/// purpose — see the module docs for why the renaming decorators are excluded.
pub(super) fn is_prop_decorator(decorator: &Decorator<'_>) -> bool {
    match &decorator.expression {
        Expression::Identifier(identifier) => identifier.name.as_str() == "Prop",
        Expression::CallExpression(call) => matches!(
            &call.callee,
            Expression::Identifier(identifier) if identifier.name.as_str() == "Prop"
        ),
        _ => false,
    }
}

/// The runtime prop declaration passed to `@Prop(...)`, if any.
///
/// `@Prop` and `@Prop()` carry no declaration, so both yield `None` and the
/// prop falls back to "optional, typed by its TS annotation".
fn prop_decorator_options<'a>(expression: &'a Expression<'a>) -> Option<&'a Expression<'a>> {
    let Expression::CallExpression(call) = expression else {
        return None;
    };
    match call.arguments.first()? {
        Argument::SpreadElement(_) => None,
        argument => argument.as_expression(),
    }
}

/// Static member name of a `@Prop` field / accessor.
///
/// Computed and hard-private (`#name`) keys never reach a parent template, so
/// they declare no usage-site contract.
fn prop_member_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::StringLiteral(literal) => Some(literal.value.as_str()),
        _ => None,
    }
}

/// Source text of a member's declared type, when it is usable in an
/// `export type Props` member position.
///
/// `unique symbol` and bare `this` types are only valid in their declaration
/// site, so they are dropped and the runtime ctor type takes over.
fn declared_prop_type_source<'a>(ts_type: &TSType<'a>, source: &'a str) -> Option<&'a str> {
    match ts_type {
        TSType::TSThisType(_) => None,
        TSType::TSTypeOperatorType(operator)
            if operator.operator == TSTypeOperatorOperator::Unique =>
        {
            None
        }
        _ => {
            let span = ts_type.span();
            let text = source.get(span.start as usize..span.end as usize)?;
            let trimmed = text.trim();
            (!trimmed.is_empty()).then_some(trimmed)
        }
    }
}
