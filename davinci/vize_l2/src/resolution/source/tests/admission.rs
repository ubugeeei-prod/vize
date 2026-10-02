use super::{LawResult, Pending, Required};
use crate::resolution::{ResolutionErrorKind, source::ProgramReferenceSource};
use oxc_ast::ast::Statement;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::{Allocator, Span, String};

#[test]
fn program_admission_rejects_equal_text_foreign_storage_profiles_and_partial_spans() -> LawResult {
    let allocator = Allocator::default();
    let file = "<script>name;</script>";
    let raw = file.get(8..13).required()?;
    let parsed = Parser::new(allocator.as_oxc(), raw, SourceType::mjs()).parse();
    let copied = String::from(raw);
    assert!(
        ProgramReferenceSource::checked(
            &parsed.program,
            copied.as_str(),
            Span::new(0, copied.len() as u32),
            SourceType::mjs()
        )
        .is_err()
    );
    assert!(
        ProgramReferenceSource::checked(&parsed.program, file, Span::new(8, 13), SourceType::ts())
            .is_err()
    );
    assert!(
        ProgramReferenceSource::checked(&parsed.program, file, Span::new(8, 12), SourceType::mjs())
            .is_err()
    );
    assert!(
        ProgramReferenceSource::checked(&parsed.program, file, Span::new(13, 8), SourceType::mjs())
            .is_err()
    );
    Ok(())
}

#[test]
fn actual_export_local_leaf_keeps_escaped_name_and_refuses_a_reexport_string() -> LawResult {
    let allocator = Allocator::default();
    let file =
        r"let name; export { \u006eame as exposed }; export { 'external' as other } from 'pkg';";
    let parsed = Parser::new(allocator.as_oxc(), file, SourceType::mjs()).parse();
    assert!(!parsed.panicked && !parsed.diagnostics.has_errors());
    let source = ProgramReferenceSource::checked(
        &parsed.program,
        file,
        Span::new(0, file.len() as u32),
        SourceType::mjs(),
    )
    .required()?;
    let [
        Statement::VariableDeclaration(_),
        Statement::ExportNamedDeclaration(local),
        Statement::ExportNamedDeclaration(reexport),
    ] = source.program().body.as_slice()
    else {
        return Err("actual export fixture");
    };
    let mut sink = Pending::default();
    source
        .export_local(&local.specifiers.first().required()?.local, &mut sink)
        .required()?;
    assert_eq!(sink.rows.first().required()?.name(), "name");
    let span = sink.rows.first().required()?.span();
    assert_eq!(
        file.get(span.start as usize..span.end as usize),
        Some(r"\u006eame")
    );
    let original = sink.rows.clone();
    assert_eq!(
        source
            .export_local(&reexport.specifiers.first().required()?.local, &mut sink)
            .err()
            .required()?
            .kind,
        ResolutionErrorKind::UnsupportedSyntax
    );
    assert_eq!(sink.rows, original);
    Ok(())
}
