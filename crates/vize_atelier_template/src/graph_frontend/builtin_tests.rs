use vize_carton::{BindingMetadata, BindingType, source_anchor::SourceAnchor};
use vize_rendu::{RenduComponentKind, RenduName, RenduNode};

use super::{
    TemplateGraphAdapter, lower_relief_snapshot_to_rendu_with_anchor_and_bindings,
    tests::transformed_snapshot_of,
};

#[test]
fn vue_component_kinds_are_first_class_in_rendu() {
    for (template, expected) in [
        ("<UserCard />", RenduComponentKind::Ordinary),
        ("<Suspense />", RenduComponentKind::Suspense),
        ("<Teleport to=\"body\" />", RenduComponentKind::Teleport),
        ("<KeepAlive />", RenduComponentKind::KeepAlive),
        ("<Transition />", RenduComponentKind::Transition),
        ("<TransitionGroup />", RenduComponentKind::TransitionGroup),
        ("<component></component>", RenduComponentKind::Ordinary),
        (
            "<component :is=\"view\"></component>",
            RenduComponentKind::Dynamic,
        ),
    ] {
        let snapshot = transformed_snapshot_of(template);
        let rendu = TemplateGraphAdapter::new(&snapshot)
            .lower_rendu()
            .expect("component kind lowers");
        let Some(RenduNode::Component { kind, .. }) = rendu.node(rendu.entry()[0]) else {
            panic!("expected component node for {template}");
        };
        assert_eq!(*kind, expected, "wrong component kind for {template}");
    }
}

#[test]
fn authored_builtin_name_survives_setup_binding_resolution() {
    let snapshot = transformed_snapshot_of("<BaseTransition />");
    let mut bindings = BindingMetadata {
        is_script_setup: true,
        ..Default::default()
    };
    bindings
        .bindings
        .insert("BaseTransition".into(), BindingType::SetupConst);
    let rendu = lower_relief_snapshot_to_rendu_with_anchor_and_bindings(
        &snapshot,
        SourceAnchor::new(1, 1),
        &bindings,
    )
    .expect("bound built-in lowers");

    let Some(RenduNode::Component {
        kind: RenduComponentKind::Transition,
        name: RenduName::Static(name),
        ..
    }) = rendu.node(rendu.entry()[0])
    else {
        panic!("authored built-in identity must survive setup binding resolution");
    };
    assert_eq!(name.as_ref(), "BaseTransition");
}
