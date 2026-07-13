use vize_armature::parse;
use vize_atelier_core::{
    codegen::{CodegenResult, generate},
    lane::transform,
};
use vize_carton::Allocator;
use vize_relief::{CodegenOptions, ExpressionNode, TemplateChildNode, TransformOptions};
fn compile(source: &str, source_map: bool) -> CodegenResult {
    let allocator = Allocator::default();
    let (mut root, errors) = parse(&allocator, source);
    assert!(errors.is_empty(), "parse errors: {errors:?}");
    let transformed = transform(
        &allocator,
        &mut root,
        TransformOptions {
            prefix_identifiers: true,
            ..Default::default()
        },
        None,
    );
    assert!(
        transformed.errors.is_empty(),
        "transform errors: {:?}",
        transformed.errors
    );

    generate(
        &root,
        &transformed.hoists,
        CodegenOptions {
            prefix_identifiers: true,
            source_map,
            filename: "LegacyAtelier.vue".into(),
            ..Default::default()
        },
    )
}

fn expression_text<'a>(expression: &'a ExpressionNode<'_>) -> &'a str {
    match expression {
        ExpressionNode::Simple(expression) => expression.content.as_str(),
        ExpressionNode::Compound(expression) => expression.loc.source.as_str(),
    }
}

#[test]
fn representative_dom_codegen_preserves_the_legacy_relief_emitter() {
    let source = r#"
<Comp v-if="ok" id="root" @click.stop="go">
  <template #default="{ item }"><slot name="row" :value="item" /></template>
</Comp>
<li v-for="(item, index) in items" :key="index">{{ item }}</li>
"#;
    let result = compile(source, false);

    assert!(
        result.code.contains("_renderList(_ctx.items"),
        "{}",
        result.code
    );
    assert!(
        result.code.contains("_renderSlot(_ctx.$slots"),
        "{}",
        result.code
    );
    assert!(result.code.contains("_withModifiers"), "{}", result.code);
}

#[test]
fn control_flow_and_slot_prop_emitters_record_attribute_names() {
    for source in [
        r#"<div v-if="ok" data-id="if"></div>"#,
        r#"<div v-for="item in items" data-id="for"></div>"#,
        r#"<slot data-id="slot" />"#,
    ] {
        let result = compile(source, true);
        let map: serde_json::Value =
            serde_json::from_str(result.map.as_deref().expect("source map")).unwrap();
        assert!(
            map["names"]
                .as_array()
                .is_some_and(|names| names.iter().any(|name| name == "data-id")),
            "source={source}\nmap={map}"
        );
    }
}

#[test]
fn transformed_relief_preserves_control_flow_and_for_aliases() {
    let allocator = Allocator::default();
    let source = concat!(
        "<div v-if=\"ok\">A</div><p v-else>B</p>",
        "<li v-for=\"(item, key, index) in items\">{{ item }}</li>",
    );
    let (mut root, errors) = parse(&allocator, source);
    assert!(errors.is_empty(), "parse errors: {errors:?}");
    let transformed = transform(&allocator, &mut root, TransformOptions::default(), None);
    assert!(transformed.errors.is_empty());

    let if_node = match &root.children[0] {
        TemplateChildNode::If(node) => node,
        node => panic!("expected transformed if node, got {node:?}"),
    };
    let for_node = match &root.children[1] {
        TemplateChildNode::For(node) => node,
        node => panic!("expected transformed for node, got {node:?}"),
    };
    assert_eq!(if_node.branches.len(), 2);
    assert_eq!(
        (
            expression_text(&for_node.source),
            for_node.value_alias.as_ref().map(expression_text),
            for_node.key_alias.as_ref().map(expression_text),
            for_node.object_index_alias.as_ref().map(expression_text),
        ),
        ("items", Some("item"), Some("key"), Some("index"))
    );
}
