//! A name a template cannot resolve is a property the component instance
//! lacks, shared by all clients.
//!
//! The virtual module binds template names as lexical variables, so
//! TypeScript reports the lexical codes (`TS2304` / `TS2552`); the Vue
//! toolchain reads the same names from the instance and reports `TS2339` /
//! `TS2551`. Authored templates get the instance codes.

use std::ops::Range;
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_carton::{String, cstr};

const CANNOT_FIND_NAME: &str = "Cannot find name '";

/// Whether TypeScript reports `code` for a lexical lookup that failed.
pub const fn is_lexical_lookup_failure(code: u32) -> bool {
    matches!(code, 2304 | 2552)
}

/// The instance-level `(code, message)` of a lexical lookup failure, or `None`
/// when the diagnostic is anything else.
pub fn instance_diagnostic(code: u32, message: &str) -> Option<(u32, String)> {
    let rest = message.strip_prefix(CANNOT_FIND_NAME)?;
    let (name, rest) = rest.split_once('\'')?;
    match code {
        2304 => Some((
            2339,
            cstr!("Property '{name}' does not exist on the component instance."),
        )),
        2552 => {
            let suggestion = rest.strip_prefix(". Did you mean '")?.strip_suffix("'?")?;
            Some((
                2551,
                cstr!(
                    "Property '{name}' does not exist on the component instance. Did you mean '{suggestion}'?"
                ),
            ))
        }
        _ => None,
    }
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
    use super::{instance_diagnostic, is_lexical_lookup_failure, template_content_range};

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
