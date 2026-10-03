#![expect(
    clippy::panic_in_result_fn,
    reason = "test assertions report complete original-Program and actual File semantic laws"
)]

mod support {
    pub mod file_jsx;
}
use oxc_span::SourceType;
use support::file_jsx::{LawResult, Required, lower};
use vize_l0::{Allocator, Span};
use vize_l2::file::{FileIssueKind, InitializerKind, Namespace, ReferenceTarget};
use vize_l2::resolution::Usage;

#[test]
fn original_jsx_and_tsx_profiles_transition_from_refusal_to_real_component_binding_facts()
-> LawResult {
    let source = "import Comp from 'dep'; let value = 1; const view = <Comp value={value}></Comp>;";
    for profile in [SourceType::jsx(), SourceType::tsx().with_module(true)] {
        let arena = Allocator::default();
        let file = lower(&arena, source, Span::new(0, source.len() as u32), profile)?;
        let unit = file.units().first().required()?;
        assert!(unit.profile.jsx && unit.profile.module);
        assert_eq!(unit.profile.typescript, profile.is_typescript());
        assert!(core::ptr::eq(file.artifact().source(), source));
        assert!(file.is_complete());
        assert_eq!(file.issues(), []);
        let comp = file
            .lookup(unit.scope, "Comp", Namespace::Value)
            .required()?;
        let value = file
            .lookup(unit.scope, "value", Namespace::Value)
            .required()?;
        let view = file
            .lookup(unit.scope, "view", Namespace::Value)
            .required()?;
        assert_eq!(
            view.declaration().required()?.initializer,
            InitializerKind::Unknown
        );
        assert_eq!(file.bindings().count(), 3);
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
                    row.namespace,
                    row.target,
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            actual,
            [
                (
                    "Comp",
                    Span::new(53, 57),
                    Usage::Read,
                    false,
                    false,
                    Namespace::Value,
                    ReferenceTarget::Resolved(comp.id())
                ),
                (
                    "value",
                    Span::new(65, 70),
                    Usage::Read,
                    false,
                    false,
                    Namespace::Value,
                    ReferenceTarget::Resolved(value.id())
                ),
            ]
        );
    }
    Ok(())
}

#[test]
fn containers_spreads_fragments_and_static_props_keep_only_real_source_ordered_reads() -> LawResult
{
    let arena = Allocator::default();
    let source = "import Comp from 'dep'; const value = 1; const props = 2; const view = <Comp yes label=\"literal\" data-id=\"x\" value={value} {...props}>text<>{value}<Comp child={value}/>{/*kept*/}{...props}</></Comp>;";
    let file = lower(
        &arena,
        source,
        Span::new(0, source.len() as u32),
        SourceType::jsx(),
    )?;
    assert!(file.is_complete());
    let scope = file.units().first().required()?.scope;
    let actual = file
        .references()
        .iter()
        .map(|row| {
            let authored = source
                .get(row.span.start as usize..row.span.end as usize)
                .required()?;
            let binding = file
                .lookup(scope, row.name.as_str(), Namespace::Value)
                .required()?;
            Ok((
                row.name.as_str(),
                authored,
                row.target == ReferenceTarget::Resolved(binding.id()),
                row.usage,
                row.shorthand,
                row.constructor,
            ))
        })
        .collect::<LawResult<Vec<_>>>()?;
    assert_eq!(
        actual,
        [
            ("Comp", "Comp", true, Usage::Read, false, false),
            ("value", "value", true, Usage::Read, false, false),
            ("props", "props", true, Usage::Read, false, false),
            ("value", "value", true, Usage::Read, false, false),
            ("Comp", "Comp", true, Usage::Read, false, false),
            ("value", "value", true, Usage::Read, false, false),
            ("props", "props", true, Usage::Read, false, false),
        ]
    );
    assert_eq!(file.bindings().count(), 4);
    assert_eq!(file.issues(), []);
    Ok(())
}

#[test]
fn original_unicode_block_and_function_scope_resolve_actual_local_component_parameters() -> LawResult
{
    let arena = Allocator::default();
    let source =
        "🌸<script>function render(作者, props) { return <作者>{props.title}</作者>; }</script>";
    let content = Span::new("🌸<script>".len() as u32, (source.len() - 9) as u32);
    let file = lower(&arena, source, content, SourceType::tsx().with_module(true))?;
    assert!(file.is_complete());
    let unit = file.units().first().required()?;
    assert_eq!(unit.span, content);
    let actual = file
        .references()
        .iter()
        .map(|row| {
            let declaration = file
                .lookup(row.scope, row.name.as_str(), Namespace::Value)
                .required()?;
            let authored = source
                .get(row.span.start as usize..row.span.end as usize)
                .required()?;
            Ok((
                row.name.as_str(),
                authored,
                row.scope == declaration.declaration().required()?.scope,
                row.target == ReferenceTarget::Resolved(declaration.id()),
                row.scope != unit.scope,
            ))
        })
        .collect::<LawResult<Vec<_>>>()?;
    assert_eq!(
        actual,
        [
            ("作者", "作者", true, true, true),
            ("props", "props", true, true, true)
        ]
    );
    assert_eq!(file.bindings().count(), 3);
    Ok(())
}

#[test]
fn unresolved_component_is_a_real_authored_use_without_intrinsic_or_context_guessing() -> LawResult
{
    let arena = Allocator::default();
    let source = "const view = <Missing/>;";
    let file = lower(
        &arena,
        source,
        Span::new(0, source.len() as u32),
        SourceType::jsx(),
    )?;
    assert!(!file.is_complete());
    let row = file.references().first().required()?;
    assert_eq!(
        (row.name.as_str(), row.span, row.target),
        ("Missing", Span::new(14, 21), ReferenceTarget::Unresolved)
    );
    assert_eq!(file.references().len(), 1);
    assert_eq!(
        file.issues()
            .iter()
            .map(|issue| (issue.kind, issue.span))
            .collect::<Vec<_>>(),
        [(FileIssueKind::UnresolvedReference, row.span)]
    );
    Ok(())
}

#[test]
fn unsupported_tag_namespace_generic_and_ts_subtrees_rollback_only_the_real_expression() -> LawResult
{
    for unsupported in [
        "<div value={value}/>",
        "<Comp.Member/>",
        "<ns:tag/>",
        "<this/>",
        "<Comp ns:attr={value}/>",
        "<Comp<Type>/>",
        "<Comp>{value as Type}</Comp>",
        "<Comp>{() => value}</Comp>",
        "<Comp><div/></Comp>",
    ] {
        let arena = Allocator::default();
        let mut source =
            vize_l0::String::from("const Comp = 1; const value = 2; value; const view = ");
        source.push_str(unsupported);
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

#[test]
fn plain_js_ts_and_actual_invocation_reference_roles_keep_their_existing_behavior() -> LawResult {
    let source = "import Maker from 'dep'; let value = 1; const made = new Maker(value); const tagged = Maker`x${value}`; const loaded = import('dep');";
    for profile in [SourceType::mjs(), SourceType::ts().with_module(true)] {
        let arena = Allocator::default();
        let file = lower(&arena, source, Span::new(0, source.len() as u32), profile)?;
        assert!(file.is_complete());
        assert_eq!(
            file.references()
                .iter()
                .map(|row| (row.name.as_str(), row.constructor))
                .collect::<Vec<_>>(),
            [
                ("Maker", true),
                ("value", false),
                ("Maker", false),
                ("value", false)
            ]
        );
        assert!(!file.units().first().required()?.profile.jsx);
    }
    Ok(())
}
