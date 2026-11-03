#![expect(
    clippy::panic_in_result_fn,
    reason = "test assertions report law failures; fixture absence is returned through Result"
)]

use alloc::vec::Vec;
use oxc_ast::ast::{Expression, Statement};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::{Allocator, Span};

use super::ProgramReferenceSource;
use crate::resolution::{
    ResolutionErrorKind, Usage,
    sink::{ReferenceEvent, ReferenceSink},
};

type LawResult<T = ()> = Result<T, &'static str>;
trait Required<T> {
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

#[derive(Default)]
struct Pending<'a> {
    rows: Vec<ReferenceEvent<'a>>,
    reject: Option<&'static str>,
}
impl<'a> ReferenceSink<'a> for Pending<'a> {
    type Checkpoint = usize;
    fn checkpoint(&self) -> usize {
        self.rows.len()
    }
    fn reference(&mut self, event: ReferenceEvent<'a>) -> Result<(), ResolutionErrorKind> {
        if self.reject == Some(event.name()) {
            return Err(ResolutionErrorKind::MissingBinding);
        }
        self.rows.push(event);
        Ok(())
    }
    fn rollback(&mut self, checkpoint: usize) {
        self.rows.truncate(checkpoint);
    }
}

fn expressions<'a>(source: &ProgramReferenceSource<'_, 'a>, sink: &mut Pending<'a>) -> LawResult {
    for statement in &source.program().body {
        let Statement::ExpressionStatement(statement) = statement else {
            return Err("expression fixture");
        };
        source.expression(&statement.expression, sink).required()?;
    }
    Ok(())
}

#[test]
fn actual_program_references_are_deferred_until_real_later_declaration_and_export() -> LawResult {
    let allocator = Allocator::default();
    let file = "<script>let local = later; const later = 1; export { local as exposed };</script>";
    let content = Span::new(8, (file.len() - 9) as u32);
    let raw = file
        .get(content.start as usize..content.end as usize)
        .required()?;
    let parsed = Parser::new(allocator.as_oxc(), raw, SourceType::mjs()).parse_observed();
    assert!(!parsed.panicked() && !parsed.diagnostics().has_errors());
    let source = ProgramReferenceSource::checked(
        parsed.admitted().required()?,
        file,
        content,
        SourceType::mjs(),
    )
    .required()?;
    let mut sink = Pending::default();
    let [
        Statement::VariableDeclaration(first),
        Statement::VariableDeclaration(later),
        Statement::ExportNamedDeclaration(export),
    ] = source.program().body.as_slice()
    else {
        return Err("actual declarations/export");
    };
    source
        .expression(
            first
                .declarations
                .first()
                .required()?
                .init
                .as_ref()
                .required()?,
            &mut sink,
        )
        .required()?;
    assert_eq!(sink.rows.first().required()?.name(), "later");
    assert!(sink.rows.first().required()?.span().end < later.span.start);
    source
        .expression(
            later
                .declarations
                .first()
                .required()?
                .init
                .as_ref()
                .required()?,
            &mut sink,
        )
        .required()?;
    source
        .export_local(&export.specifiers.first().required()?.local, &mut sink)
        .required()?;
    assert_eq!(
        sink.rows.iter().map(|row| row.name()).collect::<Vec<_>>(),
        ["later", "local"]
    );
    assert!(
        sink.rows
            .iter()
            .all(|row| row.usage() == Usage::Read && !row.shorthand() && !row.constructor())
    );
    assert_eq!(sink.rows.len(), 2); // no second expression or local-name walk
    Ok(())
}

#[test]
fn program_walk_preserves_writes_member_reads_escaped_shorthand_and_new_callee() -> LawResult {
    let allocator = Allocator::default();
    let file = r"count += value; state.key = count; ++count; ({ \u0063ount }); new Box(arg);";
    let parsed = Parser::new(allocator.as_oxc(), file, SourceType::mjs()).parse_observed();
    assert!(!parsed.panicked() && !parsed.diagnostics().has_errors());
    let source = ProgramReferenceSource::checked(
        parsed.admitted().required()?,
        file,
        Span::new(0, file.len() as u32),
        SourceType::mjs(),
    )
    .required()?;
    let mut sink = Pending::default();
    expressions(&source, &mut sink)?;
    let actual: Vec<_> = sink
        .rows
        .iter()
        .map(|row| (row.name(), row.usage(), row.shorthand(), row.constructor()))
        .collect();
    assert_eq!(
        actual,
        [
            ("count", Usage::ReadWrite, false, false),
            ("value", Usage::Read, false, false),
            ("state", Usage::Read, false, false),
            ("count", Usage::Read, false, false),
            ("count", Usage::ReadWrite, false, false),
            ("count", Usage::Read, true, false),
            ("Box", Usage::Read, false, true),
            ("arg", Usage::Read, false, false),
        ]
    );
    assert!(
        sink.rows
            .windows(2)
            .all(|pair| matches!(pair, [left, right] if left.span().end <= right.span().start))
    );
    let shorthand = sink.rows.iter().find(|row| row.shorthand()).required()?;
    assert_eq!(
        file.get(shorthand.span().start as usize..shorthand.span().end as usize),
        Some(r"\u0063ount")
    );
    Ok(())
}

#[test]
fn real_program_comments_unicode_and_block_offset_keep_exact_authored_spans() -> LawResult {
    let allocator = Allocator::default();
    let file = "🌸<script>/* before 🌸 */ 作者 + other; /* after */</script>";
    let content = Span::new("🌸<script>".len() as u32, (file.len() - 9) as u32);
    let raw = file
        .get(content.start as usize..content.end as usize)
        .required()?;
    let parsed = Parser::new(allocator.as_oxc(), raw, SourceType::mjs()).parse_observed();
    assert!(!parsed.panicked() && !parsed.diagnostics().has_errors());
    let comments = parsed.admitted().required()?.program().comments.as_ptr();
    let source = ProgramReferenceSource::checked(
        parsed.admitted().required()?,
        file,
        content,
        SourceType::mjs(),
    )
    .required()?;
    let mut sink = Pending::default();
    expressions(&source, &mut sink)?;
    assert_eq!(source.program().comments.len(), 2);
    assert_eq!(source.program().comments.as_ptr(), comments);
    for row in &sink.rows {
        let authored = source.authored_span(row.span()).required()?;
        assert_eq!(
            file.get(authored.start as usize..authored.end as usize),
            Some(row.name())
        );
    }
    assert_eq!(
        source.authored_span(Span::new(1, 2)),
        Some(Span::new(content.start + 1, content.start + 2))
    );
    let unicode = sink.rows.first().required()?.span();
    assert_eq!(
        source.authored_span(Span::new(unicode.start + 1, unicode.end)),
        None
    );
    Ok(())
}

#[test]
fn incomplete_expression_and_sink_failure_restore_previous_pending_rows() -> LawResult {
    let allocator = Allocator::default();
    let file = "seed; first + (() => missing); first + refused;";
    let parsed = Parser::new(allocator.as_oxc(), file, SourceType::mjs()).parse_observed();
    assert!(!parsed.panicked() && !parsed.diagnostics().has_errors());
    let source = ProgramReferenceSource::checked(
        parsed.admitted().required()?,
        file,
        Span::new(0, file.len() as u32),
        SourceType::mjs(),
    )
    .required()?;
    let [
        Statement::ExpressionStatement(seed),
        Statement::ExpressionStatement(incomplete),
        Statement::ExpressionStatement(rejected),
    ] = source.program().body.as_slice()
    else {
        return Err("expression fixture");
    };
    let mut sink = Pending::default();
    source.expression(&seed.expression, &mut sink).required()?;
    let original = sink.rows.clone();
    assert_eq!(
        source
            .expression(&incomplete.expression, &mut sink)
            .err()
            .required()?
            .kind,
        ResolutionErrorKind::UnsupportedSyntax
    );
    assert_eq!(sink.rows, original);
    sink.reject = Some("refused");
    assert_eq!(
        source
            .expression(&rejected.expression, &mut sink)
            .err()
            .required()?
            .kind,
        ResolutionErrorKind::MissingBinding
    );
    assert_eq!(sink.rows, original);
    Ok(())
}

#[test]
fn a_short_program_borrow_keeps_arena_semantic_names_without_a_root_handoff() -> LawResult {
    let allocator = Allocator::default();
    let file = r"\u006eame + 作者;";
    let mut sink = Pending::default();
    {
        let parsed = Parser::new(allocator.as_oxc(), file, SourceType::mjs()).parse_observed();
        assert!(!parsed.panicked() && !parsed.diagnostics().has_errors());
        let source = ProgramReferenceSource::checked(
            parsed.admitted().required()?,
            file,
            Span::new(0, file.len() as u32),
            SourceType::mjs(),
        )
        .required()?;
        expressions(&source, &mut sink)?;
    }
    assert_eq!(
        sink.rows.iter().map(|row| row.name()).collect::<Vec<_>>(),
        ["name", "作者"]
    );
    Ok(())
}

#[test]
fn invalid_unicode_endpoint_rolls_back_before_a_reference_is_visible() -> LawResult {
    let allocator = Allocator::default();
    let file = "作者;";
    let parsed = Parser::new(allocator.as_oxc(), file, SourceType::mjs()).parse_observed();
    // Mutate a private resolver probe; it never establishes Program admission.
    let mut probe = Parser::new(allocator.as_oxc(), file, SourceType::mjs()).parse();
    let Statement::ExpressionStatement(statement) = probe.program.body.first_mut().required()?
    else {
        return Err("expression fixture");
    };
    let Expression::Identifier(identifier) = &mut statement.expression else {
        return Err("identifier fixture");
    };
    identifier.span = oxc_span::Span::new(1, 6);
    let source = ProgramReferenceSource::checked(
        parsed.admitted().required()?,
        file,
        Span::new(0, file.len() as u32),
        SourceType::mjs(),
    )
    .required()?;
    let Statement::ExpressionStatement(statement) = probe.program.body.first().required()? else {
        return Err("expression fixture");
    };
    let mut sink = Pending::default();
    assert_eq!(
        source
            .expression(&statement.expression, &mut sink)
            .err()
            .required()?
            .kind,
        ResolutionErrorKind::InvalidSpan
    );
    assert!(sink.rows.is_empty());
    Ok(())
}

mod admission;
