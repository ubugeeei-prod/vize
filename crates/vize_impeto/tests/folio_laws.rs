use vize_davinci::folio::{Folio, FolioMode};
use vize_impeto::folio::S3Folio;
use vize_impeto::op::{
    EdgeKind, EffectId, EffectScope, Op, OpId, OpKind, Phase, Program, Region, RegionId, StateEdge,
};
use vize_s0::{Allocator, Span};

const CANONICAL: &str = "\
[s3-folio]
phase=scheduled

[s3-folio.regions]
id=0 parent=- owner=- span=0:80
id=1 parent=0 owner=0 span=5:70

[s3-folio.ops]
id=0 kind=impeto.if region=0 effect=- span=0:80
id=1 kind=impeto.set-text region=1 effect=0 span=10:20
id=2 kind=impeto.insert-node region=1 effect=0 span=21:30

[s3-folio.edges]
from=1 to=2 kind=effect-order effect=0

[s3-folio.effects]
id=0 owner=1 region=1 span=10:30

";

fn program<'a>(allocator: &'a Allocator) -> Program<'a> {
    let mut program = Program::new(allocator, Phase::Scheduled);
    program.push_region(Region::root(Span::new(0, 80)));
    program.push_op(Op::new(
        OpId::new(0),
        OpKind::If,
        RegionId::ROOT,
        Span::new(0, 80),
    ));
    program.push_region(Region::child(
        RegionId::new(1),
        RegionId::ROOT,
        OpId::new(0),
        Span::new(5, 70),
    ));
    program.push_effect(EffectScope {
        id: EffectId::new(0),
        owner: OpId::new(1),
        region: RegionId::new(1),
        span: Span::new(10, 30),
    });
    program.push_op(
        Op::new(
            OpId::new(1),
            OpKind::SetText,
            RegionId::new(1),
            Span::new(10, 20),
        )
        .with_effect(EffectId::new(0)),
    );
    program.push_op(
        Op::new(
            OpId::new(2),
            OpKind::InsertNode,
            RegionId::new(1),
            Span::new(21, 30),
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

#[test]
fn full_print_is_identity_on_canonical_text() {
    let folio = S3Folio::parse(CANONICAL).expect("canonical text parses");
    assert_eq!(folio.print_to_string(FolioMode::Full).as_str(), CANONICAL);
}

#[test]
fn parse_print_is_structural_identity() {
    let arena = Allocator::default();
    let folio = S3Folio::of(&program(&arena));
    let printed = folio.print_to_string(FolioMode::Full);
    assert_eq!(printed.as_str(), CANONICAL);
    assert_eq!(S3Folio::parse(printed.as_str()).unwrap(), folio);
}
