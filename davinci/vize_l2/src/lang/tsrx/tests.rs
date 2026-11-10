extern crate std;

use alloc::vec::Vec;
use vize_l0::dump::{Dump, Mode};
use vize_l0::{Allocator, String};
use vize_l1::markup::tsrx::{RegionSite, parse};

use super::{TsrxIssue, TsrxIssueKind, lower};
use crate::dump::Page;
use crate::expr::ExprRef;
use crate::op::Op;

fn dumps(source: &str) -> Vec<(RegionSite, String)> {
    let allocator = Allocator::default();
    let module = parse(&allocator, source);
    let lowered = lower(&allocator, &module).unwrap();
    lowered
        .regions()
        .iter()
        .map(|region| {
            let page = Page::of(&region.artifact().root().ops);
            (region.site(), page.print_to_string(Mode::Full))
        })
        .collect()
}

fn issues(source: &str) -> Vec<(TsrxIssueKind, String)> {
    let allocator = Allocator::default();
    let module = parse(&allocator, source);
    let Err(issues) = lower(&allocator, &module) else {
        panic!("refused: {source}");
    };
    issues
        .into_iter()
        .map(|TsrxIssue { kind, span }| {
            (
                kind,
                String::from(source.get(span.start as usize..span.end as usize).unwrap()),
            )
        })
        .collect()
}

#[test]
fn elements_components_text_and_bindings_lower_in_authored_preorder() {
    let source = "export function Card({ title, attrs }: { title: string; attrs: object }) @{
\tconst label = title.trim();
\t<article class=\"card\" data-id={title} {...attrs}>
\t\t<h2 {title}>{label}</h2>
\t\t/* note */
\t\t<p>
\t\t\tPlain text, then {label.length}\tcharacters.
\t\t</p>
\t\t<>
\t\t\t<Badge.Item text={label} />{}
\t\t</>
\t\t<svg><foreignObject><div /></foreignObject></svg>
\t\t<input disabled />
\t</article>
}
";
    let regions = dumps(source);
    let [(RegionSite::FunctionBody, page)] = regions.as_slice() else {
        panic!("{regions:?}");
    };
    assert_eq!(
        page,
        r#"[l2-dump-v2]
ops=16

[l2-dump-v2.ops]
ui.element article @106:384
  attr class="card" @115:127
  ui.bind name="data-id" value=js("title" @137:142) @128:143
  ui.bind value=js("attrs" @148:153) @144:154
  ui.element h2 @158:182
    ui.bind name="title" value=js("title" @163:168) @162:169
    ui.jsx-child js("label" @171:176) @170:177
  ui.element p @198:255
    ui.text "Plain text, then " @201:222
    ui.jsx-child js("label.length" @223:235) @222:236
    ui.text " characters." @236:251
  ui.component Badge.Item @264:291
    ui.bind name="text" value=js("label" @282:287) @276:288
  ui.element svg ns=svg @302:351
    ui.element foreignObject ns=svg @307:345
      ui.element div @322:329
  ui.element input @354:372
    attr disabled @361:369

"#
    );
}

#[test]
fn nested_regions_lower_separately_and_conditionals_keep_branch_order() {
    let source = "const view = (items: string[], ok: boolean) =>
\t<ul>
\t\t{items.map((item) => <li>{item}</li>)}
\t\t@if (ok) { <b>yes</b> } @else if (items.length) { <i>some</i> } @else { <s>none</s> }
\t</ul>;
";
    let regions = dumps(source);
    // The outer child keeps the nested region's placeholder in its retained
    // expression; the nested region is its own artifact.
    let [
        (RegionSite::Expression, outer),
        (RegionSite::Expression, inner),
    ] = regions.as_slice()
    else {
        panic!("{regions:?}");
    };
    assert_eq!(
        outer,
        r#"[l2-dump-v2]
ops=9

[l2-dump-v2.ops]
ui.element ul @48:188
  ui.jsx-child js("items.map((item) => <li>{item}</li>)" @56:92) @55:93
  ui.if @96:181
    branch js("ok" @101:103) @96:119
      ui.element b @107:117
        ui.text "yes" @110:113
    branch js("items.length" @130:142) @120:159
      ui.element i @146:157
        ui.text "some" @149:153
    branch @160:181
      ui.element s @168:179
        ui.text "none" @171:175

"#
    );
    assert_eq!(
        inner,
        r#"[l2-dump-v2]
ops=2

[l2-dump-v2.ops]
ui.element li @76:91
  ui.jsx-child js("item" @81:85) @80:86

"#
    );
}

#[test]
fn retained_expressions_project_absolute_host_spans() {
    let source = "const a = <p title={x + y}>{z}</p>;";
    let allocator = Allocator::default();
    let module = parse(&allocator, source);
    let lowered = lower(&allocator, &module).unwrap();
    let [region] = lowered.regions() else {
        panic!("one region");
    };
    let [Op::Element(element)] = region.artifact().root().ops.as_slice() else {
        panic!("one element");
    };
    let [crate::op::BindingOp::Bind(bind)] = element.bindings.as_slice() else {
        panic!("one bind");
    };
    let Some(ExprRef::Js(value)) = bind.value else {
        panic!("retained value");
    };
    assert_eq!((value.source, value.span.start), ("x + y", 20));
    let oxc_ast::ast::Expression::BinaryExpression(binary) = value.ast else {
        panic!("binary");
    };
    let right = value
        .ast_span_to_source(oxc_span::GetSpan::span(&binary.right))
        .unwrap();
    assert_eq!(right, vize_l0::Span::new(4, 5));
    assert_eq!(value.authored_span(right), Some(vize_l0::Span::new(24, 25)));
}

#[test]
fn unsupported_constructs_refuse_the_module_at_their_spans() {
    let source = "function A() @{
\tconst ok = true;
\t<div title=\"a &amp; b\">
\t\t@for (const item of items) { <p>{item}</p> }
\t\t@switch (mode) { @case 'a': { <b /> } }
\t\t@try { <C /> } @catch (error) { <p>failed</p> }
\t\t<{tag} />
\t\t<style>{`.x { color: red }`}</style>
\t\t@if (ok) { const inner = 1; <p>{inner}</p> }
\t\tTom &amp; Jerry
\t</div>
}
";
    let found = issues(source);
    let expected = [
        (TsrxIssueKind::Entity, "\"a &amp; b\""),
        (
            TsrxIssueKind::For,
            "@for (const item of items) { <p>{item}</p> }",
        ),
        (
            TsrxIssueKind::Switch,
            "@switch (mode) { @case 'a': { <b /> } }",
        ),
        (
            TsrxIssueKind::Try,
            "@try { <C /> } @catch (error) { <p>failed</p> }",
        ),
        (TsrxIssueKind::DynamicTag, "{tag}"),
        (
            TsrxIssueKind::RawText,
            "<style>{`.x { color: red }`}</style>",
        ),
        (
            TsrxIssueKind::StatementContainer,
            "{ const inner = 1; <p>{inner}</p> }",
        ),
        (TsrxIssueKind::Entity, "\n\t\tTom &amp; Jerry\n\t"),
    ];
    assert_eq!(found.len(), expected.len(), "{found:?}");
    for ((kind, text), (expected_kind, expected_text)) in found.iter().zip(expected) {
        assert_eq!((*kind, text.as_str()), (expected_kind, expected_text));
    }
}

#[test]
fn syntax_errors_refuse_before_lowering() {
    let found = issues("const a = <div>;");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].0, TsrxIssueKind::Syntax);
}

/// Every clean corpus module lowers, or is refused only for a construct a
/// later slice owns; construction and coordinate checks never fail.
#[test]
fn reference_corpus_lowers_or_refuses_only_owned_constructs() {
    let directory =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/_fixtures/tsrx");
    let mut lowered = 0;
    for entry in std::fs::read_dir(&directory).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("tsrx") {
            continue;
        }
        let source = std::fs::read_to_string(&path).unwrap();
        let allocator = Allocator::default();
        let module = parse(&allocator, &source);
        assert!(module.is_clean(), "{}", path.display());
        match lower(&allocator, &module) {
            Ok(artifacts) => {
                assert_eq!(artifacts.regions().len(), module.regions().len());
                lowered += 1;
            }
            Err(issues) => {
                for issue in issues {
                    assert!(
                        !matches!(
                            issue.kind,
                            TsrxIssueKind::Syntax
                                | TsrxIssueKind::Expression(_)
                                | TsrxIssueKind::Artifact(_)
                        ),
                        "{}: {issue:?}",
                        path.display()
                    );
                }
            }
        }
    }
    assert!(lowered > 0);
}
