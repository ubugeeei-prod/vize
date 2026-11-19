use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::{Allocator, SourceRoot, Span};
use vize_l2::file::{FileArtifact, FileIssueKind};
use vize_l2::lang::js::{FileProducer, ProgramInput, ProgramScope};

pub type LawResult<T = ()> = Result<T, &'static str>;
pub trait Required<T> {
    fn required(self) -> LawResult<T>;
}
impl<T> Required<T> for Option<T> {
    fn required(self) -> LawResult<T> {
        self.ok_or("missing actual fixture node")
    }
}
impl<T, E> Required<T> for Result<T, E> {
    fn required(self) -> LawResult<T> {
        self.map_err(|_| "fixture operation rejected")
    }
}

pub fn lower<'a>(
    arena: &'a Allocator,
    file: &'a str,
    content: Span,
    profile: SourceType,
) -> LawResult<FileArtifact<'a>> {
    let raw = file
        .get(content.start as usize..content.end as usize)
        .required()?;
    let block = SourceRoot::new(file)
        .map_err(|_| "source root rejected")?
        .block(raw, content.start)
        .map_err(|_| "source block rejected")?;
    let syntax = Parser::new(arena, raw, profile).parse_observed();
    let input = ProgramInput::checked(syntax.admitted().required()?, block, 2).map_err(
        |error| match error.kind {
            FileIssueKind::InvalidProfile => "Program profile rejected",
            FileIssueKind::InvalidSource => "Program source rejected",
            _ => "Program input rejected",
        },
    )?;
    let mut producer = FileProducer::new(arena, file).map_err(|_| "File producer rejected")?;
    producer
        .program(input, ProgramScope::Module)
        .map_err(|_| "Program walk rejected")?;
    producer.finish().map_err(|_| "File finish rejected")
}
