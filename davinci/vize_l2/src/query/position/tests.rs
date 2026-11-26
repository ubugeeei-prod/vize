use crate::query::Model;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::{Allocator, SourceBlock, SourceRoot, Span};

use crate::file::{
    Declaration, L2IrBuilder, L2IrIssueKind, Namespace, TemplatePolicy, TemplateScope,
};
use crate::lang::js::{L2IrProducer, ProgramInput, ProgramScope};

use super::{L2Ir, PositionQueryError as Error};

fn append<'a>(
    arena: &'a Allocator,
    producer: &mut L2IrProducer<'a>,
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

fn file<'a>(arena: &'a Allocator, source: &'a str, source_type: SourceType) -> L2Ir<'a> {
    let mut producer = L2IrProducer::new(arena, source).unwrap();
    append(
        arena,
        &mut producer,
        SourceRoot::new(source).unwrap().whole_block(),
        0,
        source_type,
        ProgramScope::Module,
    );
    let owner = producer.finish().unwrap();
    super::index::tests::assert_equivalent(&owner);
    owner
}

fn offset(source: &str, needle: &str) -> u32 {
    source.find(needle).unwrap() as u32
}

fn refused(l2ir: &L2Ir<'_>) {
    super::index::tests::assert_equivalent(l2ir);
    let declaration = l2ir
        .bindings()
        .next()
        .and_then(|binding| binding.declaration())
        .map_or(0, |row| row.span.start);
    for at in [
        0,
        declaration,
        l2ir.artifact().source().len() as u32,
        u32::MAX,
    ] {
        assert!(matches!(
            Model::new(&l2ir).scope_at_offset(at),
            Err(Error::IncompleteFile)
        ));
        assert!(matches!(
            Model::new(&l2ir).binding_at_offset(at),
            Err(Error::IncompleteFile)
        ));
        assert!(matches!(
            Model::new(&l2ir).reference_at_offset(at),
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

#[test]
fn moving_an_owned_position_snapshot_keeps_its_file_and_queries_together() {
    let arena = Allocator::default();
    let source = "const name=1; name;";
    let file = file(&arena, source, SourceType::mjs());
    let snapshot = crate::query::PositionIndex::from_file(file);
    let moved = core::hint::black_box(snapshot);
    let binding = moved
        .model()
        .binding_at_offset(offset(source, "name;"))
        .unwrap()
        .unwrap();
    assert!(core::ptr::eq(binding.l2ir(), moved.model().l2ir()));
    assert_eq!(binding.declaration().unwrap().span, Span::new(6, 10));
    super::index::tests::assert_equivalent(&moved.into_file());
}
