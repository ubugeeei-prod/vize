use super::{StaticFacts, StaticLevel, run};
use crate::artifact::Builder;
use crate::op::Namespace;
use crate::provenance::ProvenanceRecord;
use vize_l0::{Allocator, Span, String, Vec, id::NodeId};

#[test]
fn native_artifact_publishes_exact_static_facts_and_postorder_provenance() {
    let allocator = Allocator::default();
    let source = "<x><p>t</p><C/></x>";
    let mut builder = Builder::new(&allocator, source).unwrap();
    builder
        .element(
            "x",
            Namespace::Html,
            Vec::new_in(&&allocator),
            Span::new(0, 19),
            |region, _| {
                region
                    .element(
                        "p",
                        Namespace::Html,
                        Vec::new_in(&&allocator),
                        Span::new(3, 11),
                        |region, _| {
                            region.text("t", Span::new(6, 7)).unwrap();
                        },
                    )
                    .unwrap();
                region
                    .component("C", Vec::new_in(&&allocator), Span::new(11, 15), |_, _| {})
                    .unwrap();
            },
        )
        .unwrap();
    let artifact = builder.finish().unwrap();
    assert_eq!(artifact.node_count(), 4);
    let prefix = ProvenanceRecord {
        rule: String::from("native.owner"),
        node: Some(NodeId::FIRST),
        before: String::from(source),
        after: String::from("ui.element"),
        span: Span::new(0, 19),
    };
    let mut provenance = alloc::vec![prefix.clone()];
    let facts = run(artifact.root(), artifact.node_count(), &mut provenance);
    let rows: alloc::vec::Vec<_> = facts
        .sorted_entries()
        .into_iter()
        .map(|(id, fact)| (id.index(), *fact))
        .collect();
    let fact = |level, nested_static, native_descendants| StaticFacts {
        level,
        props_hoistable: false,
        nested_static,
        native_descendants,
        foreign: false,
    };
    assert_eq!(
        rows,
        [
            (0, fact(StaticLevel::NotStatic, false, false)),
            (1, fact(StaticLevel::FullyStatic, true, true)),
            (3, fact(StaticLevel::NotStatic, false, true)),
        ]
    );
    let record = |node, before, after, span| ProvenanceRecord {
        rule: String::from("pass.hoist-static.fact"),
        node: NodeId::from_index(node),
        before: String::from(before),
        after: String::from(after),
        span,
    };
    assert_eq!(
        provenance,
        [
            prefix,
            record(
                1,
                "ui.element",
                "level=fully-static props=false nested=true native=true",
                Span::new(3, 11),
            ),
            record(
                3,
                "ui.component",
                "level=not-static props=false nested=false native=true",
                Span::new(11, 15),
            ),
            record(
                0,
                "ui.element",
                "level=not-static props=false nested=false native=false",
                Span::new(0, 19),
            ),
        ]
    );
    assert_eq!(artifact.source(), source);
    assert_eq!(artifact.node_count(), 4);
    assert_eq!(artifact.provenance(), []);
}
