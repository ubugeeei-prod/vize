//! The SFC's default export in Content Mapper output.
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
use std::path::Path;

use super::{CONTENT_MAPPER_SPAN_FEATURES_NAVIGATION_TARGET, ContentMapperSpanKind};
use crate::batch::generate_vue_content_mapper_transform;

/// The line declaring the default export's value, and the export itself.
fn default_export(text: &str) -> Option<(&str, &str)> {
    let lines = text.lines().collect::<Vec<_>>();
    let export = lines
        .iter()
        .position(|line| line.starts_with("export default "))?;
    Some((lines.get(export.checked_sub(1)?)?, lines.get(export)?))
}

fn declared_export(value: &str) -> (String, String) {
    (
        format!("declare const {value}: typeof __vize_component__;"),
        format!("export default {value};"),
    )
}

fn assert_declared_export(text: &str, value: &str) {
    let (declaration, export) = declared_export(value);
    assert_eq!(
        default_export(text),
        Some((declaration.as_str(), export.as_str())),
        "{text}"
    );
}

/// The exported name navigates to the SFC itself, and its type prints as
/// the component rather than as generated members (#11635, #11637).
#[test]
fn the_export_navigates_to_the_sfc_and_its_type_prints_as_the_component() {
    for (file_name, source, value, type_name) in [
        (
            "MkNoteHeader.vue",
            "<script setup lang=\"ts\">\ndefineProps<{ note: string }>();\n</script>\n<template><p /></template>",
            "MkNoteHeader",
            "MkNoteHeaderComponent",
        ),
        (
            "Child.vue",
            "<script setup lang=\"ts\">\nconst Child = 1;\ntype ChildComponent = 2;\n</script>",
            "Child",
            "ChildComponent_",
        ),
    ] {
        let result =
            generate_vue_content_mapper_transform(Path::new(file_name), source).expect("transform");
        assert_declared_export(&result.text, value);
        let declared = declared_export(value).0;
        let start = result
            .text
            .find(&declared)
            .unwrap_or_else(|| panic!("{}", result.text))
            + "declare const ".len();
        let at_sfc = result
            .mappings
            .iter()
            .filter(|span| span.0[2] == 0 && span.0[3] == 0)
            .map(|span| span.0)
            .collect::<Vec<_>>();
        assert_eq!(
            at_sfc,
            [[
                start,
                value.len(),
                0,
                0,
                ContentMapperSpanKind::Atom as usize,
                CONTENT_MAPPER_SPAN_FEATURES_NAVIGATION_TARGET,
            ]],
            "{}",
            result.text
        );
        let type_prefix = format!("type {type_name} = ");
        let declared_type = format!("declare const __vize_component__: {type_name};");
        let named = (
            result
                .text
                .lines()
                .filter(|line| line.starts_with(&type_prefix))
                .count(),
            result
                .text
                .lines()
                .filter(|line| *line == declared_type)
                .count(),
        );
        assert_eq!(named, (1, 1), "{}", result.text);
    }
}

#[test]
fn names_the_default_export_from_the_sfc_filename() {
    let source = "<script setup lang=\"ts\"></script>";
    for (file_name, component_name) in [
        ("Child.vue", "Child"),
        ("user-card.vue", "UserCard"),
        ("user_card.vue", "UserCard"),
        ("component.name.vue", "Component_name"),
        ("123-widget.vue", "_123Widget"),
        ("日本語.vue", "VueComponent"),
    ] {
        let result =
            generate_vue_content_mapper_transform(Path::new(file_name), source).expect("transform");
        assert_declared_export(&result.text, component_name);
    }
}

#[test]
fn avoids_a_filename_collision_with_an_authored_module_binding() {
    for (source, value) in [
        (
            "<script setup lang=\"ts\">\nimport Child from \"./dependency\";\nvoid Child;\n</script>",
            "ChildVueComponent",
        ),
        (
            "<script setup lang=\"ts\">\nconst Child = 1;\n</script>",
            "Child",
        ),
    ] {
        let result = generate_vue_content_mapper_transform(Path::new("Child.vue"), source)
            .expect("transform");
        assert_declared_export(&result.text, value);
    }
}
