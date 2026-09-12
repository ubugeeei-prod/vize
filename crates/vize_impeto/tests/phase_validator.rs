use vize_impeto::op::{
    EdgeKind, EffectId, EffectScope, Op, OpId, OpKind, Phase, Program, Region, RegionId, StateEdge,
};
use vize_impeto::verify::{ViolationCode, verify};
use vize_s0::{Allocator, Span, String, cstr};

fn base<'a>(allocator: &'a Allocator, phase: Phase) -> Program<'a> {
    let mut program = Program::new(allocator, phase);
    program.push_region(Region::root(Span::new(0, 100)));
    program.push_op(Op::new(
        OpId::new(0),
        OpKind::If,
        RegionId::ROOT,
        Span::new(0, 100),
    ));
    program.push_region(Region::child(
        RegionId::new(1),
        RegionId::ROOT,
        OpId::new(0),
        Span::new(10, 90),
    ));
    program.push_effect(EffectScope {
        id: EffectId::new(0),
        owner: OpId::new(1),
        region: RegionId::new(1),
        span: Span::new(20, 60),
    });
    program.push_op(
        Op::new(
            OpId::new(1),
            OpKind::SetText,
            RegionId::new(1),
            Span::new(20, 30),
        )
        .with_effect(EffectId::new(0)),
    );
    program.push_op(
        Op::new(
            OpId::new(2),
            OpKind::SetProp,
            RegionId::new(1),
            Span::new(31, 40),
        )
        .with_effect(EffectId::new(0)),
    );
    program.push_edge(StateEdge::scoped(
        OpId::new(1),
        OpId::new(2),
        EdgeKind::EffectOrder,
        EffectId::new(0),
    ));
    program
}

fn messages(program: &Program<'_>) -> Vec<String> {
    verify(program)
        .into_iter()
        .map(|violation| cstr!("{violation}"))
        .collect()
}

#[test]
fn valid_program_has_no_phase_violations() {
    let arena = Allocator::default();
    assert_eq!(verify(&base(&arena, Phase::Built)), []);
    assert_eq!(verify(&base(&arena, Phase::Partitioned)), []);
    assert_eq!(verify(&base(&arena, Phase::Scheduled)), []);
}

#[test]
fn unresolved_state_edges_fail_exactly() {
    let arena = Allocator::default();
    let mut program = base(&arena, Phase::Built);
    program.push_edge(StateEdge::new(
        OpId::new(42),
        OpId::new(99),
        EdgeKind::DomOrder,
    ));

    assert_eq!(
        messages(&program)
            .into_iter()
            .filter(|line| line.starts_with("S3V004"))
            .collect::<Vec<_>>(),
        [
            "S3V004 @0:0 state edge source op#42 does not resolve",
            "S3V004 @0:0 state edge target op#99 does not resolve",
        ]
    );
}

#[test]
fn region_nesting_failures_name_the_parent_or_owner() {
    let arena = Allocator::default();
    let mut program = base(&arena, Phase::Built);
    program.push_region(Region::child(
        RegionId::new(2),
        RegionId::new(1),
        OpId::new(0),
        Span::new(80, 99),
    ));

    assert!(
        messages(&program).contains(&cstr!("S3V006 @80:99 region r#2 escapes parent region r#1"),)
    );
    assert!(messages(&program).contains(&cstr!(
        "S3V006 @80:99 region r#2 is owned by op#0, but the owner lives in r#0"
    ),));
}

#[test]
fn effect_scopes_must_contain_their_edges() {
    let arena = Allocator::default();
    let mut program = base(&arena, Phase::Built);
    program.push_op(Op::new(
        OpId::new(3),
        OpKind::SetHtml,
        RegionId::ROOT,
        Span::new(1, 2),
    ));
    program.push_edge(StateEdge::scoped(
        OpId::new(1),
        OpId::new(3),
        EdgeKind::EffectOrder,
        EffectId::new(0),
    ));

    assert!(messages(&program).contains(&cstr!(
        "S3V007 @20:60 edge op#1 -> op#3 leaves effect scope fx#0"
    ),));
}

#[test]
fn scheduled_phase_rejects_back_edges() {
    let arena = Allocator::default();
    let mut program = base(&arena, Phase::Scheduled);
    program.push_edge(StateEdge::new(
        OpId::new(2),
        OpId::new(1),
        EdgeKind::DomOrder,
    ));

    let violations = verify(&program);
    assert!(violations.iter().any(|violation| {
        violation.code == ViolationCode::ScheduledOrder
            && cstr!("{violation}")
                == cstr!("S3V008 @0:0 scheduled edge op#2 -> op#1 points backward or to itself")
    }));
}
