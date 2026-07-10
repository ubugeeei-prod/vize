use vize_atelier_core::{
    Allocator, CodegenOptions, TransformOptions,
    codegen::{CodegenResult, generate},
    lane::transform,
    parse,
};
use vize_rendu::{RenduElementKind, RenduOp, walk_rendu_ops};

#[derive(Default)]
struct ObservedOps {
    has_if: bool,
    has_if_branch: bool,
    has_for: bool,
    has_component: bool,
    has_slot_outlet: bool,
    has_stopped_event: bool,
}

fn compile(source: &str, source_map: bool) -> (CodegenResult, ObservedOps) {
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

    let mut observed = ObservedOps::default();
    walk_rendu_ops(&root, |op| match op {
        RenduOp::If { .. } => observed.has_if = true,
        RenduOp::IfBranch { .. } => observed.has_if_branch = true,
        RenduOp::For { .. } => observed.has_for = true,
        RenduOp::Element {
            kind: RenduElementKind::Component,
            ..
        } => observed.has_component = true,
        RenduOp::Element {
            kind: RenduElementKind::SlotOutlet,
            ..
        } => observed.has_slot_outlet = true,
        RenduOp::Directive {
            name: "on",
            modifiers,
            ..
        } if modifiers.contains("stop") => observed.has_stopped_event = true,
        _ => {}
    });
    let result = generate(
        &root,
        &transformed.hoists,
        CodegenOptions {
            prefix_identifiers: true,
            source_map,
            filename: "Rendu.vue".into(),
            ..Default::default()
        },
    );
    (result, observed)
}

#[test]
fn representative_dom_codegen_is_driven_by_the_rendu_operation_vocabulary() {
    let source = r#"
<Comp v-if="ok" id="root" @click.stop="go">
  <template #default="{ item }"><slot name="row" :value="item" /></template>
</Comp>
<li v-for="(item, index) in items" :key="index">{{ item }}</li>
"#;
    let (result, observed) = compile(source, false);

    assert!(observed.has_if);
    assert!(observed.has_if_branch);
    assert!(observed.has_for);
    assert!(observed.has_component);
    assert!(observed.has_slot_outlet);
    assert!(observed.has_stopped_event);

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
fn control_flow_and_slot_prop_emitters_record_rendu_attribute_names() {
    for source in [
        r#"<div v-if="ok" data-id="if"></div>"#,
        r#"<div v-for="item in items" data-id="for"></div>"#,
        r#"<slot data-id="slot" />"#,
    ] {
        let (result, _) = compile(source, true);
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
fn transformed_control_flow_preserves_structure_and_for_aliases_in_rendu() {
    let allocator = Allocator::default();
    let source = concat!(
        "<div v-if=\"ok\">A</div><p v-else>B</p>",
        "<li v-for=\"(item, key, index) in items\">{{ item }}</li>",
    );
    let (mut root, errors) = parse(&allocator, source);
    assert!(errors.is_empty(), "parse errors: {errors:?}");
    let transformed = transform(&allocator, &mut root, TransformOptions::default(), None);
    assert!(transformed.errors.is_empty());

    let mut if_branches = 0;
    let mut for_aliases = None;
    walk_rendu_ops(&root, |op| match op {
        RenduOp::IfBranch { .. } => if_branches += 1,
        RenduOp::For {
            source,
            value,
            key,
            index,
            ..
        } => {
            for_aliases = Some((
                source.text(),
                value.map(|expression| expression.text()),
                key.map(|expression| expression.text()),
                index.map(|expression| expression.text()),
            ));
        }
        _ => {}
    });

    assert_eq!(if_branches, 2);
    assert_eq!(
        for_aliases,
        Some(("items", Some("item"), Some("key"), Some("index")))
    );
}
