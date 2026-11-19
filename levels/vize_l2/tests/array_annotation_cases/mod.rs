use oxc_parser::{Parser, ProgramObservation};
use oxc_span::SourceType;
use vize_l0::{Allocator, SourceBlock, SourceRoot, String, cstr};
use vize_l2::file::{
    FileArtifact, FileIssueKind, InitializerKind, Namespace, PositionQueryError, ScriptUnit,
};
use vize_l2::lang::js::{FileProducer, ProgramInput, ProgramScope};

fn lower<'a>(
    arena: &'a Allocator,
    source: &'a str,
    block: SourceBlock<'a>,
    observed: &ProgramObservation<'a>,
) -> FileArtifact<'a> {
    let mut producer = FileProducer::new(arena, source).unwrap();
    producer
        .program(
            ProgramInput::checked(observed.admitted().unwrap(), block, 7).unwrap(),
            ProgramScope::Module,
        )
        .unwrap();
    producer.finish().unwrap()
}

mod custody;
mod forms;
mod navigation;
