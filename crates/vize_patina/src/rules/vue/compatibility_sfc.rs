//! Compatibility rules whose evidence belongs to the SFC or script surface.

mod content;
mod enforce_style_attribute;
mod events;
mod next_tick_style;
mod no_define_model_default;
mod options;
mod require_macro_variable_name;
mod sort_keys;

pub use content::NoContentOutsideBlock;
pub use enforce_style_attribute::{EnforceStyleAttribute, StyleAttribute};
pub use events::NoShadowNativeEvents;
pub use next_tick_style::NextTickStyle;
pub use no_define_model_default::NoDefineModelDefault;
pub use options::NoPropsShadow;
pub use require_macro_variable_name::RequireMacroVariableName;
pub use sort_keys::SortKeys;

use crate::context::LintContext;
use crate::ir::ByteRange;
use oxc_ast::ast::{
    CallExpression, Expression, ObjectExpression, ObjectPropertyKind, Program, PropertyKey,
};
use oxc_span::{GetSpan, Span};

pub(super) fn setup_program<T>(
    ctx: &LintContext<'_>,
    collect: impl FnOnce(&Program<'_>, u32, &str) -> T,
) -> T
where
    T: Default,
{
    let Some(descriptor) = ctx.sfc_descriptor() else {
        return T::default();
    };
    let Some(block) = descriptor.script_setup.as_ref() else {
        return T::default();
    };
    let Some(parsed) = ctx
        .sfc_scripts()
        .and_then(|scripts| scripts.script_setup())
        .filter(|parsed| !parsed.panicked && parsed.diagnostics.is_empty())
    else {
        return T::default();
    };
    collect(
        &parsed.program,
        block.loc.start as u32,
        block.content.as_ref(),
    )
}

pub(super) fn collect_script_programs(
    ctx: &LintContext<'_>,
    mut collect: impl FnMut(&Program<'_>, u32, &str),
) {
    let (Some(descriptor), Some(scripts)) = (ctx.sfc_descriptor(), ctx.sfc_scripts()) else {
        return;
    };
    if let (Some(block), Some(parsed)) = (descriptor.script.as_ref(), scripts.script())
        && !parsed.panicked
        && parsed.diagnostics.is_empty()
    {
        collect(
            &parsed.program,
            block.loc.start as u32,
            block.content.as_ref(),
        );
    }
    if let (Some(block), Some(parsed)) = (descriptor.script_setup.as_ref(), scripts.script_setup())
        && !parsed.panicked
        && parsed.diagnostics.is_empty()
    {
        collect(
            &parsed.program,
            block.loc.start as u32,
            block.content.as_ref(),
        );
    }
}

pub(super) fn callee_name<'a>(call: &'a CallExpression<'a>) -> Option<&'a str> {
    match call.callee.get_inner_expression() {
        Expression::Identifier(identifier) => Some(identifier.name.as_str()),
        Expression::StaticMemberExpression(member) => Some(member.property.name.as_str()),
        _ => None,
    }
}

pub(super) fn property_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::Identifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::StringLiteral(literal) => Some(literal.value.as_str()),
        _ => None,
    }
}

pub(super) fn property_span(object: &ObjectExpression<'_>, name: &str) -> Option<Span> {
    object.properties.iter().find_map(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        (property_name(&property.key) == Some(name)).then(|| property.span())
    })
}

pub(super) fn absolute_range(offset: u32, span: Span) -> ByteRange {
    ByteRange {
        start: offset + span.start,
        end: offset + span.end,
    }
}

#[cfg(test)]
mod tests;
