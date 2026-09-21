//! P3-3 export contract: `PartitionFacts::stale` names the first fact that
//! stops describing the program, and optional passes never produce one.

use vize_davinci::pass::NoObserver;
use vize_s0::{Allocator, Span};
use vize_s2_to_s3::{Lowered, PartitionFact, PartitionKind, lower};
use vize_s3::extract::OptTier;
use vize_s3::op::{EffectId, OpId};
use vize_s3::optimize::optimize;

const SOURCE: &str = r#"<section><p v-if="ready"><b>ok</b></p><i :title="n">{{ n }}</i></section>"#;

fn with_lowered(check: impl FnOnce(Lowered<'_>)) {
    let allocator = Allocator::default();
    let (tree, errors) = vize_s1::parse(&allocator, SOURCE);
    let s2 = vize_s1_to_s2::lower(&allocator, &tree, &errors);
    check(lower(&allocator, &s2.root));
}

#[test]
fn fresh_and_optimized_exports_are_current_at_every_tier() {
    for tier in OptTier::ALL {
        with_lowered(|mut lowered| {
            assert_eq!(lowered.partition.stale(&lowered.program), None);
            optimize(&mut lowered.program, tier, &mut NoObserver).expect("closed pipeline");
            assert_eq!(lowered.partition.stale(&lowered.program), None);
        });
    }
}

#[test]
fn each_drift_names_its_first_position() {
    with_lowered(|mut lowered| {
        lowered.partition.ops[2].span = Span::new(0, 1);
        assert_eq!(lowered.partition.stale(&lowered.program), Some(2));
    });
    with_lowered(|mut lowered| {
        lowered.partition.ops[3].op = OpId::new(99);
        assert_eq!(lowered.partition.stale(&lowered.program), Some(3));
    });
    with_lowered(|mut lowered| {
        lowered.partition.ops[0].kind = PartitionKind::Dynamic;
        assert_eq!(lowered.partition.stale(&lowered.program), Some(0));
    });
    with_lowered(|mut lowered| {
        let effect = lowered.program.ops[1].effect.take();
        assert_eq!(effect, Some(EffectId::new(0)));
        assert_eq!(lowered.partition.stale(&lowered.program), Some(1));
    });
    with_lowered(|mut lowered| {
        let last = lowered.partition.ops.len() - 1;
        lowered.partition.ops.pop();
        assert_eq!(lowered.partition.stale(&lowered.program), Some(last));
    });
    with_lowered(|mut lowered| {
        let extra = lowered.partition.ops.len();
        lowered.partition.push(PartitionFact {
            op: OpId::new(extra as u32),
            kind: PartitionKind::Static,
            span: Span::new(0, 0),
        });
        assert_eq!(lowered.partition.stale(&lowered.program), Some(extra));
    });
}
