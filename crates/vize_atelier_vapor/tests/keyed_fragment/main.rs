//! Key ownership preserves whole public compiler results and list carriers.

use serde_json::{Value, json};
use vize_atelier_core::parser::parse;
use vize_atelier_vapor::{
    VaporCompilerExperimentalOptions, VaporCompilerOptions,
    compile_vapor_with_experimental_options,
    ir::{BlockIRNode, IRSlotControl, InsertionAnchor, NegativeBranch, OperationNode},
    lower::transform_to_ir,
};
use vize_carton::Allocator;

mod branch;

fn keys(block: &BlockIRNode<'_>) -> usize {
    block
        .operation
        .iter()
        .map(|op| match op {
            OperationNode::Key(node) => 1 + keys(&node.render),
            OperationNode::For(node) => keys(&node.render),
            OperationNode::If(node) => {
                keys(&node.positive)
                    + node.negative.as_ref().map_or(0, |negative| match negative {
                        NegativeBranch::Block(block) => keys(block),
                        NegativeBranch::If(node) => keys(&node.positive),
                    })
            }
            OperationNode::CreateComponent(node) => node
                .slots
                .iter()
                .flat_map(|slot| {
                    std::iter::successors(Some(slot), |slot| match &slot.control {
                        Some(IRSlotControl::If {
                            negative: Some(next),
                            ..
                        }) => Some(next.as_ref()),
                        _ => None,
                    })
                })
                .map(|slot| keys(&slot.block))
                .sum(),
            _ => 0,
        })
        .sum()
}

fn complete(source: &str, retained: bool, mapped: bool) -> (Value, usize) {
    let allocator = Allocator::new();
    let result = compile_vapor_with_experimental_options(
        &allocator,
        source,
        VaporCompilerOptions {
            davinci_retained_lane: retained,
            ..Default::default()
        },
        VaporCompilerExperimentalOptions {
            source_map: mapped,
            ..Default::default()
        },
    );
    assert!(
        result.error_messages.is_empty(),
        "{:?}",
        result.error_messages
    );
    // Count actual lexical calls in this newly authored fixture, not strings.
    let calls = result.code.matches(" = _createKeyedFragment(").count();
    (
        json!({"code":result.code,"templates":result.templates,"map":result.map,"errors":result.error_messages}),
        calls,
    )
}

#[test]
fn keyed_owner_retains_original_expression_span_and_inner_component_scope() {
    let allocator = Allocator::new();
    let source = "<Counter :key=\"epoch\" />";
    let (root, errors) = parse(&allocator, source);
    assert!(errors.is_empty(), "{errors:?}");
    let ir = transform_to_ir(&allocator, &root, source);
    let [OperationNode::Key(node)] = ir.block.operation.as_slice() else {
        panic!("exact keyed owner")
    };
    assert_eq!(node.value.content, "epoch");
    assert_eq!(node.value.loc.span.slice(source), "epoch");
    assert_eq!(
        node.value.content.as_ptr(),
        node.value.loc.span.slice(source).as_ptr()
    );
    assert_eq!(ir.block.returns.as_slice(), &[node.id]);
    assert_eq!((node.parent, node.anchor), (None, None));
    let [OperationNode::CreateComponent(child)] = node.render.operation.as_slice() else {
        panic!("exact original component")
    };
    assert_eq!(child.tag, "Counter");
    assert!(child.props.is_empty());
    assert_eq!(node.render.returns.as_slice(), &[child.id]);
    assert_eq!((child.parent, child.anchor), (None, None));
    assert_eq!(keys(&ir.block), 1);
}

#[test]
fn literal_once_template_keeps_one_original_input_and_trailing_key_anchor() {
    let allocator = Allocator::new();
    let source = "<section><template v-once><input :key=\"held\" :value=\"value\" /></template><p>after</p><Counter :key=\"epoch\" /></section>";
    let (root, errors) = parse(&allocator, source);
    assert!(errors.is_empty(), "{errors:?}");
    let vize_atelier_core::TemplateChildNode::Element(section) = &root.children[0] else {
        panic!("original section")
    };
    let vize_atelier_core::TemplateChildNode::Element(template) = &section.children[0] else {
        panic!("original literal template")
    };
    assert_eq!(template.tag, "template");
    assert_eq!(template.tag_type, vize_atelier_core::ElementType::Element);
    let ir = transform_to_ir(&allocator, &root, source);
    assert_eq!(
        ir.templates.as_slice(),
        &["<section><template><input></template><p>after</p></section>"]
    );
    let [parent] = ir.block.returns.as_slice() else {
        panic!("one original native parent")
    };
    let node = ir
        .block
        .operation
        .iter()
        .find_map(|op| match op {
            OperationNode::Key(node) => Some(node),
            _ => None,
        })
        .expect("only outside-once key owns a fragment");
    assert_eq!(node.value.content, "epoch");
    assert_eq!(node.value.loc.span.slice(source), "epoch");
    assert_eq!(node.parent, Some(*parent));
    assert_eq!(node.anchor, Some(InsertionAnchor::Index(2)));
    assert_eq!(keys(&ir.block), 1);
}

#[test]
fn all_complete_keyed_outputs_maps_and_ordinary_boundaries_agree() {
    for (source, count) in [
        ("<i :key=\"epoch\">{{ value }}</i>", 1),
        ("<Counter :key=\"epoch\" />", 1),
        (
            "<section><p>before</p><input :key=\"epoch\" :value=\"value\" /><p>after</p></section>",
            1,
        ),
        ("<section><Counter :key=\"epoch\" /></section>", 1),
        (
            "<section><i :key=\"a\"><input :key=\"b\" :value=\"value\" /></i></section>",
            2,
        ),
        (
            "<i v-for=\"item in items\" :key=\"item.id\">{{ item.label }}</i>",
            0,
        ),
        (
            "<i v-for=\"item in items\" :key=\"item.id\"><Counter :key=\"epoch\" /></i>",
            1,
        ),
        ("<i v-once :key=\"epoch\">{{ value }}</i>", 0),
        (
            "<section v-once><input :key=\"epoch\" :value=\"value\" /></section>",
            0,
        ),
        ("<i v-memo=\"[]\" :key=\"epoch\">{{ value }}</i>", 0),
        (
            "<section><div v-once><input :key=\"epoch\" :value=\"value\" /></div></section>",
            0,
        ),
        (
            "<section><div v-memo=\"[]\"><input :key=\"epoch\" :value=\"value\" /></div></section>",
            0,
        ),
        (
            "<section><template v-once><input :key=\"epoch\" :value=\"value\" /></template></section>",
            0,
        ),
        (
            "<section><div v-once><i v-if=\"ok\"><Counter :key=\"epoch\" /></i></div></section>",
            0,
        ),
        ("<i key=\"literal\">text</i>", 0),
        ("<i :key=\"true\">text</i>", 0),
        ("<i>ordinary</i>", 0),
    ] {
        for mapped in [false, true] {
            let (selected, actual) = complete(source, false, mapped);
            let (retained, retained_count) = complete(source, true, mapped);
            assert_eq!(selected, retained, "{source}");
            assert_eq!((actual, retained_count), (count, count), "{source}");
        }
        let (mut mapped, _) = complete(source, false, true);
        let (plain, _) = complete(source, false, false);
        let graph: Value = serde_json::from_str(
            mapped
                .get("map")
                .and_then(Value::as_str)
                .expect("complete template map string"),
        )
        .expect("template Source Map v3 JSON");
        assert_eq!(graph.get("version"), Some(&json!(3)));
        *mapped.get_mut("map").expect("complete map field") = Value::Null;
        assert_eq!(
            mapped, plain,
            "mapping must preserve every complete public field"
        );
    }
}

#[test]
fn only_the_actual_returned_keyed_root_templates_get_fallthrough_flags() {
    let cases: [(&str, &[&str]); 5] = [
        (
            "<input :key=\"epoch\" :title=\"inner\" />",
            &["const t0 = _template(\"<input>\", true)"],
        ),
        (
            "<section><input :key=\"epoch\" :title=\"inner\" /></section>",
            &[
                "const t0 = _template(\"<input>\")",
                "const t1 = _template(\"<section></section>\", true)",
            ],
        ),
        (
            "<input v-if=\"ok\" :key=\"epoch\" :title=\"inner\" />",
            &["const t0 = _template(\"<input>\", true)"],
        ),
        (
            "<section><input v-if=\"ok\" :key=\"epoch\" :title=\"inner\" /></section>",
            &[
                "const t0 = _template(\"<input>\")",
                "const t1 = _template(\"<section></section>\", true)",
            ],
        ),
        (
            "<input :key=\"epoch\" :title=\"inner\" /><p>after</p>",
            &[
                "const t0 = _template(\"<input>\")",
                "const t1 = _template(\"<p>after</p>\")",
            ],
        ),
    ];
    for (source, expected) in cases {
        for retained in [false, true] {
            let allocator = Allocator::new();
            let result = compile_vapor_with_experimental_options(
                &allocator,
                source,
                VaporCompilerOptions {
                    davinci_retained_lane: retained,
                    ..Default::default()
                },
                Default::default(),
            );
            assert!(
                result.error_messages.is_empty(),
                "{:?}",
                result.error_messages
            );
            let mut declarations = result
                .code
                .lines()
                .filter(|line| line.starts_with("const t"));
            for declaration in expected {
                assert_eq!(declarations.next(), Some(*declaration), "{source}");
            }
            assert_eq!(declarations.next(), None, "no extra template declaration");
        }
    }
}
