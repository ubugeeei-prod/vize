//! Known script components must keep prop checks across Vue tag spellings.
#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "regression assertions must fail loudly"
)]

use std::path::Path;
use vize_canon::generate_vue_content_mapper_transform;
use vize_canon::sfc_typecheck::{SfcTypeCheckOptions, type_check_sfc};

#[test]
fn imported_component_props_survive_tag_case_resolution() {
    for (binding, tag) in [
        ("WitnessChild", "WitnessChild"),
        ("WitnessChild", "witnessChild"),
        ("WitnessChild", "witness-child"),
        ("witnessChild", "witnessChild"),
        ("witnessChild", "witness-child"),
    ] {
        for newline in ["\n", "\r\n"] {
            let source = vize_carton::cstr!(
                "<!-- 日本 🧭 -->\n<script setup lang=\"ts\">\nimport {binding} from './component';\n</script>\n<template><{tag} :value=\"'bad'\" /></template>\n"
            ).replace('\n', newline);
            let checker = type_check_sfc(
                &source,
                &SfcTypeCheckOptions::new("src/TagCase.vue").with_virtual_ts(),
            );
            let checker_text = checker.virtual_ts.expect("checker projection");
            let mapper =
                generate_vue_content_mapper_transform(Path::new("src/TagCase.vue"), &source)
                    .unwrap();
            for text in [checker_text.as_str(), mapper.text.as_str()] {
                assert!(
                    text.contains("// Component props type declarations"),
                    "known {binding} rendered as {tag} must retain canonical prop checks"
                );
                assert!(
                    text.contains("'bad'"),
                    "authored invalid expression must survive"
                );
            }
        }
    }
}
