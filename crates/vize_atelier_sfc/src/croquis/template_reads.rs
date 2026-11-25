//! Blocks outside the template walk complete the demanded read relation.

use crate::SfcDescriptor;
use vize_croquis::croquis::TemplateBindingReads;
use vize_croquis::drawer::extract_identifiers_checked;

/// Join `<style>` `v-bind()` reads, and mark the relation incomplete when a
/// block that can read a script name was not walked.
pub(super) fn complete(
    reads: &mut TemplateBindingReads,
    descriptor: &SfcDescriptor<'_>,
    template_drawn: bool,
    derived: bool,
) {
    if super::unused::has_opaque_blocks(descriptor, derived)
        || (descriptor.template.is_some() && !template_drawn)
    {
        reads.mark_incomplete();
        return;
    }
    for style in &descriptor.styles {
        let Some(ranges) =
            vize_croquis::sfc::__internal::checked_v_bind_expression_ranges(&style.content)
        else {
            reads.mark_incomplete();
            return;
        };
        for range in ranges {
            let Some(names) = style
                .content
                .get(range)
                .and_then(extract_identifiers_checked)
            else {
                reads.mark_incomplete();
                return;
            };
            reads.note_style_bind(&names);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::croquis::{SfcCroquisOptions, analyze_sfc_descriptor};
    use crate::{SfcParseOptions, parse_sfc};

    /// `(complete, may read each of names, a template expression reads each
    /// of names, a style `v-bind()` reads each of names)`.
    fn reads(source: &str, names: &[&str]) -> Option<(bool, Vec<bool>, Vec<bool>, Vec<bool>)> {
        let descriptor = parse_sfc(source, SfcParseOptions::default()).unwrap();
        let allocator = vize_carton::Allocator::new();
        let root = descriptor
            .template
            .as_ref()
            .filter(|template| template.src.is_none())
            .map(|template| {
                vize_armature::Parser::new(&allocator, template.content.as_ref())
                    .parse()
                    .0
            });
        let croquis = analyze_sfc_descriptor(
            &descriptor,
            root.as_ref(),
            SfcCroquisOptions::lint_demand().with_template_binding_reads(),
        );
        let reads = croquis.template_binding_reads()?;
        Some((
            reads.is_complete(),
            names.iter().map(|name| reads.may_read(name)).collect(),
            names
                .iter()
                .map(|name| reads.expression_reads(name))
                .collect(),
            names
                .iter()
                .map(|name| reads.style_bind_reads(name))
                .collect(),
        ))
    }

    #[test]
    fn the_relation_exists_only_on_demand() {
        let source = "<script setup>import Foo from './foo'</script><template>{{ Foo }}</template>";
        let descriptor = parse_sfc(source, SfcParseOptions::default()).unwrap();
        let croquis = analyze_sfc_descriptor(&descriptor, None, SfcCroquisOptions::lint_demand());
        assert!(croquis.template_binding_reads().is_none());
    }

    #[test]
    fn template_and_style_reads_join_one_relation() {
        let source = "<script setup>import { Foo, Bar, Baz, vFocus, panel, Qux, Local } from './foo'</script>\
<template><Bar ref=\"panel\" v-focus><p v-for=\"Local in Foo\" :key=\"Local\">{{ Local }}</p><div is=\"vue:Qux\" /></Bar></template>\
<style>p { color: v-bind('Baz.color'); }</style>";
        assert_eq!(
            reads(
                source,
                &[
                    "Foo", "Bar", "Baz", "vFocus", "panel", "Qux", "Local", "Other"
                ]
            ),
            Some((
                true,
                vec![true, true, true, true, true, true, false, false],
                vec![true, false, false, false, false, false, false, false],
                vec![false, false, true, false, false, false, false, false],
            ))
        );
    }

    #[test]
    fn a_pattern_default_reads_the_scope_around_its_pattern() {
        // `Foo` and `Bar` are defaults read from the script. `item` names the
        // enclosing slot prop and `Baz` the enclosing `v-for` alias.
        let source = "<script setup>import { Foo, Bar, Baz, item } from './foo'</script>\
<template><Card v-slot=\"{ item = Foo }\"><p v-for=\"{ label = Bar, other = item } in [item]\" :key=\"label\">{{ other }}</p></Card>\
<p v-for=\"Baz in [1]\" :key=\"Baz\"><Card v-slot=\"{ value = Baz }\">{{ value }}</Card></p></template>";
        assert_eq!(
            reads(source, &["Foo", "Bar", "Baz", "item"]),
            Some((
                true,
                vec![true, true, false, false],
                vec![true, true, false, false],
                vec![false, false, false, false],
            ))
        );
    }

    #[test]
    fn unwalked_or_unreadable_blocks_rule_nothing_out() {
        for source in [
            "<script setup>import Foo from './foo'</script><template src=\"./view.html\" />",
            "<script setup>import Foo from './foo'</script><template lang=\"pug\">\np\n</template>",
            "<script setup>import Foo from './foo'</script><template><p /></template><style src=\"./a.css\" />",
            "<script setup>import Foo from './foo'</script><template><p /></template><style>p { color: v-bind(broken +); }</style>",
            "<script setup>import Foo from './foo'</script><template>{{ eval('Foo') }}</template>",
            "<script setup>import Foo from './foo'</script><template>{{ broken + }}</template>",
        ] {
            assert_eq!(
                reads(source, &["Foo"]),
                Some((false, vec![true], vec![false], vec![false])),
                "{source}"
            );
        }
    }

    #[test]
    fn a_script_only_component_has_a_complete_empty_relation() {
        assert_eq!(
            reads("<script setup>import Foo from './foo'</script>", &["Foo"]),
            Some((true, vec![false], vec![false], vec![false]))
        );
    }
}
