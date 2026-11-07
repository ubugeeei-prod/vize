use super::l2_emit_supported;
use crate::DomCompilerOptions;
use vize_atelier_core::options::{CodegenOptions, CustomElementMatcher, TemplateSyntaxMode};
use vize_l0::config::VueVersion;

#[test]
fn supported_vue3_standard_dom_options_enter_l2() {
    assert!(supported(DomCompilerOptions::default()));
}

#[test]
fn every_vue_dialect_enters_l2() {
    for dialect in [
        VueVersion::V0_10,
        VueVersion::V0_11,
        VueVersion::V1,
        VueVersion::V2,
        VueVersion::V2_7,
        VueVersion::V3,
    ] {
        assert!(supported(DomCompilerOptions {
            dialect,
            ..Default::default()
        }));
    }
}

#[test]
fn every_template_syntax_mode_enters_l2() {
    for syntax in [
        TemplateSyntaxMode::Standard,
        TemplateSyntaxMode::Strict,
        TemplateSyntaxMode::Quirks,
    ] {
        assert!(l2_emit_supported(
            &DomCompilerOptions::default(),
            &CodegenOptions::default(),
            &CustomElementMatcher::default(),
            syntax,
            false,
            super::L2EmitSelection::Allowed,
            false,
        ));
    }
}

#[test]
fn optimize_imports_codegen_option_does_not_disarm_l2() {
    assert!(l2_emit_supported(
        &DomCompilerOptions::default(),
        &CodegenOptions {
            optimize_imports: true,
            ..Default::default()
        },
        &CustomElementMatcher::default(),
        TemplateSyntaxMode::Standard,
        false,
        super::L2EmitSelection::Allowed,
        false,
    ));
}

#[test]
fn opaque_custom_element_predicates_enter_the_l2_lane() {
    assert!(l2_emit_supported(
        &DomCompilerOptions::default(),
        &CodegenOptions::default(),
        &CustomElementMatcher::from_static_predicate(is_custom_element),
        TemplateSyntaxMode::Standard,
        false,
        super::L2EmitSelection::Allowed,
        false,
    ));
}

fn supported(options: DomCompilerOptions) -> bool {
    l2_emit_supported(
        &options,
        &CodegenOptions::default(),
        &CustomElementMatcher::default(),
        TemplateSyntaxMode::Standard,
        false,
        super::L2EmitSelection::Allowed,
        false,
    )
}

fn is_custom_element(tag: &str) -> bool {
    tag.starts_with("x-")
}
