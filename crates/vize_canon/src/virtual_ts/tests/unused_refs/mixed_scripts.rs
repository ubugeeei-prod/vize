//! A normal script must not erase the authored owner of a setup ref binding.

use crate::{ContentMapperTransformOptions, generate_vue_content_mapper_transform_with_options};
use std::path::Path;

const OBJECT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/mixed-script-static-ref-11811/WithScriptObject.vue.txt"
));
const PLAIN: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/mixed-script-static-ref-11811/WithScriptPlain.vue.txt"
));
const SETUP_ONLY: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/mixed-script-static-ref-11811/SetupOnlyObject.vue.txt"
));

fn raw_reads(source: &str) -> Vec<std::string::String> {
    let mut observed = None;
    for preserve_unused in [false, true] {
        let transform = generate_vue_content_mapper_transform_with_options(
            Path::new("StaticRefs11811.vue"),
            source,
            ContentMapperTransformOptions::default()
                .with_preserve_unused_diagnostics(preserve_unused),
        )
        .expect("the complete authored SFC must project");
        println!("mixed static ref projection ({preserve_unused}): {transform:?}");
        let reads = transform
            .text
            .lines()
            .filter_map(|line| line.trim().strip_prefix("void (() => "))
            .filter_map(|line| line.strip_suffix(");"))
            .map(str::to_owned)
            .collect::<Vec<_>>();
        if let Some(prior) = &observed {
            assert_eq!(
                prior, &reads,
                "unused-diagnostic policy must not change runtime reads"
            );
        }
        observed = Some(reads);
    }
    observed.expect("both projection policies were observed")
}

#[test]
fn mixed_script_destructured_static_refs_keep_the_same_runtime_read_as_controls() {
    for (case, source) in [
        ("mixed-object", OBJECT),
        ("mixed-plain", PLAIN),
        ("setup-only", SETUP_ONLY),
    ] {
        assert_eq!(raw_reads(source), ["el"], "{case}");
    }
}

#[test]
fn mixed_script_ref_ownership_survives_aliases_nested_patterns_and_same_spelling() {
    for pattern in ["{ el }", "{ value: el }", "{ nested: { el } }", "[el]"] {
        let source = OBJECT.replace("{ el }", pattern);
        assert_eq!(raw_reads(&source), ["el"], "{pattern}");
    }
    let collision = OBJECT.replace(
        "export const LABEL = \"box\";",
        "const el = 1; export const LABEL = el;",
    );
    assert_eq!(
        raw_reads(&collision),
        ["el"],
        "setup declaration owns the shadowing read"
    );
}

#[test]
fn normal_script_destructures_and_template_shadows_are_not_setup_runtime_reads() {
    let normal_owned = r#"<script lang="ts">
import { useBoxObject } from "./useBoxObject";
const { el } = useBoxObject();
export const LABEL = "box";
</script>
<script setup lang="ts">const other = 1;</script>
<template><div ref="el">{{ LABEL }} {{ other }}</div></template>"#;
    assert!(raw_reads(normal_owned).is_empty());
    let shadowed = OBJECT.replace("ref=\"el\"", "v-for=\"el in [1]\" ref=\"el\"");
    assert!(raw_reads(&shadowed).is_empty());
}
