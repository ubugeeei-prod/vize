use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::{Allocator, SourceBlock, SourceRoot, Span};

use crate::file::{
    Declaration, FileBuilder, FileIssueKind, Namespace, TemplatePolicy, TemplateScope,
};
use crate::lang::js::{FileProducer, ProgramInput, ProgramScope};

use super::{FileArtifact, PositionQueryError as Error};

fn append<'a>(
    arena: &'a Allocator,
    producer: &mut FileProducer<'a>,
    block: SourceBlock<'a>,
    index: usize,
    source_type: SourceType,
    scope: ProgramScope,
) {
    let syntax = Parser::new(arena, block.source(), source_type).parse_observed();
    assert!(syntax.diagnostics().is_empty());
    producer
        .program(
            ProgramInput::checked(syntax.admitted().unwrap(), block, index).unwrap(),
            scope,
        )
        .unwrap();
}

fn file<'a>(arena: &'a Allocator, source: &'a str, source_type: SourceType) -> FileArtifact<'a> {
    let mut producer = FileProducer::new(arena, source).unwrap();
    append(
        arena,
        &mut producer,
        SourceRoot::new(source).unwrap().whole_block(),
        0,
        source_type,
        ProgramScope::Module,
    );
    producer.finish().unwrap()
}

fn offset(source: &str, needle: &str) -> u32 {
    source.find(needle).unwrap() as u32
}

fn refused(owner: &FileArtifact<'_>) {
    let declaration = owner
        .bindings()
        .next()
        .and_then(|binding| binding.declaration())
        .map_or(0, |row| row.span.start);
    for at in [
        0,
        declaration,
        owner.artifact().source().len() as u32,
        u32::MAX,
    ] {
        assert!(matches!(
            owner.scope_at_offset(at),
            Err(Error::IncompleteFile)
        ));
        assert!(matches!(
            owner.binding_at_offset(at),
            Err(Error::IncompleteFile)
        ));
        assert!(matches!(
            owner.reference_at_offset(at),
            Err(Error::IncompleteFile)
        ));
    }
}

#[derive(Clone, Copy)]
struct Values;
impl TemplatePolicy for Values {
    fn visible(self, _: &Declaration) -> bool {
        true
    }
}

mod boundaries;
mod completion;
mod sites;
