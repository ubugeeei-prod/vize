//! Compatibility lowering retains the same source bridge as the native owner.

use vize_l0::{Allocator, SourceRoot, Span, id::NodeId};
use vize_l1::embed::Lang;
use vize_l1_to_l2::{lower_source_block, native::NativeComponent};
use vize_l2::analysis::template_complexity::{self, ComplexityFacts, Contribution, DecisionKind};
use vize_l2::artifact::Builder;
use vize_l2::expr::ExprRef;
use vize_l2::op::{BindingOp, Op};

fn span(source: &str, text: &str) -> Span {
    let start = source.find(text).unwrap() as u32;
    Span::new(start, start + text.len() as u32)
}

fn row(source: &str, text: &str, kind: DecisionKind, node: u32, nesting: u32) -> Contribution {
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
fn actual_native_and_compatibility_producers_agree_on_complete_encoded_expression_facts() {
    for newline in ["\n", "\r\n"] {
        for (quote, literal) in [('"', "'😀'"), ('\'', "\"😀\"")] {
            for and in ["&amp;&amp;", "&#38;&#x26;", "&amp;&amp"] {
                let a = Allocator::default();
                let binding = format!("a {and} (b ? {literal} : c)");
                let conditional = format!("x ? (y {and} z) : end");
                let body = format!("<p :title={quote}{binding}{quote}>{{{{ {conditional} }}}}</p>");
                let file = format!(
                    "<script>// 作者😀{newline}</script>{newline}<template>{body}</template>"
                );
                let start = file.find(&body).unwrap();
                let body = file.get(start..start + body.len()).unwrap();
                let block = SourceRoot::new(&file)
                    .unwrap()
                    .block(body, start as u32)
                    .unwrap();
                let (tree, errors) = vize_l1::markup::parse(&a, body);
                assert!(errors.is_empty());
                let compatibility = lower_source_block(&a, &tree, &errors, block);
                let facts = template_complexity::run(&compatibility.root, compatibility.op_count);
                let expected = ComplexityFacts {
                    cyclomatic: 5,
                    cognitive: 4,
                    unknown: 0,
                    max_nesting: 0,
                    contributions: vec![
                        row(&file, &binding, DecisionKind::Logical, 1, 0),
                        row(
                            &file,
                            &format!("b ? {literal} : c"),
                            DecisionKind::Conditional,
                            1,
                            0,
                        ),
                        row(&file, &conditional, DecisionKind::Conditional, 2, 0),
                        row(&file, &format!("y {and} z"), DecisionKind::Logical, 2, 1),
                    ],
                };
                assert_eq!(facts, expected);

                let mut builder = Builder::new(&a, &file).unwrap();
                let owner = NativeComponent::parse_in(&a, block).unwrap();
                let produced = owner.construct_in(&mut builder.region(), Lang::Ts).unwrap();
                assert!(produced.is_supported(), "{:?}", produced.holes);
                let native = builder.finish().unwrap();
                assert_eq!(
                    template_complexity::run(native.root(), native.node_count()),
                    expected
                );
                assert!(compatibility.diagnostics.is_empty());
                assert!(produced.diagnostics.is_empty());
            }
        }
    }
}

#[test]
fn once_decoding_unknown_references_and_partial_entity_refusals_keep_original_geometry() {
    let a = Allocator::default();
    let source = r#"<p :title="'&amp;amp;' && value">{{ '&fjlig;' && ready }}</p>"#;
    let (tree, errors) = vize_l1::markup::parse(&a, source);
    let block = SourceRoot::new(source).unwrap().whole_block();
    let lowered = lower_source_block(&a, &tree, &errors, block);
    let [Op::Element(element)] = lowered.root.ops.as_slice() else {
        panic!("element")
    };
    let [BindingOp::Bind(binding)] = element.bindings.as_slice() else {
        panic!("binding")
    };
    let Some(ExprRef::Js(value)) = binding.value else {
        panic!("retained binding")
    };
    assert_eq!(value.source, "'&amp;' && value");
    assert!(value.matches_authored_source(source));
    let [Op::Interpolation(interpolation)] = element.children.ops.as_slice() else {
        panic!("interpolation")
    };
    let ExprRef::Js(value) = interpolation.expression else {
        panic!("retained interpolation")
    };
    assert_eq!(value.source, "'fj' && ready");
    assert!(value.matches_authored_source(source));
    assert_eq!(value.authored_span(Span::new(2, 3)), None);
    assert_eq!(
        value.authored_span(Span::new(1, 3)),
        Some(span(source, "&fjlig;"))
    );
    let facts = template_complexity::run(&lowered.root, lowered.op_count);
    assert_eq!(
        facts.contributions,
        vec![
            row(source, "'&amp;amp;' && value", DecisionKind::Logical, 1, 0),
            row(source, "'&fjlig;' && ready", DecisionKind::Logical, 2, 0),
        ]
    );

    let source = r#"<p :title="'&unknown;' && value"/>"#;
    let (tree, errors) = vize_l1::markup::parse(&a, source);
    let lowered = lower_source_block(
        &a,
        &tree,
        &errors,
        SourceRoot::new(source).unwrap().whole_block(),
    );
    let [Op::Element(element)] = lowered.root.ops.as_slice() else {
        panic!("element")
    };
    let [BindingOp::Bind(binding)] = element.bindings.as_slice() else {
        panic!("binding")
    };
    let Some(ExprRef::Js(value)) = binding.value else {
        panic!("unknown reference is literal")
    };
    assert_eq!(value.source, "'&unknown;' && value");
    assert!(value.coordinates.is_none());
    assert!(value.matches_authored_source(source));
}
