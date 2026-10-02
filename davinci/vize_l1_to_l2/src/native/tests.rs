use super::{NativeHoleKind, lower_component_native, retain_expression_in};
use vize_l0::{Allocator, Span, cstr};
use vize_l1::embed::syntax::{EmbedHole, parse_once};
use vize_l1::embed::{Embed, Grammar, Lang, Shape, prepare_attribute_value};
use vize_l2::expr::ExprRef;
use vize_l2::op::{Namespace, Op};
use vize_l2::walk::NodeRef;

#[test]
fn actual_native_component_keeps_the_once_ast_and_ordered_partial_ownership() {
    let a = Allocator::default();
    let source = "<div title=\"x\">作者 {{ value /* kept */ }}<!--tail--></div>";
    let lowered = lower_component_native(&a, source, Lang::Js).unwrap();
    assert!(lowered.is_supported());
    assert_eq!(lowered.artifact.node_count(), 4);
    assert!(core::ptr::eq(
        lowered.artifact.source().as_ptr(),
        source.as_ptr()
    ));
    let retained = &lowered.embeds.first().unwrap().syntax;
    assert_eq!(retained.comments().count(), 1);
    let ast = retained.expression().unwrap();
    let mut ids = alloc::vec::Vec::new();
    lowered
        .artifact
        .visit_nodes(&mut |id, node| {
            ids.push(id.index());
            if let NodeRef::Op(Op::Interpolation(interpolation)) = node {
                let ExprRef::Js(js) = interpolation.expression else {
                    panic!("once AST");
                };
                assert!(core::ptr::eq(js.ast, ast));
                assert_eq!(js.source, "value /* kept */");
                assert!(js.matches_authored_source(source));
            }
        })
        .unwrap();
    assert_eq!(ids, [0, 1, 2, 3]);
}

#[test]
fn actual_attribute_handoff_decodes_once_and_projects_exact_authored_entities() {
    let a = Allocator::default();
    let file = "--作者 &amp;&amp; value--";
    let authored = Span::new(2, (file.len() - 2) as u32);
    let source = prepare_attribute_value(&a, file, authored).unwrap();
    assert_eq!(source.text(), "作者 && value");
    let retained = parse_once(
        &a,
        Embed {
            grammar: Grammar {
                shape: Shape::Expr,
                lang: Lang::Js,
            },
            source,
        },
    )
    .into_expression()
    .unwrap();
    let ast = retained.expression().unwrap();
    let js = retain_expression_in(&a, file, &retained).unwrap();
    assert!(core::ptr::eq(js.ast, ast));
    assert_eq!(js.authored_span(Span::new(7, 9)), Some(Span::new(9, 19)));
    assert!(js.authored_span(Span::new(7, 8)).is_some());
    assert_eq!(js.authored_span(Span::new(1, 2)), None);
    assert!(js.matches_authored_source(file));
}

#[test]
fn missing_owner_retains_actual_admitted_descendants_without_guessing_extent() {
    let a = Allocator::default();
    let lowered = lower_component_native(&a, "<div><span>ok</span>", Lang::Js).unwrap();
    assert!(!lowered.is_supported());
    assert!(
        lowered
            .holes
            .iter()
            .any(|hole| hole.kind == NativeHoleKind::MissingMarkup)
    );
    let Some(Op::Element(element)) = lowered.artifact.root().ops.first() else {
        panic!("fragment");
    };
    assert_eq!(element.tag, "span");
    assert_eq!(element.span, Span::new(5, 20));
    assert_eq!(lowered.artifact.node_count(), 2);
    assert!(
        lowered
            .artifact
            .provenance()
            .iter()
            .any(|record| record.node.is_none() && record.after.is_empty())
    );
}

#[test]
fn unsupported_directive_does_not_parse_its_value_or_claim_native_semantics() {
    let a = Allocator::default();
    let source = "<div @click=\"a &amp;&amp; b\">{{good}}</div>";
    let lowered = lower_component_native(&a, source, Lang::Js).unwrap();
    assert!(!lowered.is_supported());
    assert!(
        lowered
            .holes
            .iter()
            .any(|hole| hole.kind == NativeHoleKind::Directive)
    );
    assert_eq!(lowered.embeds.len(), 1);
    let Some(Op::Element(element)) = lowered.artifact.root().ops.first() else {
        panic!("owner");
    };
    assert!(element.attributes.is_empty());
    assert!(element.bindings.is_empty());
    assert_eq!(lowered.artifact.node_count(), 2);
}

#[test]
fn rejected_syntax_keeps_full_parser_diagnostics_comments_and_other_fragments() {
    let a = Allocator::default();
    let lowered =
        lower_component_native(&a, "<p>before {{ value + /*kept*/ }} after</p>", Lang::Ts).unwrap();
    assert!(!lowered.is_supported());
    let retained = &lowered.embeds.first().unwrap().syntax;
    assert_eq!(retained.hole(), Some(EmbedHole::Syntax));
    assert!(retained.expression().is_none());
    assert!(retained.diagnostics().count() > 0);
    assert_eq!(retained.comments().count(), 1);
    assert_eq!(lowered.artifact.node_count(), 3);
}

#[test]
fn native_text_and_attribute_character_reference_contexts_stay_distinct() {
    let a = Allocator::default();
    let lowered =
        lower_component_native(&a, "<div title=\"&timesX\">&fjlig; &timesX</div>", Lang::Js)
            .unwrap();
    assert!(lowered.is_supported());
    let Some(Op::Element(element)) = lowered.artifact.root().ops.first() else {
        panic!("owner");
    };
    assert_eq!(element.attributes.first().unwrap().value, Some("&timesX"));
    let Some(Op::Text(text)) = element.children.ops.first() else {
        panic!("text");
    };
    assert_eq!(text.content, "fj ×X");
}

#[test]
fn svg_integration_point_and_component_children_share_factory_numbering() {
    let a = Allocator::default();
    let lowered = lower_component_native(
        &a,
        "<svg><foreignObject><div/><Child/></foreignObject></svg>",
        Lang::Js,
    )
    .unwrap();
    assert!(lowered.is_supported());
    let mut kinds = alloc::vec::Vec::new();
    lowered
        .artifact
        .visit_nodes(&mut |id, node| {
            if let NodeRef::Op(op) = node {
                kinds.push((id.index(), op.mnemonic()));
                if let Op::Element(element) = op {
                    assert_eq!(
                        element.namespace,
                        if id.index() < 2 {
                            Namespace::Svg
                        } else {
                            Namespace::Html
                        }
                    );
                }
            }
        })
        .unwrap();
    assert_eq!(
        kinds,
        [
            (0, "ui.element"),
            (1, "ui.element"),
            (2, "ui.element"),
            (3, "ui.component")
        ]
    );
}

#[test]
fn token_budget_hole_never_falls_back_to_a_second_l2_parser() {
    let a = Allocator::default();
    let source = cstr!("{{{{ {} }}}}", "x + ".repeat(40) + "x");
    let lowered = lower_component_native(&a, &source, Lang::Js).unwrap();
    let retained = &lowered.embeds.first().unwrap().syntax;
    assert_eq!(retained.hole(), Some(EmbedHole::TokenBudget));
    assert!(retained.expression().is_none());
    assert_eq!(lowered.artifact.node_count(), 0);
    assert_eq!(
        retained.source().text(),
        source
            .strip_prefix("{{ ")
            .unwrap()
            .strip_suffix(" }}")
            .unwrap()
    );
}

#[test]
fn real_component_carrier_keeps_full_surface_errors_and_partial_native_nodes() {
    let a = Allocator::default();
    let source = "<!DOCTYPE html><p>{{kept}}</p>";
    let lowered = lower_component_native(&a, source, Lang::Js).unwrap();
    assert!(!lowered.is_supported());
    assert!(lowered.component.authored.is_none());
    assert_eq!(
        vize_l1::render::check_fidelity(&lowered.component.tree),
        Ok(())
    );
    let error = lowered
        .component
        .errors
        .iter()
        .find(|error| error.code == vize_l0::ErrorCode::IncorrectlyOpenedComment)
        .unwrap();
    assert!(lowered.holes.iter().any(|hole| {
        hole.kind == NativeHoleKind::Surface(error.code)
            && hole.span == Span::new(error.offset, error.offset)
    }));
    assert_eq!(lowered.artifact.node_count(), 2);
    assert_eq!(lowered.embeds.len(), 1);
    assert!(core::ptr::eq(
        lowered.artifact.source().as_ptr(),
        source.as_ptr()
    ));
}

#[test]
fn real_component_admission_facts_survive_and_prevent_a_false_supported_result() {
    let a = Allocator::default();
    let opens: vize_l0::String = (0..64)
        .map(|index| if index % 2 == 0 { '(' } else { '[' })
        .collect();
    let closes: vize_l0::String = opens
        .chars()
        .rev()
        .map(|ch| if ch == '(' { ')' } else { ']' })
        .collect();
    let head = cstr!("v-pre:[{opens}key{closes}]");
    let source = cstr!("<div {head}>{{{{kept}}}}</div><p>tail</p>");
    let lowered = lower_component_native(&a, &source, Lang::Js).unwrap();
    assert!(!lowered.is_supported());
    let admission = lowered.component.unsupported.first().unwrap();
    assert_eq!(lowered.component.unsupported.len(), 1);
    assert_eq!(
        admission.error,
        vize_l1::markup::DirectiveNameError::NestingLimit
    );
    assert_eq!(
        source.get(admission.span.start as usize..admission.span.end as usize),
        Some(head.as_str())
    );
    assert!(lowered.holes.iter().any(|hole| {
        hole.kind == NativeHoleKind::DirectiveAdmission(admission.error)
            && hole.span == admission.span
    }));
    assert_eq!(lowered.embeds.len(), 1);
    assert_eq!(lowered.artifact.node_count(), 4);
    assert_eq!(lowered.component.tree.children.len(), 2);
}
