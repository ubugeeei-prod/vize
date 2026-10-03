//! Authored ranges refuse malformed UTF16 and generated suffix diagnostics.

use super::{MappingError, map_range};
use lsp_types::{Position, Range};
use vize_l0::{Allocator, SourceRoot, Span};
use vize_l1::embed::{
    EmbedSource, Lang,
    syntax::{ProgramOptions, parse_program_once},
};
use vize_l2::lang::js::{FileProducer, ProgramInput, ProgramScope};
use vize_l4::targets::ts::project_program;

#[test]
fn authored_lsp_ranges_preserve_newlines_and_refuse_bad_or_generated_positions() {
    for newline in ["\n", "\r", "\r\n"] {
        let arena = Allocator::default();
        let source = vize_l0::cstr!("const x='😀';{newline}x.missing;");
        let block = SourceRoot::new(&source).unwrap().whole_block();
        let syntax = parse_program_once(
            &arena,
            EmbedSource::authored(&source, block.span()).unwrap(),
            ProgramOptions::module(Lang::Js),
        );
        let mut producer = FileProducer::new(&arena, &source).unwrap();
        producer
            .program(
                ProgramInput::checked(syntax.admitted_program().unwrap(), block, 17).unwrap(),
                ProgramScope::Module,
            )
            .unwrap();
        let file = producer.finish().unwrap();
        let projection = project_program(&file).unwrap();
        let start = u32::try_from(source.find("missing").unwrap()).unwrap();
        assert_eq!(
            map_range(
                &projection,
                Range::new(Position::new(1, 2), Position::new(1, 9))
            ),
            Ok(Span::new(start, start + 7))
        );
        assert_eq!(
            map_range(
                &projection,
                Range::new(Position::new(0, 10), Position::new(0, 11))
            ),
            Err(MappingError::InvalidUtf16Boundary)
        );
        assert_eq!(
            map_range(
                &projection,
                Range::new(Position::new(1, 9), Position::new(1, 2))
            ),
            Err(MappingError::InvalidRange)
        );
        assert_eq!(
            map_range(
                &projection,
                Range::new(Position::new(3, 0), Position::new(3, 6))
            ),
            Err(MappingError::GeneratedOnly)
        );
        assert_eq!(
            map_range(
                &projection,
                Range::new(Position::new(1, 2), Position::new(3, 6))
            ),
            Err(MappingError::CrossesBoundary)
        );
        assert_eq!(
            map_range(
                &projection,
                Range::new(Position::new(1, 100), Position::new(1, 100))
            ),
            Err(MappingError::InvalidUtf16Boundary)
        );
    }
}
