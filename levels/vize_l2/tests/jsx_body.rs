#![expect(
    clippy::panic_in_result_fn,
    reason = "assertions verify completed original JSX custody and real reference ranges"
)]

mod support {
    pub mod file_jsx;
}
use fixture::{LawResult, Required};
use oxc_parser::Parser;
use oxc_span::SourceType;
use support::file_jsx as fixture;
use vize_l0::{Allocator, SourceRoot, Span};
use vize_l2::file::{FileIssueKind, Namespace, ReferenceTarget};
use vize_l2::lang::js::{JsxFile, JsxFileError, JsxFileProducer};
use vize_l2::resolution::{SyntaxKind, Usage};

fn lower<'a>(
    arena: &'a Allocator,
    file: &'a str,
    content: Span,
    profile: SourceType,
) -> LawResult<JsxFile<'a>> {
    let raw = file
        .get(content.start as usize..content.end as usize)
        .required()?;
    let block = SourceRoot::new(file)
        .required()?
        .block(raw, content.start)
        .required()?;
    let original = Parser::new(arena, raw, profile).parse_observed();
    let mut producer = JsxFileProducer::new(arena, original, block, 2).required()?;
    producer.walk().required()?;
    producer.finish().required()
}

#[test]
fn completed_owner_survives_moves_with_original_ast_comments_values_and_real_scoped_references()
-> LawResult {
    let arena = Allocator::default();
    let source = "🌸<script>/*kept*/ import UI from 'dep'; const 作者 = 1; const view = <UI.組.Button yes label=\"&amp;\" {...作者}>前&amp;{作者}<>後</></UI.組.Button>;</script>";
    let content = Span::new("🌸<script>".len() as u32, (source.len() - 9) as u32);
    for profile in [SourceType::jsx(), SourceType::tsx().with_module(true)] {
        let raw = source
            .get(content.start as usize..content.end as usize)
            .required()?;
        let block = SourceRoot::new(source)
            .required()?
            .block(raw, content.start)
            .required()?;
        let original = Parser::new(&arena, raw, profile).parse_observed();
        let body_ptr = original.admitted().required()?.program().body.as_ptr();
        let mut producer = JsxFileProducer::new(&arena, original, block, 2).required()?;
        producer.walk().required()?;
        assert_eq!(producer.walk(), Err(JsxFileError::AlreadyWalked));
        let completed = producer.finish().required()?;
        let mut owners = Vec::new();
        owners.push(completed);
        owners.reserve(128);
        let owner = owners.pop().required()?;
        assert_eq!(
            owner
                .observation()
                .admitted()
                .required()?
                .program()
                .body
                .as_ptr(),
            body_ptr
        );
        assert!(core::ptr::eq(
            owner.observation().admitted().required()?.source(),
            raw
        ));
        assert!(core::ptr::eq(owner.file().artifact().source(), source));
        assert_eq!(owner.observation().comments().len(), 1);
        assert!(!owner.observation().diagnostics().has_errors());
        assert!(owner.file().is_complete());
        let actual = owner
            .file()
            .references()
            .iter()
            .map(|reference| {
                let binding = owner
                    .file()
                    .lookup(reference.scope, reference.name.as_str(), Namespace::Value)
                    .required()?;
                Ok((
                    reference.name.as_str(),
                    reference.usage,
                    reference.target == ReferenceTarget::Resolved(binding.id()),
                ))
            })
            .collect::<LawResult<Vec<_>>>()?;
        assert_eq!(
            actual,
            [
                ("UI", Usage::Read, true),
                ("作者", Usage::Read, true),
                ("作者", Usage::Read, true)
            ]
        );
        let element = owner
            .nodes()
            .find(|node| node.kind() == Some(SyntaxKind::Element))
            .required()?;
        assert!(core::ptr::eq(element.owner(), &owner));
        assert_eq!(
            element.source(),
            Some("<UI.組.Button yes label=\"&amp;\" {...作者}>前&amp;{作者}<>後</></UI.組.Button>")
        );
        assert_eq!(element.references().required()?.len(), 3);
        let children = element
            .children()
            .map(|node| node.kind().required())
            .collect::<LawResult<Vec<_>>>()?;
        assert_eq!(
            children,
            [
                SyntaxKind::Opening {
                    self_closing: false
                },
                SyntaxKind::Text("前&amp;"),
                SyntaxKind::Container,
                SyntaxKind::Fragment,
                SyntaxKind::Closing
            ]
        );
        let opening = element.children().next().required()?;
        assert_eq!(
            opening
                .references()
                .required()?
                .iter()
                .map(|row| row.name.as_str())
                .collect::<Vec<_>>(),
            ["UI", "作者"]
        );
        let closing = element.children().last().required()?;
        assert_eq!(closing.references().required()?.len(), 0);
        let component = owner
            .nodes()
            .filter(|node| node.kind() == Some(SyntaxKind::Component("UI")))
            .map(|node| node.references().required().map(<[_]>::len))
            .collect::<LawResult<Vec<_>>>()?;
        assert_eq!(component, [1, 0]);
        let values = owner
            .nodes()
            .filter_map(|node| match node.kind()? {
                SyntaxKind::AttributeString(value) | SyntaxKind::Text(value) => {
                    Some((value, node.source()?))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            values,
            [("&amp;", "\"&amp;\""), ("前&amp;", "前&amp;"), ("後", "後")]
        );
        for node in owner.nodes() {
            let span = node.span().required()?;
            assert_eq!(
                node.source().required()?,
                source
                    .get(span.start as usize..span.end as usize)
                    .required()?
            );
            assert!(node.scope().is_some() && node.unit().is_some());
            if let Some(parent) = node.parent() {
                assert!(node.same_owner(parent));
                let range = parent.span().required()?;
                assert!(range.start <= span.start && span.end <= range.end);
            }
        }
    }
    Ok(())
}

#[test]
fn nested_function_scope_is_recorded_by_actual_file_and_foreign_node_owner_is_distinct() -> LawResult
{
    let arena = Allocator::default();
    let source = "function render(作者, value) { return <作者 enabled>{value + 1}</作者>; }";
    let one = lower(
        &arena,
        source,
        Span::new(0, source.len() as u32),
        SourceType::jsx(),
    )?;
    let two = lower(
        &arena,
        source,
        Span::new(0, source.len() as u32),
        SourceType::jsx(),
    )?;
    let element = one
        .nodes()
        .find(|node| node.kind() == Some(SyntaxKind::Element))
        .required()?;
    let foreign = two
        .nodes()
        .find(|node| node.kind() == Some(SyntaxKind::Element))
        .required()?;
    assert!(!element.same_owner(foreign));
    let unit = one.file().units().first().required()?;
    assert_ne!(element.scope().required()?, unit.scope);
    let scope = element.scope().required()?;
    let rows = element
        .references()
        .required()?
        .iter()
        .map(|row| {
            let binding = one
                .file()
                .lookup(scope, row.name.as_str(), Namespace::Value)
                .required()?;
            Ok((
                row.name.as_str(),
                row.scope == scope,
                row.target == ReferenceTarget::Resolved(binding.id()),
            ))
        })
        .collect::<LawResult<Vec<_>>>()?;
    assert_eq!(rows, [("作者", true, true), ("value", true, true)]);
    assert_eq!(one.node(usize::MAX).map(|node| node.kind()), None);
    Ok(())
}

#[test]
fn structural_file_finish_with_unresolved_semantics_does_not_mint_a_completed_body() -> LawResult {
    let arena = Allocator::default();
    let source = "/*kept*/ const view = <Missing>{value}</Missing>;";
    let before = fixture::lower(
        &arena,
        source,
        Span::new(0, source.len() as u32),
        SourceType::jsx(),
    )?;
    assert!(!before.is_complete());
    assert_eq!(
        before
            .issues()
            .iter()
            .map(|issue| issue.kind)
            .collect::<Vec<_>>(),
        [
            FileIssueKind::UnresolvedReference,
            FileIssueKind::UnresolvedReference
        ]
    );
    let block = SourceRoot::new(source).required()?.whole_block();
    let original = Parser::new(&arena, source, SourceType::jsx()).parse_observed();
    let mut producer = JsxFileProducer::new(&arena, original, block, 0).required()?;
    producer.walk().required()?;
    let rejected = producer.finish().err().required()?;
    assert_eq!(rejected.error(), JsxFileError::IncompleteFile);
    let file = rejected.file().required()?;
    assert!(!file.is_complete());
    assert_eq!(
        file.references()
            .iter()
            .map(|row| (row.name.as_str(), row.target))
            .collect::<Vec<_>>(),
        [
            ("Missing", ReferenceTarget::Unresolved),
            ("value", ReferenceTarget::Unresolved)
        ]
    );
    assert_eq!(rejected.observation().comments().len(), 1);
    assert_eq!(rejected.recorded_nodes(), 8);
    Ok(())
}

#[test]
fn unsupported_late_subtree_rolls_back_body_rows_but_keeps_prior_original_expressions() -> LawResult
{
    let arena = Allocator::default();
    let source = "import C from 'dep'; const good = <C/>; const bad = <C>{run()}<ns:tag/></C>;";
    let original = Parser::new(&arena, source, SourceType::jsx()).parse_observed();
    let block = SourceRoot::new(source).required()?.whole_block();
    let mut producer = JsxFileProducer::new(&arena, original, block, 0).required()?;
    producer.walk().required()?;
    let rejected = producer.finish().err().required()?;
    assert_eq!(rejected.error(), JsxFileError::IncompleteFile);
    assert_eq!(rejected.recorded_nodes(), 4);
    let file = rejected.file().required()?;
    assert_eq!(
        file.references()
            .iter()
            .map(|row| row.name.as_str())
            .collect::<Vec<_>>(),
        ["C"]
    );
    assert_eq!(
        file.issues()
            .iter()
            .map(|issue| issue.kind)
            .collect::<Vec<_>>(),
        [FileIssueKind::UnsupportedSyntax]
    );
    Ok(())
}

#[test]
fn pending_and_bodyless_actual_programs_cannot_seal_a_jsx_view() -> LawResult {
    let arena = Allocator::default();
    for (source, walk, expected) in [
        ("const view = <div/>;", false, JsxFileError::NotWalked),
        ("const value = 1;", true, JsxFileError::IncompleteBody),
    ] {
        let block = SourceRoot::new(source).required()?.whole_block();
        let mut producer = JsxFileProducer::new(
            &arena,
            Parser::new(&arena, source, SourceType::jsx()).parse_observed(),
            block,
            0,
        )
        .required()?;
        if walk {
            producer.walk().required()?;
        }
        let rejected = producer.finish().err().required()?;
        assert_eq!(rejected.error(), expected);
        assert!(rejected.observation().admitted().is_some());
        assert_eq!(rejected.recorded_nodes(), usize::from(walk));
    }
    Ok(())
}
