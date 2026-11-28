use oxc_parser::{Parser, ProgramObservation};
use oxc_span::SourceType;
use vize_l0::{Allocator, SourceBlock};
use vize_l2::ir::L2Ir;
use vize_l2::lang::js::{L2IrProducer, ProgramInput, ProgramScope};

mod coverage;
mod custody;
mod geometry;
mod interruption;
mod jsx;
mod profiles;

fn moved<T>(owner: T) -> T {
    owner
}

fn observe<'a>(
    arena: &'a Allocator,
    block: SourceBlock<'a>,
    profile: SourceType,
) -> ProgramObservation<'a> {
    let original = Parser::new(arena, block.source(), profile).parse_observed();
    assert!(
        original.diagnostics().is_empty(),
        "original parser diagnostics"
    );
    original
}

fn lower<'a>(
    arena: &'a Allocator,
    original: &ProgramObservation<'a>,
    block: SourceBlock<'a>,
    index: usize,
) -> L2Ir<'a> {
    let input = ProgramInput::checked(original.admitted().unwrap(), block, index).unwrap();
    let mut producer = L2IrProducer::new(arena, block.root_source()).unwrap();
    producer.program(input, ProgramScope::Module).unwrap();
    let file = producer.finish().unwrap();
    assert!(file.is_complete(), "actual original File must complete");
    file
}
