//! Re-read the authored IR and independently certify all two-edge responses
//! from the resident graph on each buffer produced by the editor.
use super::*;
use std::ops::ControlFlow;
use vize_l0::Allocator;
use vize_l3::query::{EdgePattern, TwoHopMatch, TwoHopQuery, check_two_hop_matches};

pub(super) fn read(
    db: &ResidentDatabase,
    file: SourceFile,
    source: &str,
) -> Vec<(TwoHopQuery, TwoHopMatch, Span, Span)> {
    let block = view(db, file);
    let batch = block.reach_batch(Lang::Ts).unwrap();
    let template = block.source();
    let arena = Allocator::default();
    let native =
        vize_l1_to_l2::native::lower_component_native(&arena, &template.text, Lang::Ts).unwrap();
    assert!(native.is_supported());
    let lowering = vize_l2_to_l3::lower(&arena, native.artifact.root());
    let ir = lowering.l3ir();
    let mut observations = Vec::new();
    for op in &ir.ops {
        for first in [Direction::Forward, Direction::Reverse] {
            for second in [Direction::Forward, Direction::Reverse] {
                let step = |direction| EdgePattern {
                    direction,
                    kinds: EdgeKinds::ALL,
                };
                let query = TwoHopQuery {
                    start: op.id,
                    first: step(first),
                    second: step(second),
                };
                let mut proof = Vec::new();
                let visit = batch
                    .visit_two_hop(query, |row| {
                        let position = |id| {
                            let relative = ir.ops.iter().find(|op| op.id == id).unwrap().span;
                            let absolute = batch.authored_span(id).unwrap();
                            assert_eq!(Some(absolute), block.authored_span(relative));
                            assert_eq!(absolute.slice(source), relative.slice(&template.text));
                            relative
                        };
                        observations.push((query, row, position(row.middle), position(row.end)));
                        proof.push(row);
                        ControlFlow::<()>::Continue(())
                    })
                    .unwrap();
                assert_eq!(visit, ControlFlow::Continue(()));
                assert_eq!(batch.check_two_hop(query, &proof), Ok(()));
                assert_eq!(check_two_hop_matches(ir, query, &proof), Ok(()));
            }
        }
    }
    observations
}
