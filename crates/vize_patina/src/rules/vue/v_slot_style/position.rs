//! Classify a `v-slot` occurrence the way `eslint-plugin-vue` does.
//!
//! Upstream picks the expected style from three facts: whether the slot carries
//! an argument, whether that argument is the static name `default`, and whether
//! the element the directive sits on is a `<template>`. The actual style comes
//! from the authored text — a `#` prefix is shorthand, an argument without one
//! is longform, and a bare `v-slot` is its own style.

use super::VSlotStyleOption;
use crate::markup::{MarkupDirective, MarkupElement};

/// Which of the three configurable positions a `v-slot` occupies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SlotPosition {
    /// The default slot on a component element.
    AtComponent,
    /// The default slot on a `<template>` element.
    DefaultOnTemplate,
    /// A named slot, including one with a dynamic argument.
    Named,
}

pub(super) fn slot_position(
    element: &MarkupElement<'_>,
    directive: &MarkupDirective<'_>,
) -> SlotPosition {
    let is_default =
        directive.argument_range().is_none() || directive.is_static_arg_exact("default");
    if !is_default {
        return SlotPosition::Named;
    }
    if element.is_tag("template") {
        SlotPosition::DefaultOnTemplate
    } else {
        SlotPosition::AtComponent
    }
}

/// The style the directive is authored in.
pub(super) fn actual_style(source: &str, directive: &MarkupDirective<'_>) -> VSlotStyleOption {
    if slot_name(source, directive).starts_with('#') {
        VSlotStyleOption::Shorthand
    } else if directive.argument_range().is_some() {
        VSlotStyleOption::Longform
    } else {
        VSlotStyleOption::VSlot
    }
}

/// The authored directive name and argument, without the value — the text
/// upstream substitutes into its message as `actual`.
pub(super) fn slot_argument<'a>(
    source: &'a str,
    directive: &MarkupDirective<'_>,
) -> Option<&'a str> {
    let text = slot_name(source, directive);
    (!text.is_empty()).then_some(text)
}

/// The argument exactly as authored, brackets and all, which is what upstream
/// reads off the argument node for its message. `None` means the default slot.
pub(super) fn argument_text<'a>(
    source: &'a str,
    directive: &MarkupDirective<'_>,
) -> Option<&'a str> {
    let name = slot_name(source, directive);
    let argument = match name.strip_prefix('#') {
        Some(rest) => rest,
        None => name.split_once(':')?.1,
    };
    (!argument.is_empty()).then_some(argument)
}

fn slot_name<'a>(source: &'a str, directive: &MarkupDirective<'_>) -> &'a str {
    let range = directive.range();
    let start = range.start as usize;
    let end = range.end as usize;
    source.get(start..end).map_or("", name_of)
}

/// The name-and-argument half of an authored attribute.
fn name_of(attribute: &str) -> &str {
    match attribute.split_once('=') {
        Some((name, _)) => name.trim_end(),
        None => attribute.trim_end(),
    }
}

#[cfg(test)]
mod tests {
    use super::name_of;

    #[test]
    fn a_valueless_directive_is_its_whole_text() {
        assert_eq!(name_of("v-slot"), "v-slot");
        assert_eq!(name_of("#header"), "#header");
    }

    #[test]
    fn a_valued_directive_stops_at_the_separator() {
        assert_eq!(name_of(r#"v-slot:header="props""#), "v-slot:header");
        assert_eq!(name_of(r#"#default="{ item }""#), "#default");
    }

    #[test]
    fn whitespace_around_the_separator_is_dropped() {
        assert_eq!(name_of(r#"v-slot = "props""#), "v-slot");
    }
}
