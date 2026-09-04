use super::VirtualTsOptions;
use super::helpers::{SHARED_PREAMBLE_DTS, VUE_TYPE_HELPERS, generate_template_context};
use vize_carton::config::VueVersion;

const PUBLIC_BASE_GUARD: &str = "// @ts-ignore TS2694/TS2307: a `vue` without `ComponentPublicInstance` must degrade component public extras to unchecked, never error.";
const PUBLIC_BASE_ALIAS: &str = "type __VizeComponentPublicBase = Omit<import('vue').ComponentPublicInstance, '$props' | '$emit' | '$slots'>;";
const TEMPLATE_CONTEXT_GUARD: &str = "    // @ts-ignore TS2694/TS2307: a `vue` without `ComponentPublicInstance` must degrade template context extras to unchecked, never error.";
const TEMPLATE_CONTEXT_ALIAS: &str = "    type __Ctx = import('vue').ComponentPublicInstance;";

#[test]
fn component_public_base_degrades_when_vue_alias_omits_public_instance() {
    for (name, text) in [
        ("SHARED_PREAMBLE_DTS", SHARED_PREAMBLE_DTS),
        ("VUE_TYPE_HELPERS", VUE_TYPE_HELPERS),
    ] {
        assert_guarded_public_base(name, text);
    }

    let context = generate_template_context(&VirtualTsOptions::default(), VueVersion::V3, false);
    let lines = context.lines().collect::<Vec<_>>();
    let alias_line = lines
        .iter()
        .position(|line| *line == TEMPLATE_CONTEXT_ALIAS)
        .expect("template context must use Vue's public instance helper");
    assert_eq!(
        lines[alias_line - 1],
        TEMPLATE_CONTEXT_GUARD,
        "template context must guard the public instance helper"
    );
}

fn assert_guarded_public_base(name: &str, text: &str) {
    let lines = text.lines().collect::<Vec<_>>();
    let alias_lines = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| **line == PUBLIC_BASE_ALIAS)
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    assert_eq!(
        alias_lines.len(),
        1,
        "{name} must name Vue's public instance helper exactly once"
    );
    assert_eq!(
        lines[alias_lines[0] - 1],
        PUBLIC_BASE_GUARD,
        "{name} must guard it"
    );
}
