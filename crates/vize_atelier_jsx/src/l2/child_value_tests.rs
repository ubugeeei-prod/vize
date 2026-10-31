use super::{L2Refusal, try_lower_root};
use crate::{JsxCompatMode, JsxLang, JsxOutputMode, lower::BabelLoweringOptions};
use vize_l0::Allocator;
use vize_l2::op::Op;
use vize_relief::{CompoundExpressionChild, TemplateChildNode};

#[test]
fn actual_client_jsx_mint_owns_the_child_role_without_changing_analysis_or_vapor() {
    let allocator = Allocator::new();
    let source = "const App=()=> <main>{state.value}</main>;";
    for (mode, lane, expected) in [
        (JsxOutputMode::Vdom, true, true),
        (JsxOutputMode::Vdom, false, false),
        (JsxOutputMode::Vapor, true, false),
    ] {
        let (lowered, _) = crate::lower_source_with_compat(
            &allocator,
            allocator.as_oxc(),
            source,
            JsxLang::Jsx,
            JsxCompatMode::Native,
            mode,
            BabelLoweringOptions {
                vdom_lane: lane,
                ..Default::default()
            },
        );
        assert!(!lowered.has_errors());
        let root = &lowered.roots[0];
        assert_eq!(root.vdom_value_children, expected);
        let l2 = root.l2.as_ref().expect("lossless native projection");
        assert_eq!(l2.features.has_jsx_child_values(), expected);
        let Op::Element(element) = &l2.root.ops[0] else {
            panic!("intrinsic element")
        };
        assert_eq!(
            matches!(&element.children.ops[0], Op::JsxChildValue(_)),
            expected
        );
        if let Op::JsxChildValue(child) = &element.children.ops[0] {
            assert_eq!(child.expression.source(), "state.value");
            assert_eq!(
                child.expression.span().start,
                source.find("state.value").unwrap() as u32
            );
        }
    }
}

#[test]
fn unmarked_or_malformed_compounds_keep_the_original_transformed_child_refusal() {
    let allocator = Allocator::new();
    let source = "const App=()=> <main>{state.value}</main>;";
    let (mut lowered, _) = crate::lower_source_with_compat(
        &allocator,
        allocator.as_oxc(),
        source,
        JsxLang::Jsx,
        JsxCompatMode::Native,
        JsxOutputMode::Vdom,
        BabelLoweringOptions {
            vdom_lane: true,
            ..Default::default()
        },
    );
    let root = &mut lowered.roots[0].root;
    let TemplateChildNode::Element(element) = &mut root.children[0] else {
        panic!("original element")
    };
    let TemplateChildNode::CompoundExpression(compound) = &mut element.children[0] else {
        panic!("real JSX producer carrier")
    };
    compound.is_jsx_child_value = false;
    assert!(matches!(
        try_lower_root(&allocator, source, root),
        Err(L2Refusal::TransformedChild)
    ));
    let TemplateChildNode::Element(element) = &mut root.children[0] else {
        unreachable!()
    };
    let TemplateChildNode::CompoundExpression(compound) = &mut element.children[0] else {
        unreachable!()
    };
    compound.is_jsx_child_value = true;
    compound
        .children
        .push(CompoundExpressionChild::String(" + 1"));
    assert!(matches!(
        try_lower_root(&allocator, source, root),
        Err(L2Refusal::TransformedChild)
    ));
}

#[test]
fn babel_compat_retains_unmarked_raw_compounds_without_native_role_or_scope_fact() {
    let allocator = Allocator::new();
    let source = "const App=()=> <main>{state.value}</main>;";
    let (lowered, _) = crate::lower_source_with_compat(
        &allocator,
        allocator.as_oxc(),
        source,
        JsxLang::Jsx,
        JsxCompatMode::Babel,
        JsxOutputMode::Vdom,
        BabelLoweringOptions {
            vdom_lane: true,
            ..Default::default()
        },
    );
    assert!(!lowered.has_errors());
    let root = &lowered.roots[0];
    assert!(!root.vdom_value_children);
    assert!(matches!(root.l2, Err(L2Refusal::TransformedChild)));
    let TemplateChildNode::Element(element) = &root.root.children[0] else {
        panic!("original element")
    };
    let TemplateChildNode::CompoundExpression(compound) = &element.children[0] else {
        panic!("original raw compound")
    };
    assert!(!compound.is_jsx_child_value);
    assert!(!compound.jsx_child_needs_grouping);
    let [CompoundExpressionChild::Simple(expression)] = compound.children.as_slice() else {
        panic!("one original raw expression")
    };
    assert_eq!(expression.content, "state.value");
}
