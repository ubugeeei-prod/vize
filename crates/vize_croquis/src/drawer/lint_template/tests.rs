use super::{CompatibilityRoot, LintTemplate};
use crate::Drawer;
use oxc_span::GetSpan;
use vize_armature::Parser;
use vize_l0::Allocator;
use vize_l1_to_l2::lower_preserving_comments;
use vize_relief::{ExpressionNode, PropNode, TemplateChildNode};

const CASES: &[&str] = &[
    "<div id=app :aria-controls=panel>{{ title }}<input ref=field></div>",
    "<Card :title @save.stop=submit><template #default=\"{ item }\">{{ item.name }}</template></Card>",
    "<ul><li v-for=\"(item, index) in items\" :key=item.id>{{ index }} {{ item.name }}</li></ul>",
    "<component :is=kind v-if=visible v-model.trim=value />",
    "<RouterLink to=\"/home\"/><RouterLink/><div v-pin:top.lazy=offset></div>",
    "<section v-bind=attrs><button @click=\"count++; save(count)\">{{ $attrs.id }}</button></section>",
    "<div v-pre :raw=\"ignored\">{{ literal }}</div><span v-else-if=other>{{ other }}</span>",
    "<p>café 日本語</p><button :title=\"状態\" @click=\"保存(状態)\">{{ 状態 }}</button>",
    "<p title=\"&fjlig;&amp;&#x1f642;\">entities</p>",
    "<Child :a=\"x &amp;&amp; y\" v-slot=\"{ value = &quot;fallback&quot;, ...rest }\">{{ value }} {{ rest }}</Child>",
    "<section v-pre><div :title=\"ignored\"><span v-if=\"never\">{{ raw }}</span></div></section><p :id=\"live\">{{ live }}</p>",
    "<div v-if=\"first\"></div><span v-else-if=\"second\"></span><p v-else></p>",
    "<svg viewBox=\"0 0 10 10\"><g :[classKey]=\"group\"><circle :cx=\"x\"/></g></svg><Comp @[eventName].once=\"handle\"/>",
    "<svg><my-card></my-card></svg>",
    "<div v-pre.foo v-html='own'><Foo :innerHTML='child' /></div>",
    "<FancyPanel><template #[(slot=>slot)(outside)]=\"slot\">ready</template></FancyPanel>",
    "<Comp :column-id :number-of-months />",
];

#[test]
fn l1_l2_lint_projection_matches_relief_semantics() {
    for source in CASES {
        let allocator = Allocator::default();
        let (legacy, legacy_errors) = Parser::new(&allocator, source).parse();
        assert!(
            legacy_errors.is_empty(),
            "legacy fixture must parse: {source}"
        );
        let (surface, errors) = vize_l1::markup::parse(&allocator, source);
        assert!(errors.is_empty(), "L1 fixture must parse: {source}");
        let lowered = lower_preserving_comments(&allocator, &surface, &errors);
        let mut expected = Drawer::for_lint();
        expected.draw_template(&legacy);
        let mut actual = Drawer::for_lint();
        actual.draw_lint_template(LintTemplate::new(
            &surface,
            lowered.allocator,
            &lowered.root,
            &lowered.consumed_expressions,
            lowered.caps.scoped_slot_attrs,
        ));

        let actual = actual.finish().semantic_snapshot();
        let expected = expected.finish().semantic_snapshot();
        assert_eq!(actual, expected, "semantic mismatch for {source}");
        for scope in &actual.scopes {
            assert!(
                source.is_char_boundary(scope.range.start as usize)
                    && source.is_char_boundary(scope.range.end as usize),
                "scope range is not on UTF-8 boundaries: {:?}: {source}",
                scope.range
            );
        }
        for expression in &actual.template_expressions {
            assert!(
                source.is_char_boundary(expression.range.start as usize)
                    && source.is_char_boundary(expression.range.end as usize),
                "expression range is not on UTF-8 boundaries: {:?}: {source}",
                expression.range
            );
        }
    }
}

#[test]
fn l1_l2_static_attribute_entities_are_decoded() {
    let source = "<p title=\"&fjlig;&amp;&#x1f642;\">entities</p>";
    let allocator = Allocator::default();
    let (surface, errors) = vize_l1::markup::parse(&allocator, source);
    assert!(errors.is_empty());
    let lowered = lower_preserving_comments(&allocator, &surface, &errors);
    let root = CompatibilityRoot::new(LintTemplate::new(
        &surface,
        lowered.allocator,
        &lowered.root,
        &lowered.consumed_expressions,
        lowered.caps.scoped_slot_attrs,
    ))
    .build();
    let TemplateChildNode::Element(element) = &root.children[0] else {
        panic!("element root")
    };
    let PropNode::Attribute(attribute) = &element.props[0] else {
        panic!("title attribute")
    };
    assert_eq!(
        attribute.value.as_ref().expect("title value").content,
        "fj&🙂"
    );
}

#[test]
fn l1_l2_entity_decoding_respects_attribute_and_text_contexts() {
    let source = "<p title=\"&copy=1\">&amp</p>";
    let allocator = Allocator::default();
    let (surface, errors) = vize_l1::markup::parse(&allocator, source);
    assert!(errors.is_empty());
    let lowered = lower_preserving_comments(&allocator, &surface, &errors);
    let expected_attribute = vize_l1_to_l2::emit::decode_html_attribute_entities("&copy=1");
    let expected_text = vize_l1_to_l2::emit::decode_html_entities("&amp");

    let root = CompatibilityRoot::new(LintTemplate::new(
        &surface,
        lowered.allocator,
        &lowered.root,
        &lowered.consumed_expressions,
        lowered.caps.scoped_slot_attrs,
    ))
    .build();
    let TemplateChildNode::Element(element) = &root.children[0] else {
        panic!("compatibility element")
    };
    let PropNode::Attribute(attribute) = &element.props[0] else {
        panic!("compatibility attribute")
    };
    let TemplateChildNode::Text(text) = &element.children[0] else {
        panic!("compatibility text child")
    };

    assert_eq!(expected_attribute, "&copy=1");
    assert_eq!(expected_text, "&");
    assert_eq!(
        attribute.value.as_ref().map(|value| value.content),
        Some(expected_attribute.as_str())
    );
    assert_eq!(text.content, expected_text);
}

#[test]
fn l1_l2_slot_props_retain_decoded_authored_text() {
    let source = "<Child v-slot=\"{ value = &quot;fallback&quot;, ...rest }\">{{ value }}</Child>";
    let allocator = Allocator::default();
    let (surface, errors) = vize_l1::markup::parse(&allocator, source);
    assert!(errors.is_empty());
    let lowered = lower_preserving_comments(&allocator, &surface, &errors);
    let template = LintTemplate::new(
        &surface,
        lowered.allocator,
        &lowered.root,
        &lowered.consumed_expressions,
        lowered.caps.scoped_slot_attrs,
    );
    let root = CompatibilityRoot::new(template).build();
    let TemplateChildNode::Element(element) = &root.children[0] else {
        panic!("component root")
    };
    let directive = element
        .props
        .iter()
        .find_map(|property| match property {
            PropNode::Directive(directive) if directive.name == "slot" => Some(directive),
            _ => None,
        })
        .expect("slot directive");
    let Some(ExpressionNode::Simple(expression)) = directive.exp.as_ref() else {
        panic!("slot expression")
    };
    assert_eq!(expression.content, "{ value = \"fallback\", ...rest }");
    assert!(
        expression
            .js_ast
            .and_then(|retained| retained.as_slot_parameters())
            .is_some_and(|parameters| parameters.is_ok()),
        "decoded slot parameters must retain their owned grammar role"
    );

    let mut drawer = Drawer::for_lint();
    drawer.draw_lint_template(template);
    let snapshot = drawer.finish().semantic_snapshot();
    let slot = snapshot
        .component_usages
        .iter()
        .flat_map(|usage| &usage.slots)
        .next()
        .expect("scoped slot usage");
    assert_eq!(
        slot.scope_vars
            .iter()
            .map(|name| name.as_str())
            .collect::<Vec<_>>(),
        ["value", "rest"]
    );
}

#[test]
fn l1_l2_compatibility_retains_whitespace_wrapped_expression() {
    let source = r#"<div v-if="  !ok "/>"#;
    let allocator = Allocator::default();
    let (surface, errors) = vize_l1::markup::parse(&allocator, source);
    let lowered = lower_preserving_comments(&allocator, &surface, &errors);
    let projection = CompatibilityRoot::new(LintTemplate::new(
        &surface,
        lowered.allocator,
        &lowered.root,
        &lowered.consumed_expressions,
        lowered.caps.scoped_slot_attrs,
    ));
    let indexed = projection
        .expressions
        .iter()
        .map(|(span, expression)| (*span, expression.source()))
        .collect::<Vec<_>>();
    let root = projection.build();
    let TemplateChildNode::Element(element) = &root.children[0] else {
        panic!("element root")
    };
    let Some(PropNode::Directive(directive)) = element.props.first() else {
        panic!("v-if directive")
    };
    let Some(ExpressionNode::Simple(expression)) = directive.exp.as_ref() else {
        panic!("v-if expression")
    };
    assert!(
        expression.js_ast.is_some(),
        "indexed expressions: {indexed:?}"
    );
    let retained = expression.js_ast.unwrap().as_expression().unwrap();
    assert_eq!(expression.content, "  !ok ");
    assert_eq!(retained.ast.span(), oxc_span::Span::new(2, 5));
}

#[test]
fn l1_l2_projection_omits_tree_construction_ignored_start_tags() {
    let source = "<form><form v-html='x'><span>kept</span></form></form>";
    let allocator = Allocator::default();
    let (legacy, _) = Parser::new(&allocator, source).parse();
    let (surface, _, errors) = vize_l1::markup::parse_with_authored_diagnostics(&allocator, source);
    let lowered = lower_preserving_comments(&allocator, &surface, &errors);
    assert!(surface.children.iter().any(|child| matches!(child, vize_l1::markup::SurfaceChild::Element(element) if element.children.iter().any(|child| matches!(child, vize_l1::markup::SurfaceChild::Element(element) if element.open.is_ignored_start())))));

    let mut expected = Drawer::for_lint();
    expected.draw_template(&legacy);
    let mut actual = Drawer::for_lint();
    actual.draw_lint_template(LintTemplate::new(
        &surface,
        lowered.allocator,
        &lowered.root,
        &lowered.consumed_expressions,
        lowered.caps.scoped_slot_attrs,
    ));
    assert_eq!(
        actual.finish().semantic_snapshot(),
        expected.finish().semantic_snapshot()
    );
}
