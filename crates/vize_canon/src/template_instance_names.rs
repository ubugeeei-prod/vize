//! A name a template cannot resolve is a property the component instance
//! lacks, shared by all clients.
//!
//! The virtual module binds template names as lexical variables, so
//! TypeScript reports the lexical codes (`TS2304` / `TS2552`); the Vue
//! toolchain reads the same names from the instance and reports `TS2339` /
//! `TS2551`. Authored templates get the instance codes.
//!
//! A Vue 3 template without an Options API default export reads the names it
//! cannot bind off the generated `__VizeTemplateInstance` (the public instance
//! with its `ComponentCustomProperties`); the missing-property diagnostic that
//! names that generated type gets the same instance wording.

use std::ops::Range;
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_carton::{String, cstr};

const CANNOT_FIND_NAME: &str = "Cannot find name '";
const MISSING_PROPERTY: &str = "Property '";
const DOES_NOT_EXIST_ON_TYPE: &str = "' does not exist on type '";
/// The generated type a template reads its unknown names off.
pub const TEMPLATE_INSTANCE_TYPE: &str = "__VizeTemplateInstance";

/// Whether TypeScript reports `code` for a lexical lookup that failed.
pub const fn is_lexical_lookup_failure(code: u32) -> bool {
    matches!(code, 2304 | 2552)
}

/// Whether `code` can report a template name TypeScript failed to find, as a
/// lexical name or as a property of the generated template instance.
pub const fn is_template_name_lookup_failure(code: u32) -> bool {
    matches!(code, 2304 | 2552 | 2339 | 2551)
}

/// The instance-level `(code, message)` of a lexical lookup failure or of a
/// property the generated template instance lacks, or `None` when the
/// diagnostic is anything else.
pub fn instance_diagnostic(code: u32, message: &str) -> Option<(u32, String)> {
    if let Some(rest) = message.strip_prefix(CANNOT_FIND_NAME) {
        let (name, rest) = rest.split_once('\'')?;
        return match code {
            2304 => Some((2339, missing_instance_property(name))),
            2552 => {
                let suggestion = rest.strip_prefix(". Did you mean '")?.strip_suffix("'?")?;
                Some((2551, missing_instance_property_suggestion(name, suggestion)))
            }
            _ => None,
        };
    }
    let rest = message.strip_prefix(MISSING_PROPERTY)?;
    let (name, rest) = rest.split_once(DOES_NOT_EXIST_ON_TYPE)?;
    let rest = rest
        .strip_prefix(TEMPLATE_INSTANCE_TYPE)?
        .strip_prefix('\'')?;
    match code {
        2339 if rest == "." => Some((2339, missing_instance_property(name))),
        2551 => {
            let suggestion = rest.strip_prefix(". Did you mean '")?.strip_suffix("'?")?;
            Some((2551, missing_instance_property_suggestion(name, suggestion)))
        }
        _ => None,
    }
}

fn missing_instance_property(name: &str) -> String {
    cstr!("Property '{name}' does not exist on the component instance.")
}

fn missing_instance_property_suggestion(name: &str, suggestion: &str) -> String {
    cstr!(
        "Property '{name}' does not exist on the component instance. Did you mean '{suggestion}'?"
    )
}

/// The authored content of the SFC's `<template>`, in offsets of `source`.
pub fn template_content_range(source: &str) -> Option<Range<usize>> {
    let descriptor = parse_sfc(source, SfcParseOptions::default()).ok()?;
    let template = descriptor.template.as_ref()?;
    Some(template.loc.start..template.loc.start + template.content.len())
}

#[expect(clippy::string_slice, reason = "tests assert by panicking")]
#[cfg(test)]
mod tests {
    use super::{
        instance_diagnostic, is_lexical_lookup_failure, is_template_name_lookup_failure,
        template_content_range,
    };

    #[test]
    fn a_lexical_lookup_failure_becomes_an_instance_one() {
        assert_eq!(
            instance_diagnostic(2304, "Cannot find name 'missing'."),
            Some((
                2339,
                "Property 'missing' does not exist on the component instance.".into()
            ))
        );
        assert_eq!(
            instance_diagnostic(2552, "Cannot find name 'cout'. Did you mean 'count'?"),
            Some((
                2551,
                "Property 'cout' does not exist on the component instance. Did you mean 'count'?"
                    .into()
            ))
        );
    }

    #[test]
    fn other_diagnostics_are_left_alone() {
        assert_eq!(
            instance_diagnostic(2322, "Type 'string' is not assignable to type 'number'."),
            None
        );
        assert_eq!(
            instance_diagnostic(2304, "Cannot find namespace 'missing'."),
            None
        );
        assert_eq!(
            [2304, 2552, 2339, 2551].map(is_lexical_lookup_failure),
            [true, true, false, false]
        );
        assert_eq!(
            [2304, 2552, 2339, 2551, 2322].map(is_template_name_lookup_failure),
            [true, true, true, true, false]
        );
    }

    #[test]
    fn a_property_the_template_instance_lacks_becomes_an_instance_one() {
        assert_eq!(
            instance_diagnostic(
                2339,
                "Property 'missing' does not exist on type '__VizeTemplateInstance'."
            ),
            Some((
                2339,
                "Property 'missing' does not exist on the component instance.".into()
            ))
        );
        assert_eq!(
            instance_diagnostic(
                2551,
                "Property 'isSp' does not exist on type '__VizeTemplateInstance'. Did you mean 'isSP'?"
            ),
            Some((
                2551,
                "Property 'isSp' does not exist on the component instance. Did you mean 'isSP'?"
                    .into()
            ))
        );
        // Only the generated template instance is rewritten: a missing property
        // of an authored type keeps TypeScript's wording.
        assert_eq!(
            instance_diagnostic(
                2339,
                "Property 'missing' does not exist on type '__VizeTemplateInstanceMember<\"x\">'."
            ),
            None
        );
        assert_eq!(
            instance_diagnostic(2339, "Property 'nope' does not exist on type 'string'."),
            None
        );
        assert_eq!(
            instance_diagnostic(
                2322,
                "Property 'missing' does not exist on type '__VizeTemplateInstance'."
            ),
            None
        );
    }

    #[test]
    fn the_template_range_is_its_authored_content() {
        let source = "<script setup>const a = 1</script>\n<template>{{ a }}</template>\n";
        assert_eq!(
            template_content_range(source).map(|range| &source[range]),
            Some("{{ a }}")
        );
        assert_eq!(
            template_content_range("<script>export default {}</script>"),
            None
        );
    }
}
