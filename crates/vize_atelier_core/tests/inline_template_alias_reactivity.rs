//! Metadata-only callers must use the same lexical precedence as SFC analysis.

use vize_atelier_core::{
    BindingMetadata, BindingType, CodegenOptions, TransformOptions, generate, parse, transform,
};
use vize_l0::FxHashMap;

fn compile(binding_type: BindingType) -> vize_l0::String {
    let allocator = vize_l0::Allocator::new();
    let (mut root, errors) = parse(
        &allocator,
        r#"<p>{{ visibility }}</p><option v-for="visibility in choices"
          :key="visibility" :value="visibility">{{ visibility }}</option>
        <p>{{ visibility }}</p>"#,
    );
    assert!(errors.is_empty(), "{errors:?}");
    let bindings = BindingMetadata {
        bindings: FxHashMap::from_iter([
            ("visibility".into(), binding_type),
            ("choices".into(), BindingType::SetupConst),
        ]),
        props_aliases: FxHashMap::default(),
        is_script_setup: true,
    };
    transform(
        &allocator,
        &mut root,
        TransformOptions {
            prefix_identifiers: true,
            inline: true,
            binding_metadata: Some(bindings.clone()),
            ..Default::default()
        },
        None,
    );
    generate(
        &root,
        CodegenOptions {
            prefix_identifiers: true,
            inline: true,
            binding_metadata: Some(bindings),
            ..Default::default()
        },
    )
    .code
}

#[test]
fn metadata_refs_are_unwrapped_only_outside_the_loop_alias_scope() {
    let code = compile(BindingType::SetupRef);
    println!("inline-alias-ref-complete-code {code}");
    assert_eq!(code.matches("visibility.value").count(), 2, "{code}");
    assert!(!code.contains("_unref(visibility)"), "{code}");
    assert!(code.contains("key: visibility"), "{code}");
    assert!(code.contains("value: visibility"), "{code}");
    assert!(code.contains("_toDisplayString(visibility)"), "{code}");
}

#[test]
fn metadata_maybe_refs_are_unwrapped_only_outside_the_loop_alias_scope() {
    for binding in [BindingType::SetupLet, BindingType::SetupMaybeRef] {
        let code = compile(binding);
        println!("inline-alias-maybe-ref-complete-code {binding:?} {code}");
        assert_eq!(code.matches("_unref(visibility)").count(), 2, "{code}");
        assert!(!code.contains("visibility.value"), "{code}");
        assert!(code.contains("key: visibility"), "{code}");
        assert!(code.contains("value: visibility"), "{code}");
        assert!(code.contains("_toDisplayString(visibility)"), "{code}");
    }
}
