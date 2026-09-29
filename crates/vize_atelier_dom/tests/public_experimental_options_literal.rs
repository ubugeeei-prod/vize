//! A downstream-style literal from v0.429.1 must remain constructible.

use vize_atelier_core::options::CodegenExperimentalOptions;

#[test]
fn published_experimental_codegen_options_literal_still_compiles() {
    let old_literal = CodegenExperimentalOptions {
        component_name: None,
        self_component: false,
    };
    assert_eq!(old_literal.component_name, None);
    assert!(!old_literal.self_component);
}
