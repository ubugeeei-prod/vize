#![expect(
    clippy::panic_in_result_fn,
    reason = "assertions bind rejected original syntax/profile/custody to real owners"
)]

mod support {
    pub mod file_jsx;
}
use oxc_parser::{ParseOptions, Parser};
use oxc_span::SourceType;
use support::file_jsx::{LawResult, Required, lower};
use vize_l0::{Allocator, SourceRoot, Span, String};
use vize_l2::file::FileIssueKind;
use vize_l2::lang::js::{JsxFileError, JsxFileProducer, ProgramInputError};

#[test]
fn exact_copied_and_foreign_block_sources_never_pair_original_program_with_another_file()
-> LawResult {
    let arena = Allocator::default();
    let source = "/*keep*/ const view = <div/>;";
    let copy = String::from(source);
    for candidate in [copy.as_str(), "/*keep*/ const view = <span/>;"] {
        let block = SourceRoot::new(candidate).required()?.whole_block();
        let observation = Parser::new(&arena, source, SourceType::jsx()).parse_observed();
        let rejected = JsxFileProducer::new(&arena, observation, block, 0)
            .err()
            .required()?;
        assert_eq!(
            rejected.error(),
            JsxFileError::Input(ProgramInputError {
                span: block.span(),
                kind: FileIssueKind::InvalidSource
            })
        );
        assert!(core::ptr::eq(
            rejected.observation().admitted().required()?.source(),
            source
        ));
        assert_eq!(rejected.observation().comments().len(), 1);
        assert_eq!(rejected.recorded_nodes(), 0);
        assert!(rejected.file().is_none() && rejected.rejected_file().is_none());
    }
    Ok(())
}

#[test]
fn stock_original_jsx_profile_is_required_and_recovered_observation_keeps_diagnostics() -> LawResult
{
    let arena = Allocator::default();
    for (source, profile, options, expected) in [
        (
            "const view = <div/>;",
            SourceType::tsx(),
            ParseOptions::default(),
            JsxFileError::Input(ProgramInputError {
                span: Span::new(0, 20),
                kind: FileIssueKind::InvalidProfile,
            }),
        ),
        (
            "const value = 1;",
            SourceType::mjs(),
            ParseOptions::default(),
            JsxFileError::InvalidProfile,
        ),
        (
            "const view = <div/>;",
            SourceType::jsx(),
            ParseOptions {
                allow_return_outside_function: true,
                ..ParseOptions::default()
            },
            JsxFileError::Input(ProgramInputError {
                span: Span::new(0, 20),
                kind: FileIssueKind::InvalidProfile,
            }),
        ),
        (
            "const view = <div>{/uv}</div>;",
            SourceType::jsx(),
            ParseOptions::default(),
            JsxFileError::SyntaxAdmission,
        ),
    ] {
        let observation = Parser::new(&arena, source, profile)
            .with_options(options)
            .parse_observed();
        let original_errors = observation.diagnostics().has_errors();
        let block = SourceRoot::new(source).required()?.whole_block();
        let rejected = JsxFileProducer::new(&arena, observation, block, 0)
            .err()
            .required()?;
        assert_eq!(rejected.error(), expected);
        assert_eq!(
            rejected.observation().diagnostics().has_errors(),
            original_errors
        );
        assert_eq!(
            rejected.observation().admitted().is_none(),
            expected == JsxFileError::SyntaxAdmission
        );
        assert_eq!(rejected.recorded_nodes(), 0);
    }
    Ok(())
}

#[test]
fn genuine_work_limit_rolls_back_original_body_and_keeps_prior_completed_expression() -> LawResult {
    let arena = Allocator::default();
    let mut source = String::from("const good = <div/>; const many = <div>");
    for _ in 0..1500 {
        source.push_str("<span/>");
    }
    source.push_str("</div>;");
    let source = source.as_str();
    let block = SourceRoot::new(source).required()?.whole_block();
    let mut producer = JsxFileProducer::new(
        &arena,
        Parser::new(&arena, source, SourceType::jsx()).parse_observed(),
        block,
        0,
    )
    .required()?;
    producer.walk().required()?;
    let rejected = producer.finish().err().required()?;
    assert_eq!(rejected.error(), JsxFileError::IncompleteFile);
    assert_eq!(rejected.recorded_nodes(), 4);
    assert_eq!(rejected.file().required()?.references().len(), 0);
    assert_eq!(
        rejected
            .file()
            .required()?
            .issues()
            .iter()
            .map(|issue| issue.kind)
            .collect::<Vec<_>>(),
        [FileIssueKind::BindingLimit]
    );
    Ok(())
}

#[test]
fn generic_completed_jsx_file_has_no_original_syntax_owning_body_capability() -> LawResult {
    let arena = Allocator::default();
    let source = "/*original*/ const view = <div>text</div>;";
    let file = lower(
        &arena,
        source,
        Span::new(0, source.len() as u32),
        SourceType::jsx(),
    )?;
    assert!(file.is_complete());
    assert_eq!(file.units().len(), 1);
    assert_eq!(file.references().len(), 0);
    assert_eq!(file.artifact().node_count(), 0);
    // Its semantic owner is real; it has no public body records or syntax owner.
    // Only the separate original-owning factory can establish the new view.
    Ok(())
}
