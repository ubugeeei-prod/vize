//! Real core branches keep their consumed authored key and expression owner.

use vize_atelier_core::{
    ExpressionNode, PropNode, TemplateChildNode, lane::transform, options::TransformOptions,
    parser::parse,
};
use vize_atelier_vapor::{
    ir::{NegativeBranch, OperationNode},
    lower::transform_to_ir,
};
use vize_carton::Allocator;

#[test]
fn actual_core_branch_keys_keep_original_spans_and_completed_child_blocks() {
    let allocator = Allocator::new();
    let source = "<input v-if=\"a\" :key=\"first\" /><Counter v-else-if=\"b\" :key=\"second\" /><input v-else :key=\"third\" />";
    let (mut root, errors) = parse(&allocator, source);
    assert!(errors.is_empty(), "{errors:?}");
    let errors = transform(
        &allocator,
        &mut root,
        TransformOptions {
            vapor: true,
            ..Default::default()
        },
        None,
    );
    assert!(errors.is_empty(), "{errors:?}");
    let [TemplateChildNode::If(original)] = root.children.as_slice() else {
        panic!("actual core conditional")
    };
    let originals: std::vec::Vec<_> = original.branches.iter().map(|branch| {
        assert!(!branch.is_template_if);
        let Some(PropNode::Directive(dir)) = branch.user_key.as_ref() else { panic!("actual consumed key") };
        let Some(ExpressionNode::Simple(value)) = dir.exp.as_ref() else { panic!("actual original expression") };
        let [TemplateChildNode::Element(el)] = branch.children.as_slice() else { panic!("actual single owner") };
        assert_eq!(branch.loc, el.loc);
        assert!(el.props.iter().all(|prop| !matches!(prop, PropNode::Directive(dir)
            if matches!(dir.arg.as_ref(), Some(ExpressionNode::Simple(arg)) if arg.content == "key"))));
        (value, el.tag)
    }).collect();
    let ir = transform_to_ir(&allocator, &root, source);
    let [OperationNode::If(node)] = ir.block.operation.as_slice() else {
        panic!("actual lowered conditional")
    };
    let Some(NegativeBranch::If(second)) = node.negative.as_ref() else {
        panic!("actual else-if")
    };
    let Some(NegativeBranch::Block(third)) = second.negative.as_ref() else {
        panic!("actual else")
    };
    for ((value, tag), block) in originals
        .iter()
        .zip([&node.positive, &second.positive, third])
    {
        let [OperationNode::Key(key)] = block.operation.as_slice() else {
            panic!("one original keyed branch")
        };
        assert_eq!(key.value.content, value.content);
        assert_eq!(key.value.loc, value.loc);
        assert_eq!(key.value.content.as_ptr(), value.content.as_ptr());
        assert_eq!(key.value.loc.span.slice(source), value.content);
        assert_eq!(block.returns.as_slice(), &[key.id]);
        assert_eq!((key.parent, key.anchor), (None, None));
        assert_eq!(key.render.returns.len(), 1);
        if *tag == "Counter" {
            let [OperationNode::CreateComponent(child)] = key.render.operation.as_slice() else {
                panic!("same original component")
            };
            assert_eq!(child.tag, *tag);
            assert_eq!(key.render.returns.as_slice(), &[child.id]);
        } else {
            let template = ir
                .element_template_map
                .get(&key.render.returns[0])
                .expect("same actual returned native template");
            assert_eq!(ir.templates[*template], "<input>");
        }
    }
}

#[test]
fn whole_branch_key_modules_and_maps_agree_in_both_lanes() {
    for (source, count) in [
        ("<input v-if=\"ok\" :key=\"epoch\" />", 1),
        (
            "<section><input v-if=\"ok\" :key=\"epoch\" :title=\"inner\" /></section>",
            1,
        ),
        (
            "<input v-if=\"a\" :key=\"first\" /><Counter v-else-if=\"b\" :key=\"second\" /><input v-else :key=\"third\" />",
            3,
        ),
        ("<input v-if=\"ok\" :key=\"epoch + 1\" />", 1),
        ("<input v-if=\"ok\" v-once :key=\"epoch\" />", 0),
        ("<input v-if=\"ok\" :key=\"true\" />", 0),
        (
            "<template v-if=\"ok\" :key=\"epoch\"><input /></template>",
            0,
        ),
        (
            "<input v-if=\"ok\" v-for=\"item in items\" :key=\"item.id\" />",
            0,
        ),
    ] {
        for mapped in [false, true] {
            let (selected, selected_count) = super::complete(source, false, mapped);
            let (retained, retained_count) = super::complete(source, true, mapped);
            assert_eq!(selected, retained, "{source}");
            assert_eq!((selected_count, retained_count), (count, count), "{source}");
        }
    }
}
