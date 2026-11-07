use vize_l0::{Allocator, SourceRoot};
use vize_l1::markup::document::NativeDocument;
use vize_l2::dialect::petite::PetiteDocument;
use vize_relief::{ExpressionNode, PropNode, TemplateChildNode};

use super::{Drawer, collect_petite_semantic_reuse};
use crate::drawer::{DrawerOptions, helpers::extract_v_scope_bindings};

#[test]
fn petite_hints_are_native_owned_and_preserve_the_complete_croquis() {
    for source in [
        r#"<div v-scope="{ count: 0, msg: 'x' }" v-effect="count > 0">{{ count }} {{ msg }}</div>"#,
        r#"<main v-scope><i v-scope="count"></i><b v-effect=""></b></main>"#,
        r#"<section v-scope="({ 'label': '&amp;', total: 1 })"><button v-effect="total++"></button></section>"#,
        r#"<div v-scope="{ count: 0 } // comment">{{ count }}</div>"#,
        r#"<div v-scope="{ count: 0 }, state = { count: 1 }">{{ count }}</div>"#,
    ] {
        let allocator = Allocator::new();
        let (root, errors) = vize_armature::parse(&allocator, source);
        assert!(errors.is_empty(), "legacy fixture must parse: {errors:?}");
        let owner = NativeDocument::lex_in(
            &allocator,
            SourceRoot::new(source).expect("bounded fixture"),
        );
        let document = PetiteDocument::lower_in(&allocator, &owner).expect("native fixture");
        let mut reuse = collect_petite_semantic_reuse(&root, &document).expect("petite directives");
        assert!(reuse.authenticate(&root));

        if source.contains("count: 0, msg") {
            let TemplateChildNode::Element(element) = &root.children[0] else {
                panic!("root element");
            };
            let directives: Vec<_> = element
                .props
                .iter()
                .filter_map(|prop| match prop {
                    PropNode::Directive(directive) => Some(directive.as_ref()),
                    PropNode::Attribute(_) => None,
                })
                .collect();
            let bindings = reuse
                .petite_scope_bindings(directives[0])
                .expect("native scope bindings");
            assert_eq!(
                bindings
                    .iter()
                    .map(|(name, offset)| (name.as_str(), *offset))
                    .collect::<Vec<_>>(),
                vec![
                    ("count", source.find("count: 0").unwrap() as u32),
                    ("msg", source.find("msg: 'x'").unwrap() as u32),
                ]
            );
            let effect = directives[1].exp.as_ref().expect("effect expression");
            assert_eq!(
                reuse
                    .petite_expression_names(effect)
                    .expect("native effect identifiers"),
                &[vize_carton::CompactString::new("count")]
            );
        }
        if source.contains("// comment") || source.contains("state =") {
            let TemplateChildNode::Element(element) = &root.children[0] else {
                panic!("root element");
            };
            let PropNode::Directive(directive) = &element.props[0] else {
                panic!("scope directive");
            };
            assert!(
                reuse.petite_scope_bindings(directive).is_none(),
                "non-object L2 expression must preserve the legacy extractor fallback"
            );
        }

        let mut legacy = Drawer::with_options(DrawerOptions::full());
        legacy.draw_template(&root);
        let legacy = vize_l0::cstr!("{:#?}", legacy.finish());

        let reuse = collect_petite_semantic_reuse(&root, &document).expect("petite reuse");
        let mut native =
            Drawer::with_options(DrawerOptions::full()).with_template_semantic_reuse(reuse);
        native.draw_template(&root);
        assert_eq!(
            vize_l0::cstr!("{:#?}", native.finish()),
            legacy,
            "source: {source}"
        );
    }
}

#[test]
fn petite_scope_property_shapes_match_legacy_bindings_and_offsets() {
    for expression in [
        "{ count }",
        "{ inc() { return 1 } }",
        "{ get total() { return 1 }, set total(value) {} }",
        "{ ...state, count: 0 }",
        "{ [key]: 1, count: 0 }",
        "{ 1: 'one', count: 0 }",
        "{ café: 1, count: 0 }",
        "{ count: 0, count: 1 }",
    ] {
        let source = vize_l0::cstr!(r#"<div v-scope="{expression}"></div>"#);
        let allocator = Allocator::new();
        let (root, errors) = vize_armature::parse(&allocator, &source);
        assert!(errors.is_empty(), "legacy fixture must parse: {errors:?}");
        let owner = NativeDocument::lex_in(
            &allocator,
            SourceRoot::new(&source).expect("bounded fixture"),
        );
        let document = PetiteDocument::lower_in(&allocator, &owner).expect("native fixture");
        let mut reuse = collect_petite_semantic_reuse(&root, &document).expect("petite directive");
        assert!(reuse.authenticate(&root));

        let TemplateChildNode::Element(element) = &root.children[0] else {
            panic!("root element");
        };
        let PropNode::Directive(directive) = &element.props[0] else {
            panic!("scope directive");
        };
        let ExpressionNode::Simple(simple) = directive.exp.as_ref().expect("scope expression")
        else {
            panic!("simple scope expression");
        };
        let expected: Vec<_> = extract_v_scope_bindings(simple.content)
            .into_iter()
            .map(|(name, offset)| (name, simple.loc.span.start + offset))
            .collect();
        let actual = reuse
            .petite_scope_bindings(directive)
            .expect("retained bindings must be selected");
        assert_eq!(actual, expected, "expression: {expression}");
    }
}

#[test]
fn petite_hints_honor_disabled_template_expression_collection() {
    let source = r#"<div v-scope="{ count: 0 }" v-effect="count++">{{ count }}</div>"#;
    let allocator = Allocator::new();
    let (root, errors) = vize_armature::parse(&allocator, source);
    assert!(errors.is_empty(), "legacy fixture must parse: {errors:?}");
    let owner = NativeDocument::lex_in(
        &allocator,
        SourceRoot::new(source).expect("bounded fixture"),
    );
    let document = PetiteDocument::lower_in(&allocator, &owner).expect("native fixture");

    let mut legacy = Drawer::with_options(DrawerOptions::for_lint());
    legacy.draw_template(&root);
    let legacy = legacy.finish();
    assert!(legacy.template_expressions.is_empty());

    let reuse = collect_petite_semantic_reuse(&root, &document).expect("petite reuse");
    let mut native =
        Drawer::with_options(DrawerOptions::for_lint()).with_template_semantic_reuse(reuse);
    native.draw_template(&root);
    let native = native.finish();
    assert!(native.template_expressions.is_empty());
    assert_eq!(vize_l0::cstr!("{native:#?}"), vize_l0::cstr!("{legacy:#?}"));
}
