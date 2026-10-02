use crate::native::{NativeHoleKind, lower_component_native};
use vize_l0::{Allocator, Span, cstr};
use vize_l1::embed::Lang;
use vize_l1::embed::syntax::EmbedHole;
use vize_l2::expr::ExprRef;
use vize_l2::op::{BindingOp, DynamicName, Op};
use vize_l2::walk::NodeRef;

#[test]
fn actual_const_bind_patterns_preserve_once_ast_entities_and_attached_order() {
    let a = Allocator::default();
    let file = "<Child :title=\"作者 &amp;&amp; value /* kept */\" plain=\"ok\" v-bind:id=\"other\">{{child}}</Child>";
    let lowered = lower_component_native(&a, file, Lang::Ts).unwrap();
    assert!(lowered.is_supported(), "{:?}", lowered.holes);
    assert_eq!(lowered.artifact.node_count(), 4);
    assert_eq!(lowered.embeds.len(), 3);
    let mut order = alloc::vec::Vec::new();
    lowered
        .artifact
        .visit_nodes(&mut |id, node| {
            order.push((
                id.index(),
                match node {
                    NodeRef::Op(op) => op.mnemonic(),
                    NodeRef::Binding(binding) => binding.mnemonic(),
                },
            ));
        })
        .unwrap();
    assert_eq!(
        order,
        [
            (0, "ui.component"),
            (1, "ui.bind"),
            (2, "ui.bind"),
            (3, "ui.interpolation")
        ]
    );
    let Some(Op::Component(component)) = lowered.artifact.root().ops.first() else {
        panic!("component")
    };
    assert_eq!(component.attributes.first().unwrap().value, Some("ok"));
    let Some(BindingOp::Bind(binding)) = component.bindings.first() else {
        panic!("binding")
    };
    assert!(matches!(binding.name, Some(DynamicName::Static("title"))));
    let Some(ExprRef::Js(js)) = binding.value else {
        panic!("JS")
    };
    let retained = &lowered.embeds.first().unwrap().syntax;
    assert!(core::ptr::eq(js.ast, retained.expression().unwrap()));
    assert_eq!(retained.comments().count(), 1);
    assert_eq!(js.source, "作者 && value /* kept */");
    assert!(js.matches_authored_source(file));
    let start = js.span.start;
    assert_eq!(
        js.authored_span(Span::new(7, 9)),
        Some(Span::new(start + 7, start + 17))
    );
    assert_eq!(lowered.embeds.first().unwrap().node.unwrap().index(), 1);
    assert_eq!(
        lowered
            .artifact
            .provenance()
            .iter()
            .filter(|record| record.rule.as_str() == "native.bind")
            .count(),
        2
    );
}

#[test]
fn unsupported_bind_forms_never_parse_values_or_enter_the_attached_family() {
    for head in [
        ":title",
        ":title.prop=\"x\"",
        ".title=\"x\"",
        ":[key]=\"x\"",
        "v-bind=\"object\"",
        "v-bind[x]tail=\"x\"",
        "@click=\"fn(); fn()\"",
    ] {
        let a = Allocator::default();
        let source = cstr!("<div {head}>text</div>");
        let lowered = lower_component_native(&a, &source, Lang::Js).unwrap();
        assert!(!lowered.is_supported(), "{head}");
        assert!(
            lowered
                .holes
                .iter()
                .any(|hole| hole.kind == NativeHoleKind::Directive),
            "{head}: {:?}",
            lowered.holes
        );
        assert!(lowered.embeds.is_empty(), "{head}");
        let Some(Op::Element(element)) = lowered.artifact.root().ops.first() else {
            panic!("owner")
        };
        assert!(element.bindings.is_empty(), "{head}");
        assert!(matches!(element.children.ops.first(), Some(Op::Text(_))));
        assert_eq!(lowered.artifact.node_count(), 2);
    }
}

#[test]
fn a_failed_binding_keeps_full_observations_and_supported_later_fragments() {
    let a = Allocator::default();
    let source = "<div :first=\"value + /* retained */\" :second=\"ok\">tail</div>";
    let lowered = lower_component_native(&a, source, Lang::Js).unwrap();
    assert!(!lowered.is_supported());
    assert_eq!(lowered.artifact.node_count(), 3);
    let failed = lowered.embeds.first().unwrap();
    assert_eq!(failed.node, None);
    assert_eq!(failed.syntax.hole(), Some(EmbedHole::Syntax));
    assert!(failed.syntax.expression().is_none());
    assert_eq!(failed.syntax.comments().count(), 1);
    assert!(failed.syntax.diagnostics().count() > 0);
    let [_, later, ..] = lowered.embeds.as_slice() else {
        panic!("retained later binding");
    };
    assert_eq!(later.node.unwrap().index(), 1);
    let Some(Op::Element(element)) = lowered.artifact.root().ops.first() else {
        panic!("owner")
    };
    assert_eq!(element.bindings.len(), 1);
    assert!(matches!(element.children.ops.first(), Some(Op::Text(_))));
    assert!(
        lowered
            .artifact
            .provenance()
            .iter()
            .any(|record| record.node.is_none() && record.before.as_str().contains("retained"))
    );
}

#[test]
fn actual_multiscalar_entity_binding_refuses_interior_authored_edits() {
    let a = Allocator::default();
    let source = "<div :value=\"'&acE;'\"/>";
    let lowered = lower_component_native(&a, source, Lang::Js).unwrap();
    assert!(lowered.is_supported());
    let Some(Op::Element(element)) = lowered.artifact.root().ops.first() else {
        panic!("owner")
    };
    let Some(BindingOp::Bind(binding)) = element.bindings.first() else {
        panic!("binding")
    };
    let Some(ExprRef::Js(js)) = binding.value else {
        panic!("JS")
    };
    assert_eq!(js.source, "'∾̳'");
    assert!(js.matches_authored_source(source));
    assert_eq!(
        js.authored_span(Span::new(1, 6)),
        Some(Span::new(js.span.start + 1, js.span.start + 6))
    );
    assert_eq!(js.authored_span(Span::new(1, 4)), None);
    assert!(core::ptr::eq(
        js.ast,
        lowered.embeds.first().unwrap().syntax.expression().unwrap()
    ));
}

#[test]
fn unsupported_pre_carrier_preserves_complete_source_without_admitting_inner_expressions() {
    let a = Allocator::default();
    let source = "<div v-pre :x=\"exp\"><Child :y=\"inner\">{{mustache}}</Child></div><p>ok</p>";
    let lowered = lower_component_native(&a, source, Lang::Js).unwrap();
    assert!(!lowered.is_supported());
    assert!(lowered.embeds.is_empty());
    let pre = lowered
        .holes
        .iter()
        .find(|hole| hole.kind == NativeHoleKind::PreCarrier)
        .unwrap();
    assert_eq!(
        source
            .get(pre.span.start as usize..pre.span.end as usize)
            .unwrap(),
        "<div v-pre :x=\"exp\"><Child :y=\"inner\">{{mustache}}</Child></div>"
    );
    assert_eq!(lowered.artifact.node_count(), 2);
    let Some(Op::Element(element)) = lowered.artifact.root().ops.first() else {
        panic!("sibling")
    };
    assert_eq!(element.tag, "p");
}

#[test]
fn binding_token_budget_hole_retains_once_source_and_no_fallback_node() {
    let a = Allocator::default();
    let value = "x + ".repeat(40) + "x";
    let source = cstr!("<div :value=\"{value}\">tail</div>");
    let lowered = lower_component_native(&a, &source, Lang::Js).unwrap();
    assert!(!lowered.is_supported());
    assert_eq!(lowered.artifact.node_count(), 2);
    let embed = lowered.embeds.first().unwrap();
    assert_eq!(embed.node, None);
    assert_eq!(embed.syntax.hole(), Some(EmbedHole::TokenBudget));
    assert_eq!(embed.syntax.source().text(), value);
    assert!(embed.syntax.expression().is_none());
}

#[test]
fn pre_on_unsupported_owners_never_reinterprets_raw_descendant_bindings() {
    for tag in ["template", "slot", "component"] {
        let a = Allocator::default();
        let source = cstr!(
            "<b>{{{{before}}}}</b><{tag} v-pre><p :id=\"bad + /* raw */\">{{{{raw}}}}</p></{tag}><i>{{{{after}}}}</i>"
        );
        let lowered = lower_component_native(&a, &source, Lang::Js).unwrap();
        assert!(!lowered.is_supported());
        assert_eq!(lowered.embeds.len(), 2, "{tag}: {:?}", lowered.embeds);
        assert!(
            lowered
                .embeds
                .iter()
                .all(|embed| embed.syntax.hole().is_none())
        );
        assert_eq!(lowered.artifact.node_count(), 4);
        let pre = lowered
            .holes
            .iter()
            .find(|hole| hole.kind == NativeHoleKind::PreCarrier)
            .unwrap();
        let carrier = source
            .get(pre.span.start as usize..pre.span.end as usize)
            .unwrap();
        assert!(carrier.starts_with(cstr!("<{tag} v-pre>").as_str()));
        assert!(carrier.ends_with(cstr!("</{tag}>").as_str()));
        assert!(carrier.contains(":id=\"bad + /* raw */\""));
        assert_eq!(lowered.component.tree.children.len(), 3);
        assert!(lowered.component.errors.is_empty());
    }
}

#[test]
fn pre_on_missing_owners_keeps_missing_source_without_parsing_descendants() {
    for tag in ["div", "template"] {
        let a = Allocator::default();
        let source =
            cstr!("<b>{{{{before}}}}</b><{tag} v-pre><p :id=\"bad + /* raw */\">{{{{raw}}}}</p>");
        let lowered = lower_component_native(&a, &source, Lang::Js).unwrap();
        assert!(!lowered.is_supported());
        assert_eq!(lowered.embeds.len(), 1);
        assert!(lowered.embeds.first().unwrap().syntax.hole().is_none());
        assert_eq!(lowered.artifact.node_count(), 2);
        assert_eq!(lowered.component.tree.children.len(), 2);
        let Some(vize_l1::SurfaceChild::Element(carrier)) = lowered.component.tree.children.get(1)
        else {
            panic!("retained carrier");
        };
        assert!(matches!(carrier.close, vize_l1::ElementClose::Missing));
        assert!(
            lowered
                .holes
                .iter()
                .any(|hole| hole.kind == NativeHoleKind::MissingMarkup)
        );
        assert!(
            lowered
                .holes
                .iter()
                .any(|hole| hole.kind == NativeHoleKind::PreCarrier)
        );
    }
}

#[test]
fn unsupported_owner_without_pre_still_retains_supported_binding_fragments() {
    let a = Allocator::default();
    let source = "<template><p :id=\"kept\"/></template>";
    let lowered = lower_component_native(&a, source, Lang::Js).unwrap();
    assert!(!lowered.is_supported());
    assert_eq!(lowered.artifact.node_count(), 2);
    assert_eq!(lowered.embeds.len(), 1);
    assert!(lowered.embeds.first().unwrap().syntax.hole().is_none());
    assert!(
        !lowered
            .holes
            .iter()
            .any(|hole| hole.kind == NativeHoleKind::PreCarrier)
    );
    let Some(Op::Element(element)) = lowered.artifact.root().ops.first() else {
        panic!("supported fragment");
    };
    assert_eq!(element.tag, "p");
    assert_eq!(element.bindings.len(), 1);
}
