//! Original page-order, stack and timing controls for the optional L2 walk.
use super::*;
use std::{hint::black_box, time::Instant};
use vize_l2::walk::NodeEvent;

#[test]
#[ignore = "source-built region walk with original IDs, owners, spans and events"]
fn original_region_events() {
    for (name, breadth, depth) in [
        ("empty", 0, 0),
        ("single", 1, 1),
        ("wide", 128, 1),
        ("nested", 1, 32),
        ("deep", 1, 96),
        ("branches", 16, 8),
    ] {
        let mut source = String::from("<template>");
        for _ in 0..breadth {
            for _ in 0..depth {
                source.push_str("<div>");
            }
            source.push_str("text");
            for _ in 0..depth {
                source.push_str("</div>");
            }
        }
        source.push_str("</template>");
        let arena = Allocator::default();
        let owner = lower_sfc_native(&arena, &source, options());
        let file = owner.admitted().expect(name);
        let artifact = file.file().l2ir().artifact();
        let mut events = Vec::new();
        artifact
            .visit_events(&mut |event| match event {
                NodeEvent::Enter {
                    id,
                    node,
                    parent,
                    owner_span,
                } => events.push((true, id, node.span(), parent, owner_span)),
                NodeEvent::Leave { id, node, parent } => {
                    events.push((false, id, node.span(), parent, None))
                }
            })
            .unwrap();
        let mut ids = Vec::new();
        group::preorder(artifact, &mut |id, node| ids.push((id, node.span()))).unwrap();
        let mut original_ids = Vec::new();
        original_preorder(artifact, &mut |id, node| {
            original_ids.push((id, node.span()))
        })
        .unwrap();
        assert_eq!(ids, original_ids);
        assert_eq!(events.len(), ids.len() * 2);
        eprintln!("events {name} {events:?}");
        eprintln!("nodes {name} {ids:?}");
        for nested in [false, true] {
            let mut ns = Vec::new();
            for i in 0..35 {
                let begin = Instant::now();
                for _ in 0..64 {
                    if nested {
                        black_box(artifact)
                            .visit_events(&mut |event| {
                                black_box(event);
                            })
                            .unwrap();
                    } else {
                        group::preorder(black_box(artifact), &mut |id, node| {
                            black_box((id, node));
                        })
                        .unwrap();
                    }
                }
                let elapsed = begin.elapsed().as_nanos() / 64;
                if i >= 4 {
                    ns.push(elapsed)
                }
            }
            eprintln!("region_timing {name} events={nested} ns={ns:?}");
        }
    }
}

pub(super) fn original_preorder<'s, 'a>(
    artifact: &'s vize_l2::artifact::Artifact<'a>,
    visit: &mut impl FnMut(vize_l0::id::NodeId, vize_l2::walk::NodeRef<'s, 'a>),
) -> Result<(), vize_l2::walk::NodeLimit> {
    artifact.visit_nodes(visit)
}
