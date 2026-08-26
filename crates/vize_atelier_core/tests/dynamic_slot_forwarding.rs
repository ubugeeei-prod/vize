use vize_atelier_core::{
    CodegenOptions, CodegenResult, TransformOptions, generate, parse, transform,
};

fn result_output(result: &CodegenResult) -> String {
    let mut output = String::with_capacity(result.preamble.len() + result.code.len() + 1);
    output.push_str(&result.preamble);
    output.push('\n');
    output.push_str(&result.code);
    output
}

fn compile(source: &str) -> String {
    let allocator = vize_s0::Allocator::new();
    let (mut root, errors) = parse(&allocator, source);
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);

    transform(
        &allocator,
        &mut root,
        TransformOptions {
            prefix_identifiers: true,
            ..Default::default()
        },
        None,
    );

    result_output(&generate(
        &root,
        CodegenOptions {
            prefix_identifiers: true,
            ..Default::default()
        },
    ))
}

#[test]
fn looped_dynamic_slot_name_uses_v_for_alias() {
    let output = compile(
        r#"<Wrapper>
  <template v-for="(_, s) of $slots" :key="s" #[s]="scope">
    <slot :name="s" v-bind="scope" />
  </template>
</Wrapper>"#,
    );

    assert!(
        output.contains("name: s,"),
        "looped dynamic slot names should use the v-for alias as an expression:\n{}",
        output
    );
    assert!(
        output.contains(r#"_renderSlot(_ctx.$slots, s"#),
        "forwarded slot outlet should render the matching dynamic slot name:\n{}",
        output
    );
    assert!(
        !output.contains("name: \"s\"") && !output.contains("_ctx.s"),
        "dynamic slot aliases must not be emitted as literals or outer-scope refs:\n{}",
        output
    );
}
