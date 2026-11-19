//! Exact native facts from the real L1 parse/retained-expression handoff.
//! Compatibility product paths are separate controls, not native adoption.

use vize_l0::{Allocator, Span, id::NodeId};
use vize_l1::embed::Lang;
use vize_l1_to_l2::native::lower_component_native;
use vize_l2::analysis::template_complexity::{self, ComplexityFacts, Contribution, DecisionKind};

fn span(source: &str, text: &str) -> Span {
    let start = source.find(text).unwrap() as u32;
    Span::new(start, start + text.len() as u32)
}

fn contribution(
    source: &str,
    text: &str,
    kind: DecisionKind,
    node: u32,
    nesting: u32,
) -> Contribution {
    Contribution {
        span: span(source, text),
        kind,
        op: NodeId::from_index(node),
        nesting,
        cyclomatic: 1,
        cognitive: 1,
    }
}

#[test]
fn native_retained_expressions_keep_full_facts_and_exact_authored_ranges() {
    for newline in ["\n", "\r\n"] {
        for and in ["&&", "&amp;&amp;", "&#38;&#x26;"] {
            let a = Allocator::default();
            let binding = format!("ready {and} (yes ? '😀' : 'no')");
            let interpolation = format!("alt ? (a {and} b) : c");
            let source =
                format!("作者😀{newline}<p :title=\"{binding}\">{{{{ {interpolation} }}}}</p>");
            let lowered = lower_component_native(&a, &source, Lang::Ts).unwrap();
            assert!(lowered.is_supported(), "{:?}", lowered.holes);
            assert!(lowered.diagnostics.is_empty());
            assert!(lowered.rejected_syntax.is_empty());
            assert_eq!(lowered.embeds.len(), 2);
            assert_eq!(lowered.artifact.node_count(), 4);
            let facts =
                template_complexity::run(lowered.artifact.root(), lowered.artifact.node_count());
            assert_eq!(
                facts,
                ComplexityFacts {
                    cyclomatic: 5,
                    cognitive: 4,
                    unknown: 0,
                    max_nesting: 0,
                    contributions: vec![
                        contribution(&source, &binding, DecisionKind::Logical, 2, 0),
                        contribution(
                            &source,
                            "yes ? '😀' : 'no'",
                            DecisionKind::Conditional,
                            2,
                            0,
                        ),
                        contribution(&source, &interpolation, DecisionKind::Conditional, 3, 0),
                        contribution(&source, &format!("a {and} b"), DecisionKind::Logical, 3, 1,),
                    ],
                }
            );
            // Every finished row selects the original bytes, including encoded
            // operators. No wrapper byte or decoded offset escapes into output.
            for row in &facts.contributions {
                assert!(
                    source
                        .get(row.span.start as usize..row.span.end as usize)
                        .is_some()
                );
            }
            let mut retained = 0;
            lowered
                .artifact
                .visit_nodes(&mut |_, node| {
                    let expression = match node {
                        vize_l2::walk::NodeRef::Op(vize_l2::op::Op::Interpolation(op)) => {
                            Some(op.expression)
                        }
                        vize_l2::walk::NodeRef::Binding(vize_l2::op::BindingOp::Bind(op)) => {
                            op.value
                        }
                        _ => None,
                    };
                    if let Some(vize_l2::expr::ExprRef::Js(js)) = expression {
                        let syntax = &lowered.embeds.get(retained).unwrap().syntax;
                        assert!(core::ptr::eq(js.ast, syntax.expression().unwrap()));
                        assert!(js.matches_authored_source(&source));
                        retained += 1;
                    }
                })
                .unwrap();
            assert_eq!(retained, 2);
        }
    }
}

#[test]
fn native_later_expression_holes_remain_explicit_after_scoring_admitted_fragments() {
    let a = Allocator::default();
    let source = "<p>{{ a && b }}{{ broken + }}</p>";
    let lowered = lower_component_native(&a, source, Lang::Js).unwrap();
    assert!(!lowered.is_supported());
    let holes = lowered.holes.clone();
    let diagnostics: Vec<_> = lowered
        .diagnostics
        .iter()
        .map(|d| format!("{d:?}"))
        .collect();
    let facts = template_complexity::run(lowered.artifact.root(), lowered.artifact.node_count());
    assert_eq!(
        facts,
        ComplexityFacts {
            cyclomatic: 2,
            cognitive: 1,
            unknown: 0,
            max_nesting: 0,
            contributions: vec![contribution(source, "a && b", DecisionKind::Logical, 1, 0)],
        }
    );
    assert_eq!(lowered.holes, holes);
    assert_eq!(
        lowered
            .diagnostics
            .iter()
            .map(|d| format!("{d:?}"))
            .collect::<Vec<_>>(),
        diagnostics,
    );
    assert!(!lowered.is_supported());
}
