use vize_s0::Allocator;
use vize_s2_to_s3::{Lowered, PartitionKind, lower};
use vize_s3::op::{EdgeKind, OpKind, RegionId};
use vize_s3::verify::verify;

fn with_lowered(source: &str, check: impl FnOnce(Lowered<'_>)) {
    let allocator = Allocator::default();
    let (tree, errors) = vize_s1::parse(&allocator, source);
    let s2 = vize_s1_to_s2::lower(&allocator, &tree, &errors);
    let s3 = lower(&allocator, &s2.root);
    assert_eq!(verify(&s3.program), []);
    check(s3);
}

#[test]
fn static_element_lowers_to_static_insert_node() {
    with_lowered("<main class=\"shell\">hello</main>", |lowered| {
        assert_eq!(lowered.program.regions.len(), 2);
        assert_eq!(lowered.program.ops.len(), 2);
        assert_eq!(lowered.program.ops[0].kind, OpKind::InsertNode);
        assert_eq!(lowered.program.ops[0].region, RegionId::ROOT);
        assert_eq!(lowered.partition.ops[0].kind, PartitionKind::Static);
        assert_eq!(lowered.program.ops[1].kind, OpKind::SetText);
        assert_eq!(lowered.partition.ops[1].kind, PartitionKind::Static);
    });
}

#[test]
fn bindings_and_interpolations_emit_dynamic_facts_and_effect_order() {
    with_lowered(
        "<button :disabled=\"locked\" @click=\"save\">{{ label }}</button>",
        |lowered| {
            let kinds: Vec<OpKind> = lowered.program.ops.iter().map(|op| op.kind).collect();

            assert_eq!(
                kinds,
                [
                    OpKind::InsertNode,
                    OpKind::SetProp,
                    OpKind::SetEvent,
                    OpKind::SetText
                ]
            );
            assert_eq!(lowered.partition.ops[0].kind, PartitionKind::Static);
            assert_eq!(lowered.partition.ops[1].kind, PartitionKind::Dynamic);
            assert_eq!(lowered.partition.ops[2].kind, PartitionKind::Dynamic);
            assert_eq!(lowered.partition.ops[3].kind, PartitionKind::Dynamic);
            assert_eq!(lowered.program.effects.len(), 3);
            assert!(
                lowered
                    .program
                    .edges
                    .iter()
                    .any(|edge| edge.kind == EdgeKind::EffectOrder)
            );
        },
    );
}

#[test]
fn control_regions_make_static_children_dynamic() {
    with_lowered(
        "<section><p v-if=\"ready\">ready</p><p>always</p></section>",
        |lowered| {
            let dynamic = lowered
                .program
                .ops
                .iter()
                .zip(lowered.partition.ops.iter())
                .filter(|(_, fact)| fact.kind == PartitionKind::Dynamic)
                .map(|(op, _)| op.kind)
                .collect::<Vec<_>>();

            assert_eq!(dynamic, [OpKind::If, OpKind::InsertNode, OpKind::SetText]);
            assert_eq!(
                lowered.partition.ops.last().unwrap().kind,
                PartitionKind::Static
            );
        },
    );
}

#[test]
fn every_s3_op_receives_one_partition_fact() {
    with_lowered(
        "<slot name=\"body\"><span v-text=\"fallback\" /></slot>",
        |lowered| {
            assert_eq!(lowered.partition.ops.len(), lowered.program.ops.len());
            for op in &lowered.program.ops {
                assert_eq!(lowered.partition.get(op.id).unwrap().span, op.span);
            }
        },
    );
}
