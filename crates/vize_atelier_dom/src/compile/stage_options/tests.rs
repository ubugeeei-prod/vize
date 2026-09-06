use super::s2_emit_supported;
use crate::DomCompilerOptions;
use vize_atelier_core::options::{CodegenOptions, CustomElementMatcher, TemplateSyntaxMode};
use vize_s0::config::VueVersion;

#[test]
fn supported_vue3_standard_dom_options_enter_s2() {
    assert!(supported(DomCompilerOptions::default()));
}

#[test]
fn legacy_dialects_stay_on_the_compatibility_lane() {
    assert!(!supported(DomCompilerOptions {
        dialect: VueVersion::V2,
        ..Default::default()
    }));
    assert!(!supported(DomCompilerOptions {
        dialect: VueVersion::V2_7,
        ..Default::default()
    }));
}

#[test]
fn optimize_imports_codegen_option_does_not_disarm_s2() {
    assert!(s2_emit_supported(
        &DomCompilerOptions::default(),
        &CodegenOptions {
            optimize_imports: true,
            ..Default::default()
        },
        &CustomElementMatcher::default(),
        TemplateSyntaxMode::Standard,
        false,
        super::S2EmitSelection::Allowed,
    ));
}

#[test]
fn opaque_custom_element_predicates_enter_the_s2_lane() {
    assert!(s2_emit_supported(
        &DomCompilerOptions::default(),
        &CodegenOptions::default(),
        &CustomElementMatcher::from_static_predicate(is_custom_element),
        TemplateSyntaxMode::Standard,
        false,
        super::S2EmitSelection::Allowed,
    ));
}

fn supported(options: DomCompilerOptions) -> bool {
    s2_emit_supported(
        &options,
        &CodegenOptions::default(),
        &CustomElementMatcher::default(),
        TemplateSyntaxMode::Standard,
        false,
        super::S2EmitSelection::Allowed,
    )
}

fn is_custom_element(tag: &str) -> bool {
    tag.starts_with("x-")
}
