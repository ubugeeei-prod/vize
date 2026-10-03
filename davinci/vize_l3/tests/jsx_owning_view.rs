use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::{Allocator, SourceRoot, Span};
use vize_l2::file::ReferenceTarget;
use vize_l2::lang::js::{JsxFile, JsxFileProducer};
use vize_l3::jsx::{JsxDecisionKind as Decision, JsxIssueKind as Issue, build_jsx_decisions};

type LawResult<T = ()> = Result<T, &'static str>;
trait Required<T> {
    fn required(self) -> LawResult<T>;
}
impl<T> Required<T> for Option<T> {
    fn required(self) -> LawResult<T> {
        self.ok_or("missing actual owned fixture fact")
    }
}
impl<T, E> Required<T> for Result<T, E> {
    fn required(self) -> LawResult<T> {
        self.map_err(|_| "actual fixture operation refused")
    }
}
fn equal<T: PartialEq>(actual: T, expected: T) -> LawResult {
    if actual == expected {
        Ok(())
    } else {
        Err("owned fact differs")
    }
}
fn lower<'a>(arena: &'a Allocator, source: &'a str, profile: SourceType) -> LawResult<JsxFile<'a>> {
    embedded(arena, source, Span::new(0, source.len() as u32), profile)
}
fn embedded<'a>(
    arena: &'a Allocator,
    source: &'a str,
    content: Span,
    profile: SourceType,
) -> LawResult<JsxFile<'a>> {
    let raw = source
        .get(content.start as usize..content.end as usize)
        .required()?;
    let block = SourceRoot::new(source)
        .required()?
        .block(raw, content.start)
        .required()?;
    let mut producer = JsxFileProducer::new(
        arena,
        Parser::new(arena, raw, profile).parse_observed(),
        block,
        3,
    )
    .required()?;
    producer.walk().required()?;
    producer.finish().required()
}

#[test]
fn exact_authored_structure_static_values_and_empty_comments_project_in_order() -> LawResult {
    let arena = Allocator::default();
    let source =
        "/*kept*/ const view = <div id='&amp;' disabled>前&amp;{/*empty*/}<my-widget/></div>;";
    let analysis = build_jsx_decisions(lower(&arena, source, SourceType::jsx())?).required()?;
    let actual = analysis
        .decisions()
        .map(|row| row.kind())
        .collect::<Vec<_>>();
    equal(
        actual,
        vec![
            Decision::Root,
            Decision::Element,
            Decision::Opening {
                self_closing: false,
            },
            Decision::Intrinsic("div"),
            Decision::StaticAttribute {
                name: "id",
                value: Some("&amp;"),
            },
            Decision::AttributeName("id"),
            Decision::AttributeString("&amp;"),
            Decision::StaticAttribute {
                name: "disabled",
                value: None,
            },
            Decision::AttributeName("disabled"),
            Decision::Text("前&amp;"),
            Decision::EmptyContainer,
            Decision::Empty,
            Decision::Element,
            Decision::Opening { self_closing: true },
            Decision::Intrinsic("my-widget"),
            Decision::Closing,
            Decision::Intrinsic("div"),
        ],
    )?;
    equal(analysis.owner().observation().comments().len(), 2)?;
    for row in analysis.decisions() {
        equal(core::ptr::eq(row.node().owner(), analysis.owner()), true)?;
        let span = row.node().span().required()?;
        equal(
            row.node().source(),
            source.get(span.start as usize..span.end as usize),
        )?;
    }
    Ok(())
}

#[test]
fn moved_analysis_retains_original_program_file_and_complete_component_roots() -> LawResult {
    let arena = Allocator::default();
    let source = "import UI from 'dep'; import C from 'other'; const view = <UI.Panel.Button><C/></UI.Panel.Button>;";
    let owner = lower(&arena, source, SourceType::jsx())?;
    let original = owner
        .observation()
        .admitted()
        .required()?
        .program()
        .body
        .as_ptr();
    let mut values = Vec::new();
    values.push(build_jsx_decisions(owner).required()?);
    values.reserve(256);
    let analysis = values.pop().required()?;
    equal(
        analysis
            .owner()
            .observation()
            .admitted()
            .required()?
            .program()
            .body
            .as_ptr(),
        original,
    )?;
    equal(
        core::ptr::eq(analysis.owner().file().artifact().source(), source),
        true,
    )?;
    equal(analysis.owner().file().is_complete(), true)?;
    let components = analysis
        .decisions()
        .filter_map(|row| match row.kind() {
            Decision::Component(name) => Some((name, row.node().references().map(<[_]>::len))),
            _ => None,
        })
        .collect::<Vec<_>>();
    equal(
        components,
        vec![("UI", Some(1)), ("C", Some(1)), ("UI", Some(0))],
    )?;
    equal(
        analysis
            .owner()
            .file()
            .references()
            .iter()
            .all(|row| matches!(row.target, ReferenceTarget::Resolved(_))),
        true,
    )
}

#[test]
fn jsx_and_tsx_function_scopes_keep_their_own_resolved_component_authority() -> LawResult {
    let source = "function render(作者) { return <作者.部品 title='文'/>; }";
    for profile in [SourceType::jsx(), SourceType::tsx().with_module(true)] {
        let arena = Allocator::default();
        let analysis = build_jsx_decisions(lower(&arena, source, profile)?).required()?;
        let component = analysis
            .decisions()
            .find(|row| row.kind() == Decision::Component("作者"))
            .required()?;
        let row = component
            .node()
            .references()
            .required()?
            .first()
            .required()?;
        equal(
            row.scope == analysis.owner().file().units().first().required()?.scope,
            false,
        )?;
        equal(matches!(row.target, ReferenceTarget::Resolved(_)), true)?;
        equal(row.span, component.node().span().required()?)?;
    }
    Ok(())
}

#[test]
fn equal_source_bytes_and_local_indices_never_authenticate_a_foreign_owner() -> LawResult {
    let arena = Allocator::default();
    let source = "const view = <div/>;";
    let one = build_jsx_decisions(lower(&arena, source, SourceType::jsx())?).required()?;
    let two = build_jsx_decisions(lower(&arena, source, SourceType::jsx())?).required()?;
    let left = one.decisions().next().required()?.node();
    let right = two.decisions().next().required()?.node();
    equal(left.span(), right.span())?;
    equal(left.source(), right.source())?;
    equal(left.same_owner(right), false)
}

#[test]
fn unsupported_constructs_refuse_whole_view_and_retain_original_custody() -> LawResult {
    for (source, issue) in [
        ("const view = <><div/></>;", Issue::Fragment),
        ("const view = <div>{1 + 2}</div>;", Issue::Expression),
        (
            "const x = 1; const view = <div {...x}/>;",
            Issue::SpreadAttribute,
        ),
        (
            "const x = 1; const view = <div>{...x}</div>;",
            Issue::SpreadChild,
        ),
        ("const view = <div v-show/>;", Issue::DirectiveOrSlot),
        ("const view = <div v-slots/>;", Issue::DirectiveOrSlot),
        ("const view = <div onClick='x'/>;", Issue::Attribute),
        (
            "const x = 1; const view = <div id={x + 1}/>;",
            Issue::Attribute,
        ),
    ] {
        let arena = Allocator::default();
        let owner = lower(&arena, source, SourceType::jsx())?;
        let original = owner
            .observation()
            .admitted()
            .required()?
            .program()
            .body
            .as_ptr();
        let rejected = build_jsx_decisions(owner).err().required()?;
        equal(
            rejected
                .owner()
                .observation()
                .admitted()
                .required()?
                .program()
                .body
                .as_ptr(),
            original,
        )?;
        equal(rejected.owner().file().is_complete(), true)?;
        equal(rejected.issues().iter().any(|row| row.kind == issue), true)?;
        for row in rejected.issues() {
            equal(
                source
                    .get(row.span.start as usize..row.span.end as usize)
                    .is_some(),
                true,
            )?;
        }
    }
    Ok(())
}

#[test]
fn complete_original_non_jsx_expressions_keep_custody_without_jsx_reinterpretation() -> LawResult {
    let arena = Allocator::default();
    let source = "const before = 1 + 2; const view = <div/>;";
    let owner = lower(&arena, source, SourceType::jsx())?;
    equal(owner.file().is_complete(), true)?;
    let admitted = build_jsx_decisions(owner).required()?;
    equal(
        admitted
            .decisions()
            .filter(|row| row.kind() == Decision::OriginalExpression)
            .map(|row| row.node().source().unwrap_or_default())
            .collect::<Vec<_>>(),
        vec!["1 + 2", "1", "2"],
    )?;
    equal(admitted.owner().file().artifact().source(), source)
}

#[test]
fn embedded_unicode_offsets_link_to_whole_original_file_without_reslicing_identity() -> LawResult {
    let arena = Allocator::default();
    let source = "🌸<script>const view = <div title='後'>前</div>;</script>尾";
    let start = "🌸<script>".len() as u32;
    let end = source.find("</script>").required()? as u32;
    let owner = embedded(&arena, source, Span::new(start, end), SourceType::jsx())?;
    let analysis = build_jsx_decisions(owner).required()?;
    equal(
        analysis.owner().file().units().first().required()?.span,
        Span::new(start, end),
    )?;
    for row in analysis.decisions() {
        let span = row.node().span().required()?;
        equal(span.start >= start && span.end <= end, true)?;
        equal(
            row.node().source(),
            source.get(span.start as usize..span.end as usize),
        )?;
    }
    Ok(())
}

#[test]
fn unresolved_original_components_fail_lower_sealing_before_any_upper_view() -> LawResult {
    let arena = Allocator::default();
    let source = "/*kept*/ const view = <Missing/>;";
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
    equal(rejected.file().required()?.is_complete(), false)?;
    equal(rejected.observation().comments().len(), 1)
}
