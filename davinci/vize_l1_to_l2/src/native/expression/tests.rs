use crate::native::{NativeHoleKind, lower_component_native};
use vize_l0::{Allocator, Span};
use vize_l1::embed::Lang;
use vize_l2::expr::ExprRef;
use vize_l2::op::{BindingOp, Op};

#[test]
fn actual_interpolation_window_keeps_once_ast_entities_and_full_operation_source() {
    let a = Allocator::default();
    let file = "<p>{{ \t&fjlig; + 作者 \r\n}}</p>";
    let lowered = lower_component_native(&a, file, Lang::Js).unwrap();
    assert!(lowered.is_supported(), "{:?}", lowered.holes);
    let retained = &lowered.embeds.first().unwrap().syntax;
    assert_eq!(retained.source().text(), "fj + 作者");
    let start = file.find("&fjlig;").unwrap() as u32;
    let end = file.find(" \r\n}}").unwrap() as u32;
    assert_eq!(retained.source().span(), Span::new(start, end));
    assert!(retained.source().decode_map().is_some());
    let [Op::Element(owner)] = lowered.artifact.root().ops.as_slice() else {
        panic!("original owner");
    };
    let [Op::Interpolation(interpolation)] = owner.children.ops.as_slice() else {
        panic!("actual interpolation");
    };
    let ExprRef::Js(js) = interpolation.expression else {
        panic!("retained JS");
    };
    assert!(core::ptr::eq(js.ast, retained.expression().unwrap()));
    assert_eq!(js.span, retained.source().span());
    assert_eq!(js.source, "fj + 作者");
    assert_eq!(
        js.authored_span(Span::new(0, 2)),
        Some(Span::new(start, start + 7))
    );
    assert_eq!(js.authored_span(Span::new(0, 1)), None);
    assert!(js.matches_authored_source(file));
    let open = file.find("{{").unwrap() as u32;
    let close = file.find("}}").unwrap() as u32 + 2;
    assert_eq!(interpolation.span, Span::new(open, close));
    let provenance = lowered
        .artifact
        .provenance()
        .iter()
        .find(|record| record.rule.as_str() == "native.interpolation")
        .unwrap();
    assert_eq!(provenance.span, interpolation.span);
    assert_eq!(provenance.before.as_str(), "{{ \t&fjlig; + 作者 \r\n}}");
    assert_eq!(lowered.artifact.node_count(), 2);
}

#[test]
fn trimmed_line_comment_keeps_original_observations_and_refuses_before_mint() {
    let a = Allocator::default();
    let file = "<p>{{ value //tail \n}}{{good}}after</p>";
    let lowered = lower_component_native(&a, file, Lang::Js).unwrap();
    assert!(!lowered.is_supported());
    assert!(lowered.holes.iter().any(|hole| hole.kind
        == NativeHoleKind::ExpressionCoordinates(
            vize_l2::expr::js::JsCoordinateError::OutsideExpression
        )));
    let first = lowered.embeds.first().unwrap();
    assert_eq!(first.node, None);
    assert_eq!(first.syntax.source().text(), "value //tail");
    assert!(!first.syntax.source().text().ends_with('\n'));
    assert_eq!(first.syntax.hole(), None);
    assert!(first.syntax.expression().is_some());
    assert_eq!(
        first.syntax.comments().next().unwrap().text().unwrap(),
        "//tail"
    );
    assert_eq!(lowered.embeds.get(1).unwrap().node.unwrap().index(), 1);
    assert_eq!(lowered.artifact.node_count(), 3);
}

#[test]
fn actual_identifier_edge_nbsp_and_bom_keep_syntax_but_refuse_before_coordinates_and_mint() {
    for (prefix, suffix) in [
        ("\u{00a0}", ""),
        ("", "\u{00a0}"),
        ("\u{feff}", ""),
        ("", "\u{feff}"),
        ("&#160;", ""),
        ("", "&#65279;"),
    ] {
        let a = Allocator::default();
        let file = alloc::format!("<p>{{{{ {prefix}msg{suffix} }}}}{{{{kept}}}}</p>");
        let lowered = lower_component_native(&a, &file, Lang::Js).unwrap();
        assert!(!lowered.is_supported(), "{prefix:?}/{suffix:?}");
        assert_eq!(lowered.holes.len(), 1);
        assert_eq!(
            lowered.holes.first().unwrap().kind,
            NativeHoleKind::InterpolationIdentifierTrivia
        );
        let first = lowered.embeds.first().unwrap();
        assert_eq!(first.node, None);
        assert_eq!(first.syntax.hole(), None);
        assert_eq!(first.syntax.diagnostics().count(), 0);
        assert!(matches!(
            first.syntax.expression(),
            Some(oxc_ast::ast::Expression::Identifier(_))
        ));
        assert!(core::ptr::eq(
            lowered.artifact.source().as_ptr(),
            file.as_ptr()
        ));
        assert_eq!(lowered.embeds.get(1).unwrap().node.unwrap().index(), 1);
        assert_eq!(lowered.artifact.node_count(), 2);
    }
}

#[test]
fn characters_inside_original_comments_and_literals_do_not_trigger_edge_refusal() {
    for expression in [
        "/*\u{00a0}*/msg",
        "msg/*\u{feff}*/",
        "'\u{00a0}msg'",
        "'msg\u{feff}'",
        "/*&#160;*/msg",
        "msg/*&#65279;*/",
        "'&#160;msg'",
    ] {
        let a = Allocator::default();
        let file = alloc::format!("{{{{ {expression} }}}}");
        let lowered = lower_component_native(&a, &file, Lang::Js).unwrap();
        assert!(
            lowered.is_supported(),
            "{expression:?}: {:?}",
            lowered.holes
        );
        assert_eq!(lowered.artifact.node_count(), 1);
        let retained = &lowered.embeds.first().unwrap().syntax;
        assert_eq!(retained.hole(), None);
        let [Op::Interpolation(interpolation)] = lowered.artifact.root().ops.as_slice() else {
            panic!("original source expression");
        };
        let ExprRef::Js(js) = interpolation.expression else {
            panic!("original AST");
        };
        assert!(core::ptr::eq(js.ast, retained.expression().unwrap()));
        assert!(js.matches_authored_source(&file));
        assert_eq!(
            retained.comments().count(),
            usize::from(expression.contains("/*"))
        );
    }
}

#[test]
fn interpolation_spelling_policy_does_not_change_neutral_named_binding_admission() {
    let a = Allocator::default();
    let file = "<p :value=\"&#160;msg\">{{msg&#160;}}</p>";
    let lowered = lower_component_native(&a, file, Lang::Js).unwrap();
    assert!(!lowered.is_supported());
    assert_eq!(
        lowered.holes.first().unwrap().kind,
        NativeHoleKind::InterpolationIdentifierTrivia
    );
    let [Op::Element(owner)] = lowered.artifact.root().ops.as_slice() else {
        panic!("binding owner");
    };
    let [BindingOp::Bind(binding)] = owner.bindings.as_slice() else {
        panic!("actual named binding");
    };
    let Some(ExprRef::Js(js)) = binding.value else {
        panic!("retained bound expression");
    };
    assert_eq!(js.source, "\u{00a0}msg");
    assert_eq!(lowered.embeds.first().unwrap().node.unwrap().index(), 1);
    assert_eq!(lowered.embeds.get(1).unwrap().node, None);
    assert!(owner.children.ops.is_empty());
    assert_eq!(lowered.artifact.node_count(), 2);
}
