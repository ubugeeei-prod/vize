use vize_armature::parse;
use vize_atelier_core::{CodegenResult, generate, transform};
use vize_carton::{BindingMetadata, BindingType};
use vize_relief::{CodegenOptions, TransformOptions};

fn result_output(result: &CodegenResult) -> String {
    let mut output = String::with_capacity(result.preamble.len() + result.code.len() + 1);
    output.push_str(&result.preamble);
    output.push('\n');
    output.push_str(&result.code);
    output
}

#[test]
fn scoped_slot_props_shadow_inline_props_in_slot_outlet_vbind() {
    let allocator = bumpalo::Bump::new();
    let (mut root, errors) = parse(
        &allocator,
        r#"<RouterLink v-slot="{ href }"><slot v-bind="{ href }" /></RouterLink>"#,
    );
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);

    let mut bindings = vize_carton::FxHashMap::default();
    bindings.insert("href".into(), BindingType::Props);
    let binding_metadata = BindingMetadata {
        bindings,
        props_aliases: vize_carton::FxHashMap::default(),
        is_script_setup: true,
    };

    let transformed = transform(
        &allocator,
        &mut root,
        TransformOptions {
            prefix_identifiers: true,
            inline: true,
            binding_metadata: Some(binding_metadata.clone().into()),
            ..Default::default()
        },
        None,
    );
    assert!(
        transformed.errors.is_empty(),
        "Transform errors: {:?}",
        transformed.errors
    );

    let output = result_output(&generate(
        &root,
        &transformed.hoists,
        CodegenOptions {
            prefix_identifiers: true,
            inline: true,
            binding_metadata: Some(binding_metadata.into()),
            ..Default::default()
        },
    ));

    let forwards_scoped_href =
        output.contains("href: href") || output.contains("_guardReactiveProps({ href })");
    assert!(
        forwards_scoped_href,
        "slot outlet should forward the scoped slot href, not component props. Got:\n{}",
        output
    );
    assert!(
        !output.contains("__props.href")
            && !output.contains("$props.href")
            && !output.contains("_ctx.href"),
        "slot-scoped href must not be emitted with component scope prefixes. Got:\n{}",
        output
    );
}
