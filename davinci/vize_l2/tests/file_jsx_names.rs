#![expect(
    clippy::panic_in_result_fn,
    reason = "test assertions report complete intrinsic and static member File reference facts"
)]

#[path = "support/file_jsx.rs"]
mod fixture;
use fixture::{LawResult, Required, lower};
use oxc_span::SourceType;
use vize_l0::{Allocator, Span};
use vize_l2::file::{FileIssueKind, Namespace, ReferenceTarget};
use vize_l2::resolution::Usage;

#[test]
fn intrinsic_tags_publish_only_the_real_container_reads_without_guessed_tag_bindings() -> LawResult
{
    let source = "const value = 1; const view = <div data-id='x'>{value}<my-widget/></div>;";
    for profile in [SourceType::jsx(), SourceType::tsx().with_module(true)] {
        let arena = Allocator::default();
        let file = lower(&arena, source, Span::new(0, source.len() as u32), profile)?;
        assert!(file.is_complete());
        assert_eq!(file.issues(), []);
        assert!(core::ptr::eq(file.artifact().source(), source));
        let scope = file.units().first().required()?.scope;
        let value = file.lookup(scope, "value", Namespace::Value).required()?;
        let rows = file.references();
        assert_eq!(rows.len(), 1);
        let row = rows.first().required()?;
        assert_eq!(
            (row.name.as_str(), row.span, row.usage, row.target),
            (
                "value",
                Span::new(48, 53),
                Usage::Read,
                ReferenceTarget::Resolved(value.id())
            )
        );
        assert_eq!(file.bindings().count(), 2);
        assert!(file.lookup(scope, "div", Namespace::Value).is_none());
        assert!(file.lookup(scope, "my-widget", Namespace::Value).is_none());
    }
    Ok(())
}

#[test]
fn member_tags_resolve_only_authentic_opening_roots_and_never_static_properties() -> LawResult {
    let source = "import UI from 'dep'; const ui = 1; const Button = 2; const value = 3; const view = <UI.Button a={value}><ui.Panel.Button/></UI.Button>;";
    let arena = Allocator::default();
    let file = lower(
        &arena,
        source,
        Span::new(0, source.len() as u32),
        SourceType::jsx(),
    )?;
    assert!(file.is_complete());
    assert_eq!(file.issues(), []);
    let scope = file.units().first().required()?.scope;
    let ui = file.lookup(scope, "UI", Namespace::Value).required()?;
    let lower_ui = file.lookup(scope, "ui", Namespace::Value).required()?;
    let value = file.lookup(scope, "value", Namespace::Value).required()?;
    let actual = file
        .references()
        .iter()
        .map(|row| {
            (
                row.name.as_str(),
                row.span,
                row.usage,
                row.shorthand,
                row.constructor,
                row.target,
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        actual,
        [
            (
                "UI",
                Span::new(85, 87),
                Usage::Read,
                false,
                false,
                ReferenceTarget::Resolved(ui.id())
            ),
            (
                "value",
                Span::new(98, 103),
                Usage::Read,
                false,
                false,
                ReferenceTarget::Resolved(value.id())
            ),
            (
                "ui",
                Span::new(106, 108),
                Usage::Read,
                false,
                false,
                ReferenceTarget::Resolved(lower_ui.id())
            ),
        ]
    );
    assert_eq!(file.bindings().count(), 5);
    Ok(())
}

#[test]
fn unicode_nonzero_origin_member_roots_resolve_in_the_original_function_scope() -> LawResult {
    let source = "🌸<script>function render(作者, value) { return <作者.部品>{value}</作者.部品>; }</script>尾";
    let start = source.find("function").required()?;
    let end = source.find("</script>").required()?;
    let content = Span::new(start as u32, end as u32);
    let arena = Allocator::default();
    let file = lower(&arena, source, content, SourceType::tsx().with_module(true))?;
    assert!(file.is_complete());
    assert_eq!(file.issues(), []);
    assert_eq!(file.units().first().required()?.span, content);
    assert!(core::ptr::eq(file.artifact().source(), source));
    assert_eq!(file.references().len(), 2);
    let actual = file
        .references()
        .iter()
        .map(|row| {
            Ok((
                row.name.as_str(),
                source
                    .get(row.span.start as usize..row.span.end as usize)
                    .required()?,
                row.usage,
                row.target,
            ))
        })
        .collect::<LawResult<Vec<_>>>()?;
    let first = file.references().first().required()?;
    let binding = file
        .lookup(first.scope, "作者", Namespace::Value)
        .required()?;
    let value = file
        .lookup(first.scope, "value", Namespace::Value)
        .required()?;
    assert_eq!(
        actual,
        [
            (
                "作者",
                "作者",
                Usage::Read,
                ReferenceTarget::Resolved(binding.id())
            ),
            (
                "value",
                "value",
                Usage::Read,
                ReferenceTarget::Resolved(value.id())
            ),
        ]
    );
    assert_ne!(first.scope, file.units().first().required()?.scope);
    Ok(())
}

#[test]
fn actual_unresolved_member_roots_and_remaining_refusals_retain_typed_file_issues() -> LawResult {
    let arena = Allocator::default();
    let source = "const view = <Missing.Button/>;";
    let file = lower(
        &arena,
        source,
        Span::new(0, source.len() as u32),
        SourceType::jsx(),
    )?;
    assert!(!file.is_complete());
    assert_eq!(
        file.references()
            .iter()
            .map(|row| (row.name.as_str(), row.span, row.target))
            .collect::<Vec<_>>(),
        [("Missing", Span::new(14, 21), ReferenceTarget::Unresolved)]
    );
    assert_eq!(
        file.issues()
            .iter()
            .map(|issue| issue.kind)
            .collect::<Vec<_>>(),
        [FileIssueKind::UnresolvedReference]
    );
    for refused in [
        "<this.Button/>",
        "<this.Controls.Button/>",
        "<ns:tag/>",
        "<div ns:attr={value}/>",
        "<UI.Button<Type>/>",
        "<div><UI.Button>{value as Type}</UI.Button></div>",
    ] {
        let arena = Allocator::default();
        let mut source =
            vize_l0::String::from("const UI = 1; const value = 2; value; const view = ");
        source.push_str(refused);
        source.push(';');
        let file = lower(
            &arena,
            source.as_str(),
            Span::new(0, source.len() as u32),
            SourceType::tsx().with_module(true),
        )?;
        assert!(!file.is_complete());
        assert_eq!(
            file.references()
                .iter()
                .map(|row| row.name.as_str())
                .collect::<Vec<_>>(),
            ["value"]
        );
        assert_eq!(
            file.issues()
                .iter()
                .map(|issue| issue.kind)
                .collect::<Vec<_>>(),
            [FileIssueKind::UnsupportedSyntax]
        );
        assert_eq!(file.bindings().count(), 3);
    }
    Ok(())
}
