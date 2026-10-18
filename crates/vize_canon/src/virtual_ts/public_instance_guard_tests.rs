use super::VirtualTsOptions;
use super::helpers::{
    DECLARATION_HELPERS_DTS, SHARED_PREAMBLE_DTS, VUE_TYPE_HELPERS, generate_template_context,
};
use vize_carton::config::VueVersion;

const PUBLIC_BASE_GUARD: &str = "// @ts-ignore TS2694/TS2307: a `vue` without `ComponentPublicInstance` must degrade component public extras to unchecked, never error.";
const PUBLIC_BASE_ALIAS: &str = "type __VizeComponentPublicBase = Omit<import('vue').ComponentPublicInstance, '$props' | '$emit' | '$slots'>;";
const TEMPLATE_CONTEXT_GUARD: &str = "    // @ts-ignore TS2694/TS2307: a `vue` without `ComponentPublicInstance` must degrade template context extras to unchecked, never error.";
const TEMPLATE_CONTEXT_ALIAS: &str = "    type __Ctx = import('vue').ComponentPublicInstance;";

#[test]
fn vue_public_input_contract_reaches_embedded_hoisted_and_declaration_helpers() {
    for (name, text) in [
        ("embedded", VUE_TYPE_HELPERS),
        ("hoisted", SHARED_PREAMBLE_DTS),
        ("declarations", DECLARATION_HELPERS_DTS),
    ] {
        assert_eq!(
            text.matches("type __VizePublicComponentInputProps = import('vue').PublicProps;")
                .count(),
            1,
            "{name} must use Vue's single public prop authority"
        );
        assert!(
            text.contains("__VizeFallthroughAttrs<T & Declared> & __VizePublicComponentInputProps"),
            "{name} must include that contract in listener inputs"
        );
        assert!(!text.contains("__VizeReservedComponentInputProps"));
    }

    let script = "defineProps<{ value: string }>()";
    let mut analyzer = vize_croquis::Analyzer::with_options(vize_croquis::AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    let output = super::generate_virtual_ts_with_offsets_legacy_vue2(
        &analyzer.finish(),
        Some(script),
        None,
        0,
        0,
        &VirtualTsOptions::default(),
    );
    assert!(!output.code.contains("import('vue').PublicProps"));
    assert!(!output.code.contains("__VizePublicComponentInputProps"));
}

#[test]
fn component_public_base_degrades_when_vue_alias_omits_public_instance() {
    for (name, text) in [
        ("SHARED_PREAMBLE_DTS", SHARED_PREAMBLE_DTS),
        ("VUE_TYPE_HELPERS", VUE_TYPE_HELPERS),
    ] {
        assert_guarded_public_base(name, text);
    }

    let context = generate_template_context(
        &VirtualTsOptions::default(),
        VueVersion::V3,
        false,
        false,
        (None, None, None),
    );
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
