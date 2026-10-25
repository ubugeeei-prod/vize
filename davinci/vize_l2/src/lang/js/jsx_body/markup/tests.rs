#![expect(
    clippy::panic_in_result_fn,
    reason = "native ownership laws use exact assertions"
)]

use super::{JsxFileProducer, JsxMarkupIssue, JsxMarkupIssueKind};
use crate::{
    analysis::template_complexity,
    file::FileArtifact,
    op::{Namespace, Op},
};
use alloc::{vec, vec::Vec};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::{Allocator, SourceRoot, Span, cstr};

type Law = Result<(), &'static str>;

#[derive(Debug, PartialEq, Eq)]
enum Tree<'a> {
    Element(
        &'a str,
        Namespace,
        Vec<(&'a str, Option<&'a str>, Span)>,
        Span,
        Vec<Tree<'a>>,
    ),
    Text(&'a str, Span),
}

fn tree<'a>(ops: &[Op<'a>]) -> Result<Vec<Tree<'a>>, &'static str> {
    ops.iter()
        .map(|op| {
            Ok(match op {
                Op::Element(element) => {
                    assert_eq!(element.bindings.len(), 0);
                    Tree::Element(
                        element.tag,
                        element.namespace,
                        element
                            .attributes
                            .iter()
                            .map(|a| (a.name, a.value, a.span))
                            .collect(),
                        element.span,
                        tree(&element.children.ops)?,
                    )
                }
                Op::Text(text) => Tree::Text(text.content, text.span),
                _ => return Err("unexpected canonical op"),
            })
        })
        .collect()
}

fn span(source: &str, needle: &str) -> Result<Span, &'static str> {
    let start = source.find(needle).ok_or("expected authored token")?;
    Ok(Span::new(start as u32, (start + needle.len()) as u32))
}

fn producer<'a>(
    arena: &'a Allocator,
    source: &'a str,
    typescript: bool,
) -> Result<JsxFileProducer<'a>, &'static str> {
    let kind = if typescript {
        SourceType::tsx().with_module(true)
    } else {
        SourceType::jsx()
    };
    let block = SourceRoot::new(source).map_err(|_| "root")?.whole_block();
    let original = Parser::new(arena, source, kind).parse_observed();
    let mut producer = JsxFileProducer::new(arena, original, block, 0).map_err(|_| "producer")?;
    producer.walk().map_err(|_| "walk")?;
    Ok(producer)
}

fn facts(file: &FileArtifact<'_>) -> vize_l0::String {
    let units: Vec<_> = file
        .units()
        .iter()
        .map(|unit| {
            (
                unit.id,
                unit.span,
                unit.profile,
                unit.scope,
                unit.walk_completed(),
                unit.has_invocations(),
                unit.has_comments(),
                unit.interruption(),
            )
        })
        .collect();
    let declarations: Vec<_> = file
        .bindings()
        .filter_map(|binding| binding.declaration())
        .collect();
    cstr!(
        "{units:?}|{:?}|{declarations:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}",
        file.scopes(),
        file.references(),
        file.imports(),
        file.exports(),
        file.issues(),
        file.template_issues(),
        file.template_interruption()
    )
}

#[test]
fn same_original_file_populates_complete_static_tree_and_preserves_all_script_facts() -> Law {
    for typescript in [false, true] {
        for newline in ["\n", "\r\n"] {
            let arena = Allocator::default();
            let source = cstr!(
                "/* 😀 prefix */{newline}import Child from 'dep'; const count = 1; const copy = count; export {{ copy }};{newline}const view = <section id=\"root\" hidden aria-label=\"a\"><img src=\"a.png\" alt=\"\" />hi😀{{/*kept*/}}<br/></section>;"
            );
            let old = producer(&arena, &source, typescript)?
                .finish()
                .map_err(|_| "ordinary finish")?;
            assert_eq!(
                old.file().artifact().node_count(),
                0,
                "original gap stays reproducible through existing API"
            );
            let native = producer(&arena, &source, typescript)?;
            let body = native
                .observation()
                .admitted()
                .ok_or("admission")?
                .program()
                .body
                .as_ptr();
            let native = native.finish_markup().map_err(|_| "native markup")?;
            assert!(core::ptr::eq(
                native.file().artifact().source(),
                source.as_str()
            ));
            assert!(core::ptr::eq(native.file(), native.owner().file()));
            assert_eq!(
                native
                    .owner()
                    .observation()
                    .admitted()
                    .ok_or("retained original")?
                    .program()
                    .body
                    .as_ptr(),
                body
            );
            assert_eq!(native.owner().observation().comments().len(), 2);
            assert_eq!(
                facts(native.file()),
                facts(old.file()),
                "all retained File rows unchanged"
            );
            let outer = "<section id=\"root\" hidden aria-label=\"a\"><img src=\"a.png\" alt=\"\" />hi😀{/*kept*/}<br/></section>";
            let expected = vec![Tree::Element(
                "section",
                Namespace::Html,
                vec![
                    ("id", Some("root"), span(&source, "id=\"root\"")?),
                    ("hidden", None, span(&source, "hidden")?),
                    ("aria-label", Some("a"), span(&source, "aria-label=\"a\"")?),
                ],
                span(&source, outer)?,
                vec![
                    Tree::Element(
                        "img",
                        Namespace::Html,
                        vec![
                            ("src", Some("a.png"), span(&source, "src=\"a.png\"")?),
                            ("alt", Some(""), span(&source, "alt=\"\"")?),
                        ],
                        span(&source, "<img src=\"a.png\" alt=\"\" />")?,
                        vec![],
                    ),
                    Tree::Text("hi😀", span(&source, "hi😀")?),
                    Tree::Element(
                        "br",
                        Namespace::Html,
                        vec![],
                        span(&source, "<br/>")?,
                        vec![],
                    ),
                ],
            )];
            assert_eq!(tree(&native.file().artifact().root().ops)?, expected);
            assert_eq!(native.file().artifact().node_count(), 4);
            assert_eq!(
                template_complexity::run(native.file().artifact().root(), 4),
                template_complexity::ComplexityFacts::empty(),
                "shared analysis consumes native canonical Region"
            );
        }
    }
    Ok(())
}

#[test]
fn unsupported_static_boundary_refuses_whole_markup_before_any_partial_publication() -> Law {
    use JsxMarkupIssueKind as Kind;
    for (markup, kind, needle) in [
        ("<><div/></>", Kind::Fragment, "<><div/></>"),
        ("<Child/>", Kind::Component, "Child"),
        ("<svg/>", Kind::Namespace, "svg"),
        ("<math/>", Kind::Namespace, "math"),
        ("<div {...attrs}/>", Kind::SpreadAttribute, "{...attrs}"),
        ("<div>{...attrs}</div>", Kind::SpreadChild, "{...attrs}"),
        ("<div title={value}/>", Kind::DynamicAttribute, "{value}"),
        ("<div>{value}</div>", Kind::DynamicChild, "{value}"),
        (
            "<div v-html=\"x\"/>",
            Kind::SpecialAttribute,
            "v-html=\"x\"",
        ),
        ("<div key=\"x\"/>", Kind::SpecialAttribute, "key=\"x\""),
        (
            "<div onClick=\"x\"/>",
            Kind::SpecialAttribute,
            "onClick=\"x\"",
        ),
        (
            "<div title=\"&amp;\"/>",
            Kind::ValueNormalization,
            "\"&amp;\"",
        ),
        ("<div>&amp;</div>", Kind::ValueNormalization, "&amp;"),
        (
            "<div>first\nsecond</div>",
            Kind::ValueNormalization,
            "first\nsecond",
        ),
        (
            "<div>tab\tvalue</div>",
            Kind::ValueNormalization,
            "tab\tvalue",
        ),
        ("value && <div/>", Kind::RootExpression, "value && <div/>"),
        ("<div><Child/></div>", Kind::Component, "Child"),
    ] {
        let arena = Allocator::default();
        let source = cstr!(
            "import Child from 'dep'; const value = true; const attrs = {{}}; const first = <p/>; const view = {markup};"
        );
        let expected = producer(&arena, &source, false)?
            .finish()
            .map_err(|_| "original file")?;
        let rejected = match producer(&arena, &source, false)?.finish_markup() {
            Err(rejected) => rejected,
            Ok(_) => return Err("unexpected native admission"),
        };
        assert_eq!(
            rejected.issues(),
            &[JsxMarkupIssue {
                kind,
                span: {
                    let start = source.rfind(needle).ok_or("refusal token")?;
                    Span::new(start as u32, (start + needle.len()) as u32)
                }
            }]
        );
        let owner = rejected.owner().ok_or("complete refused original owner")?;
        assert_eq!(owner.file().artifact().node_count(), 0);
        assert_eq!(facts(owner.file()), facts(expected.file()));
        assert!(rejected.rejected_owner().is_none());
    }
    Ok(())
}

mod formatting;
mod ownership;
