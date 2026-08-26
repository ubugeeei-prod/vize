use vize_atelier_core::{
    CodegenOptions, CodegenResult, TransformOptions, generate, parse, transform,
};
use vize_s0::String;

fn result_output(result: &CodegenResult) -> String {
    let mut output = String::with_capacity(result.preamble.len() + result.code.len() + 1);
    output.push_str(&result.preamble);
    output.push('\n');
    output.push_str(&result.code);
    output
}

fn compile(source: &str) -> String {
    compile_with_options(source, CodegenOptions::default())
}

fn compile_with_options(source: &str, mut codegen_options: CodegenOptions) -> String {
    let allocator = vize_s0::Allocator::new();
    let (mut root, errors) = parse(&allocator, source);
    assert!(errors.is_empty(), "Parse errors: {errors:?}");
    transform(
        &allocator,
        &mut root,
        TransformOptions {
            prefix_identifiers: true,
            ..Default::default()
        },
        None,
    );
    codegen_options.prefix_identifiers = true;
    result_output(&generate(&root, codegen_options))
}

#[test]
fn v_if_key_and_props_stay_before_and_after_object_v_bind() {
    let output = compile(r#"<a v-if="show" target="_blank" v-bind="attrs" :href="href"></a>"#);

    assert!(
        output.contains(
            "_mergeProps({\n      key: 0,\n      target: \"_blank\"\n    }, _ctx.attrs, {\n      href: _ctx.href\n    })",
        ),
        "v-if props must be flushed around v-bind in source order:\n{output}",
    );
}

#[test]
fn v_if_key_precedes_a_leading_object_v_bind() {
    let output = compile(r#"<a v-if="show" v-bind="attrs" target="_blank"></a>"#);

    assert!(
        output.contains("_mergeProps({ key: 0 }, _ctx.attrs, {\n      target: \"_blank\"\n    })",),
        "the generated branch key must precede a leading v-bind:\n{output}",
    );
}

#[test]
fn v_if_key_with_only_object_v_bind_is_normalized() {
    let output = compile(r#"<div v-if="show" v-bind="attrs"></div>"#);

    assert!(
        output.contains("_normalizeProps(_mergeProps({ key: 0 }, _ctx.attrs))"),
        "a synthetic branch key plus only v-bind spread must match Vue's normalize wrapper:\n{output}",
    );
    assert!(output.contains("guardReactiveProps"), "{output}");
}

#[test]
fn component_v_if_keeps_interleaved_object_spreads_in_source_order() {
    let output = compile(
        r#"<Widget v-if="show" id="before" v-on="listeners" title="middle" v-bind="attrs" @click="after" />"#,
    );

    assert!(
        output.contains(
            "_mergeProps({\n      key: 0,\n      id: \"before\"\n    }, _toHandlers(_ctx.listeners, true), {\n      title: \"middle\"\n    }, _ctx.attrs, {\n      onClick: _ctx.after\n    })",
        ),
        "component spreads and prop segments must keep source order:\n{output}",
    );
}

#[test]
fn v_if_class_bindings_do_not_merge_across_object_spreads() {
    let output = compile(r#"<div v-if="show" class="base" v-bind="attrs" :class="classes"></div>"#);

    assert!(
        output.contains(
            "_mergeProps({\n      key: 0,\n      class: \"base\"\n    }, _ctx.attrs, {\n      class: _ctx.classes\n    })",
        ),
        "class bindings on opposite sides of v-bind must stay separate:\n{output}",
    );
    assert!(
        !output.contains("class: [\"base\", _ctx.classes]"),
        "{output}"
    );
}

#[test]
fn v_if_style_bindings_do_not_merge_or_normalize_across_spreads() {
    let output =
        compile(r#"<div v-if="show" style="color: red" v-bind="attrs" :style="styles"></div>"#);

    assert!(
        output.contains(
            "_mergeProps({\n      key: 0,\n      style: \"color: red\"\n    }, _ctx.attrs, {\n      style: _ctx.styles\n    })",
        ),
        "style bindings on opposite sides of v-bind must stay separate:\n{output}",
    );
    assert!(!output.contains("_normalizeStyle"), "{output}");
    assert!(!output.contains("style: ["), "{output}");
}

#[test]
fn v_if_class_bindings_still_merge_within_one_segment() {
    let output = compile(r#"<div v-if="show" class="base" :class="classes" v-bind="attrs"></div>"#);

    assert!(
        output.contains(
            "_mergeProps({\n      key: 0,\n      class: [\"base\", _ctx.classes]\n    }, _ctx.attrs)",
        ),
        "class bindings in one segment must retain Vue's array merge:\n{output}",
    );
    assert!(!output.contains("_normalizeClass"), "{output}");
}

#[test]
fn v_if_user_key_is_hoisted_before_a_leading_spread() {
    let output = compile(r#"<div v-if="show" v-bind="attrs" :key="identity"></div>"#);

    assert!(
        output.contains("_mergeProps({ key: _ctx.identity }, _ctx.attrs)"),
        "the user key is the generated branch key and must win before spreads:\n{output}",
    );
    assert_eq!(output.matches("key: _ctx.identity").count(), 1, "{output}");
}

#[test]
fn v_if_consecutive_v_bind_and_v_on_spreads_do_not_emit_empty_objects() {
    let output = compile(r#"<div v-if="show" v-bind="attrs" v-on="listeners"></div>"#);

    assert!(
        output.contains("_mergeProps({ key: 0 }, _ctx.attrs, _toHandlers(_ctx.listeners, true))",),
        "consecutive spreads must remain adjacent after the branch key:\n{output}",
    );
    assert!(!output.contains("{\n    }"), "{output}");
}

#[test]
fn dynamic_component_is_prop_is_skipped_in_every_segment() {
    let output = compile(
        r#"<component v-if="show" :is="view" id="before" v-bind="attrs" title="after"></component>"#,
    );

    assert!(
        output.contains(
            "_mergeProps({\n      key: 0,\n      id: \"before\"\n    }, _ctx.attrs, {\n      title: \"after\"\n    })",
        ),
        "dynamic component props must keep order while consuming :is:\n{output}",
    );
    assert!(!output.contains("\n      is:"), "{output}");
}

#[test]
fn event_deduplication_is_scoped_to_each_merge_segment() {
    let output = compile(
        r#"<button v-if="show" @click="before" v-bind="attrs" v-on:click="after"></button>"#,
    );

    assert!(
        output.contains(
            "_mergeProps({\n      key: 0,\n      onClick: _ctx.before\n    }, _ctx.attrs, {\n      onClick: _ctx.after\n    })",
        ),
        "handlers separated by a spread must both survive for mergeProps:\n{output}",
    );
    assert_eq!(output.matches("onClick:").count(), 2, "{output}");
}

#[test]
fn scoped_css_marker_is_emitted_only_after_the_final_spread() {
    let output = compile_with_options(
        r#"<div v-if="show" id="before" v-bind="attrs" title="after"></div>"#,
        CodegenOptions {
            scope_id: Some(String::from("data-v-order")),
            ..Default::default()
        },
    );

    assert!(
        output.contains(
            "_mergeProps({\n      key: 0,\n      id: \"before\"\n    }, _ctx.attrs, {\n      title: \"after\",\n      \"data-v-order\": \"\"\n    })",
        ),
        "scope marker must be appended to the trailing props segment:\n{output}",
    );
    assert_eq!(output.matches("data-v-order").count(), 1, "{output}");
}

#[test]
fn no_spread_v_if_keeps_the_key_and_props_object_shape() {
    let output = compile(r#"<div v-if="show" id="stable"></div>"#);

    assert!(
        output.contains("{\n      key: 0,\n      id: \"stable\"\n    }"),
        "non-spread v-if props must retain their established shape:\n{output}",
    );
    assert!(!output.contains("_mergeProps"), "{output}");
}
