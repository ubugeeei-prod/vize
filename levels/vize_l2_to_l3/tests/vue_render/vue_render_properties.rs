use super::vue_render_support::expression_at;
use super::{Observed, bridge, expression, vue_render_support};
use vize_l0::{Allocator, Span, id::NodeId};
use vize_l1::embed::Lang;
use vize_l3::decision::dom::{
    DomDependency, PropertyRole, ValueKind, vue::build_vue_render_decisions,
};

#[test]
fn actual_title_class_style_and_text_events_keep_dynamic_roles_and_read_order() {
    let arena = Allocator::default();
    let source = "<script setup>let msg='x', active=true, styles='color:red';</script><template><p :title=\"msg\" :class=\"active\" :style=\"styles\">{{msg}}</p></template>";
    let observed = Observed::new(&arena, source).unwrap();
    let mut owners = Vec::new();
    let mut values = Vec::new();
    for (name, text) in [("title", "msg"), ("class", "active"), ("style", "styles")] {
        let needle = vize_l0::cstr!(":{name}=\"{text}\"");
        let start = source.find(needle.as_str()).unwrap();
        let value = start + name.len() + 3;
        let owner = expression_at(
            &arena,
            source,
            Span::new(value as u32, (value + text.len()) as u32),
            Lang::Js,
        )
        .unwrap();
        values.push((
            name,
            Span::new((start + 1) as u32, (start + 1 + name.len()) as u32),
            Span::new(start as u32, (value + text.len() + 1) as u32),
            bridge(&arena, source, &owner).unwrap(),
        ));
        owners.push(owner);
    }
    let interpolation = expression(&arena, source, "msg", Lang::Js).unwrap();
    let failed = core::cell::Cell::new(false);
    let template = source.find("<p").unwrap();
    let end = source.find("</p>").unwrap() + 4;
    let file = observed
        .element(
            &arena,
            Span::new(template as u32, end as u32),
            vue_render_support::body::Properties {
                values: values.try_into().unwrap(),
                interpolation: bridge(&arena, source, &interpolation).unwrap(),
                failed: &failed,
            },
        )
        .unwrap();
    assert!(!failed.get());
    let exposure = observed.exposure(&file).unwrap();
    let analysis = build_vue_render_decisions(&exposure).unwrap();
    let facts = analysis.dom().unwrap();
    assert!(facts.unsupported().is_empty());
    let root = facts.node(NodeId::FIRST).unwrap();
    assert!(
        root.changes.properties && root.changes.class && root.changes.style && root.changes.text
    );
    let rows = (1..=3)
        .map(|index| facts.binding(NodeId::from_index(index).unwrap()).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        rows.iter().map(|row| row.role).collect::<Vec<_>>(),
        [
            PropertyRole::Property,
            PropertyRole::Class,
            PropertyRole::Style
        ]
    );
    assert_eq!(
        rows.iter().map(|row| row.value).collect::<Vec<_>>(),
        [ValueKind::FileDependent; 3]
    );
    assert_eq!(
        facts.dependencies(),
        &[
            DomDependency::DisplayValue,
            DomDependency::ClassNormalization,
            DomDependency::StyleNormalization,
            DomDependency::BlockBoundary,
            DomDependency::NativeElementBlock
        ]
    );
    assert_eq!(
        (1..=4)
            .map(|index| analysis
                .expression(NodeId::from_index(index).unwrap())
                .unwrap()
                .reads()
                .first()
                .unwrap()
                .occurrence()
                .name)
            .collect::<Vec<_>>(),
        ["msg", "active", "styles", "msg"]
    );
    assert_eq!(
        owners
            .iter()
            .map(|owner| owner.diagnostics().count())
            .sum::<usize>(),
        0
    );
}
