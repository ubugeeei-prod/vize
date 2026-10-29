//! Issue #10877: non-inline setup components and literal-const props.
//!
//! `@vue/compiler-sfc` writes a setup component as `$setup["MyButton"]` and
//! treats a `literal-const` prop value as patch-stable, so the call carries
//! no `PROPS` flag.

use vize_atelier_core::options::{BindingMetadata, BindingType, CodegenMode};
use vize_atelier_dom::{DomCompilerOptions, compile_template_with_options};
use vize_l0::{Allocator, FxHashMap};

#[test]
fn non_inline_setup_component_and_literal_const_prop_match_vue() {
    let allocator = Allocator::new();
    let mut bindings = FxHashMap::default();
    bindings.insert("MyButton".into(), BindingType::SetupConst);
    bindings.insert("msg".into(), BindingType::LiteralConst);
    let options = DomCompilerOptions {
        mode: CodegenMode::Module,
        prefix_identifiers: true,
        inline: false,
        binding_metadata: Some(BindingMetadata {
            bindings,
            props_aliases: FxHashMap::default(),
            is_script_setup: true,
        }),
        ..Default::default()
    };

    let (_, errors, result) = compile_template_with_options(
        &allocator,
        r#"<div><MyButton :label="msg" /></div>"#,
        options,
    );
    assert!(errors.is_empty(), "errors: {errors:?}");
    let code = result.code.as_str();
    assert!(
        code.contains(r#"_createVNode($setup["MyButton"], { label: $setup.msg })"#),
        "component tag and literal-const prop should match Vue, got:\n{code}"
    );
    assert!(
        !code.contains("PROPS") && !code.contains(r#"["label"]"#),
        "literal-const prop must not keep a PROPS patch, got:\n{code}"
    );
}
