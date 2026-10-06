//! Same-original property facts retain whole IR, modules, maps and scope.

use super::{branch::transform_branch, classify};
use crate::{
    generate::{
        VaporGenerateExperimentalOptions, VaporGenerateOptions, generate_vapor_with_spans,
        spans::VaporSourceSpans,
    },
    ir::{BlockIRNode, OperationNode, RootIRNode},
    lower::{
        context::{TemplateSpans, TransformContext},
        element::{transform_classified_element, transform_slot},
        transform_children,
    },
};
use vize_atelier_core::{
    ExpressionNode, PropNode, RootNode, TemplateChildNode, lane::transform,
    options::TransformOptions, parser::parse,
};
use vize_carton::{Allocator, String, Vec};

#[derive(Debug, PartialEq, Eq)]
struct Output {
    block: std::string::String,
    diagnostics: std::vec::Vec<String>,
    modules: std::vec::Vec<(String, std::vec::Vec<String>, Option<String>)>,
}

fn output<'a>(
    ctx: TransformContext<'a>,
    block: BlockIRNode<'a>,
    root: &RootNode<'a>,
    source: &'a str,
) -> Output {
    let block_text = format!("{block:?}");
    let spans = VaporSourceSpans::collect(root, ctx.template_spans.unwrap());
    let ir = RootIRNode {
        node: RootNode::new(ctx.allocator, ""),
        source,
        template: Default::default(),
        template_index_map: Default::default(),
        root_template_indexes: Vec::new_in(&ctx.allocator),
        component: Vec::new_in(&ctx.allocator),
        directive: Vec::new_in(&ctx.allocator),
        block,
        has_template_ref: false,
        has_deferred_v_show: false,
        templates: ctx.templates,
        element_template_map: ctx.element_template_map,
        standalone_text_elements: ctx.standalone_text_elements,
    };
    let mut modules = std::vec::Vec::new();
    for jsx_closure in [false, true] {
        for source_map in [false, true] {
            let result = generate_vapor_with_spans(
                &ir,
                None,
                VaporGenerateOptions { jsx_closure },
                VaporGenerateExperimentalOptions {
                    source_map,
                    source_map_filename: Some("Original.vue"),
                    ..Default::default()
                },
                source_map.then_some(&spans),
            );
            assert_eq!(result.map.is_some(), source_map);
            modules.push((result.code, result.templates, result.map));
        }
    }
    Output {
        block: block_text,
        diagnostics: ctx.diagnostics,
        modules,
    }
}

#[test]
fn slot_property_walk_preserves_original_scope_fallback_ir_and_complete_outputs() {
    let cases = [
        ("<slot></slot>", false),
        (
            "<slot name=\"named\" plain=\"a&amp;b\" :value=\"value\" :[field]=\"dynamic\" v-bind=\"spread\"><input :key=\"epoch\" :value=\"value\" /></slot>",
            false,
        ),
        (
            "<slot :name=\"named\" :key=\"ignored\"><input :key=\"epoch\" :value=\"value\" /></slot>",
            false,
        ),
        (
            "<slot v-once><input :key=\"epoch\" :value=\"value\" /></slot>",
            false,
        ),
        (
            "<slot v-memo=\" [] \"><input :key=\"epoch\" :value=\"value\" /></slot>",
            false,
        ),
        (
            "<slot v-memo=\"deps\"><input :key=\"epoch\" :value=\"value\" /></slot>",
            false,
        ),
        (
            "<slot v-memo><input :key=\"epoch\" :value=\"value\" /></slot>",
            false,
        ),
        (
            "<slot v-memo=\"deps\" v-memo=\"[]\"><input :key=\"epoch\" :value=\"value\" /></slot>",
            false,
        ),
        (
            "<slot v-memo=\"deps\" v-once><input :key=\"epoch\" :value=\"value\" /></slot>",
            false,
        ),
        (
            "<slot><input :key=\"epoch\" :value=\"value\" /></slot>",
            true,
        ),
    ];
    for (source, inherited) in cases {
        let allocator = Allocator::new();
        let (root, errors) = parse(&allocator, source);
        assert_eq!(errors.len(), 0, "{source}: {errors:?}");
        let [TemplateChildNode::Element(el)] = root.children.as_slice() else {
            panic!("the entire original source must own one slot");
        };
        assert_eq!(el.tag_type, vize_atelier_core::ElementType::Slot);
        let mut actual = TransformContext::new(&allocator, source);
        actual.template_spans = Some(TemplateSpans::default());
        let mut original = TransformContext::new(&allocator, source);
        original.template_spans = Some(TemplateSpans::default());
        if inherited {
            actual.enter_non_reactive_scope();
            original.enter_non_reactive_scope();
        }
        let mut actual_block = BlockIRNode::new(&allocator);
        let mut original_block = BlockIRNode::new(&allocator);
        transform_slot(&mut actual, el, &mut actual_block);
        let facts = classify(el, inherited, false);
        transform_classified_element(&mut original, el, &mut original_block, facts);
        assert_eq!(actual.is_key_non_reactive(), inherited, "{source}");
        assert_eq!(original.is_key_non_reactive(), inherited, "{source}");
        assert_eq!(
            output(actual, actual_block, &root, source),
            output(original, original_block, &root, source),
            "{source}",
        );
    }
}

#[test]
fn transferred_branch_keys_keep_ineligible_original_targets_unwrapped() {
    let sources = [
        "<component v-if=\"shown\" :is=\"selected\" :key=\"epoch\"></component>",
        "<KeepAlive v-if=\"shown\" :key=\"epoch\"><Counter></Counter></KeepAlive>",
        "<slot v-if=\"shown\" :key=\"epoch\"><input :value=\"value\" /></slot>",
        "<Transition v-if=\"shown\" :key=\"epoch\"><input /></Transition>",
        "<Teleport v-if=\"shown\" :key=\"epoch\" to=\"#target\"><input /></Teleport>",
        "<Suspense v-if=\"shown\" :key=\"epoch\"><Counter></Counter></Suspense>",
        "<TransitionGroup v-if=\"shown\" :key=\"epoch\"><input /></TransitionGroup>",
    ];
    for source in sources {
        let allocator = Allocator::new();
        let (mut root, errors) = parse(&allocator, source);
        assert_eq!(errors.len(), 0, "{source}: {errors:?}");
        let [TemplateChildNode::Element(el)] = root.children.as_slice() else {
            panic!("the complete original owns the conditional element");
        };
        let authored_key = el
            .props
            .iter()
            .find_map(|prop| match prop {
                PropNode::Directive(dir)
                    if dir.name == "bind"
                        && matches!(dir.arg.as_ref(), Some(ExpressionNode::Simple(arg))
                    if arg.is_static && arg.content == "key") =>
                {
                    dir.exp.as_ref()
                }
                _ => None,
            })
            .unwrap();
        let authored_span = authored_key.loc().span;
        assert_eq!(
            &source[authored_span.start as usize..authored_span.end as usize],
            "epoch"
        );
        let errors = transform(
            &allocator,
            &mut root,
            TransformOptions {
                vapor: true,
                ..Default::default()
            },
            None,
        );
        assert_eq!(errors.len(), 0, "{source}: {errors:?}");
        let [TemplateChildNode::If(if_node)] = root.children.as_slice() else {
            panic!("the real core transform must transfer the conditional owner");
        };
        let [branch] = if_node.branches.as_slice() else {
            panic!("the authored conditional has exactly one branch");
        };
        let Some(PropNode::Directive(dir)) = branch.user_key.as_ref() else {
            panic!("the real core branch must retain its original bind key");
        };
        let Some(ExpressionNode::Simple(key)) = dir.exp.as_ref() else {
            panic!("the transferred original key is a simple expression");
        };
        assert_eq!(key.content, "epoch");
        assert_eq!(key.loc.span, authored_span);
        let mut actual = TransformContext::new(&allocator, source);
        actual.template_spans = Some(TemplateSpans::default());
        let mut original = TransformContext::new(&allocator, source);
        original.template_spans = Some(TemplateSpans::default());
        let actual_block = transform_branch(&mut actual, branch);
        assert_eq!(
            actual_block
                .operation
                .iter()
                .filter(|op| matches!(op, OperationNode::Key(_)))
                .count(),
            0
        );
        let original_block = transform_children(&mut original, &branch.children);
        assert_eq!(
            output(actual, actual_block, &root, source),
            output(original, original_block, &root, source),
            "{source}",
        );
    }
}
