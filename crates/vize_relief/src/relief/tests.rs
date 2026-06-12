//! Tests for AST node types.

use super::{
    ArrayExpression, AttributeNode, BlockStatement, CallExpression, Callee, CommentNode,
    CompoundExpressionNode, ConstantType, DirectiveNode, ElementNode, ElementType, IfBranchNode,
    IfNode, Namespace, NodeType, ObjectExpression, Position, RootNode, RuntimeHelper,
    SimpleExpressionNode, SourceLocation, TemplateChildNode, TextNode,
};
use vize_carton::Bump;

// ========================================================================
// Enum discriminant tests
// ========================================================================

#[test]
fn node_type_discriminants() {
    assert_eq!(NodeType::Root as u8, 0);
    assert_eq!(NodeType::Element as u8, 1);
    assert_eq!(NodeType::Text as u8, 2);
    assert_eq!(NodeType::Comment as u8, 3);
    assert_eq!(NodeType::SimpleExpression as u8, 4);
    assert_eq!(NodeType::Interpolation as u8, 5);
    assert_eq!(NodeType::Attribute as u8, 6);
    assert_eq!(NodeType::Directive as u8, 7);
    assert_eq!(NodeType::CompoundExpression as u8, 8);
    assert_eq!(NodeType::If as u8, 9);
    assert_eq!(NodeType::IfBranch as u8, 10);
    assert_eq!(NodeType::For as u8, 11);
    assert_eq!(NodeType::TextCall as u8, 12);
    assert_eq!(NodeType::VNodeCall as u8, 13);
    assert_eq!(NodeType::JsCallExpression as u8, 14);
    assert_eq!(NodeType::JsObjectExpression as u8, 15);
    assert_eq!(NodeType::JsProperty as u8, 16);
    assert_eq!(NodeType::JsArrayExpression as u8, 17);
    assert_eq!(NodeType::JsFunctionExpression as u8, 18);
    assert_eq!(NodeType::JsConditionalExpression as u8, 19);
    assert_eq!(NodeType::JsCacheExpression as u8, 20);
    assert_eq!(NodeType::JsBlockStatement as u8, 21);
    assert_eq!(NodeType::JsTemplateLiteral as u8, 22);
    assert_eq!(NodeType::JsIfStatement as u8, 23);
    assert_eq!(NodeType::JsAssignmentExpression as u8, 24);
    assert_eq!(NodeType::JsSequenceExpression as u8, 25);
    assert_eq!(NodeType::JsReturnStatement as u8, 26);
}

#[test]
fn element_type_discriminants() {
    assert_eq!(ElementType::Element as u8, 0);
    assert_eq!(ElementType::Component as u8, 1);
    assert_eq!(ElementType::Slot as u8, 2);
    assert_eq!(ElementType::Template as u8, 3);
}

#[test]
fn namespace_discriminants() {
    assert_eq!(Namespace::Html as u8, 0);
    assert_eq!(Namespace::Svg as u8, 1);
    assert_eq!(Namespace::MathMl as u8, 2);
}

#[test]
fn constant_type_discriminants() {
    assert_eq!(ConstantType::NotConstant as u8, 0);
    assert_eq!(ConstantType::CanSkipPatch as u8, 1);
    assert_eq!(ConstantType::CanCache as u8, 2);
    assert_eq!(ConstantType::CanStringify as u8, 3);
}

#[test]
fn constant_type_ordering() {
    assert!(ConstantType::NotConstant < ConstantType::CanSkipPatch);
    assert!(ConstantType::CanSkipPatch < ConstantType::CanCache);
    assert!(ConstantType::CanCache < ConstantType::CanStringify);
}

// ========================================================================
// Default impl tests
// ========================================================================

#[test]
fn element_type_default() {
    assert_eq!(ElementType::default(), ElementType::Element);
}

#[test]
fn namespace_default() {
    assert_eq!(Namespace::default(), Namespace::Html);
}

#[test]
fn constant_type_default() {
    assert_eq!(ConstantType::default(), ConstantType::NotConstant);
}

// ========================================================================
// RuntimeHelper tests
// ========================================================================

#[test]
fn runtime_helper_core_names() {
    assert_eq!(RuntimeHelper::Fragment.name(), "Fragment");
    assert_eq!(RuntimeHelper::CreateVNode.name(), "createVNode");
    assert_eq!(
        RuntimeHelper::CreateElementVNode.name(),
        "createElementVNode"
    );
    assert_eq!(
        RuntimeHelper::CreateElementBlock.name(),
        "createElementBlock"
    );
    assert_eq!(RuntimeHelper::OpenBlock.name(), "openBlock");
    assert_eq!(RuntimeHelper::ToDisplayString.name(), "toDisplayString");
    assert_eq!(RuntimeHelper::ResolveComponent.name(), "resolveComponent");
    assert_eq!(RuntimeHelper::WithDirectives.name(), "withDirectives");
    assert_eq!(RuntimeHelper::RenderList.name(), "renderList");
    assert_eq!(RuntimeHelper::RenderSlot.name(), "renderSlot");
    assert_eq!(RuntimeHelper::VShow.name(), "vShow");
    assert_eq!(RuntimeHelper::WithCtx.name(), "withCtx");
}

#[test]
fn runtime_helper_ssr_names() {
    assert_eq!(RuntimeHelper::SsrInterpolate.name(), "ssrInterpolate");
    assert_eq!(
        RuntimeHelper::SsrRenderComponent.name(),
        "ssrRenderComponent"
    );
    assert_eq!(RuntimeHelper::SsrRenderList.name(), "ssrRenderList");
    assert_eq!(RuntimeHelper::SsrRenderAttrs.name(), "ssrRenderAttrs");
    assert_eq!(RuntimeHelper::SsrRenderAttr.name(), "ssrRenderAttr");
    assert_eq!(RuntimeHelper::SsrRenderClass.name(), "ssrRenderClass");
    assert_eq!(RuntimeHelper::SsrRenderStyle.name(), "ssrRenderStyle");
    assert_eq!(RuntimeHelper::SsrRenderSlot.name(), "ssrRenderSlot");
}

#[test]
fn runtime_helper_is_ssr() {
    // All 19 SSR helpers should return true
    let ssr_helpers = [
        RuntimeHelper::SsrInterpolate,
        RuntimeHelper::SsrRenderVNode,
        RuntimeHelper::SsrRenderComponent,
        RuntimeHelper::SsrRenderSlot,
        RuntimeHelper::SsrRenderSlotInner,
        RuntimeHelper::SsrRenderAttrs,
        RuntimeHelper::SsrRenderAttr,
        RuntimeHelper::SsrRenderDynamicAttr,
        RuntimeHelper::SsrIncludeBooleanAttr,
        RuntimeHelper::SsrRenderClass,
        RuntimeHelper::SsrRenderStyle,
        RuntimeHelper::SsrRenderDynamicModel,
        RuntimeHelper::SsrGetDynamicModelProps,
        RuntimeHelper::SsrRenderList,
        RuntimeHelper::SsrLooseEqual,
        RuntimeHelper::SsrLooseContain,
        RuntimeHelper::SsrGetDirectiveProps,
        RuntimeHelper::SsrRenderTeleport,
        RuntimeHelper::SsrRenderSuspense,
    ];
    for helper in &ssr_helpers {
        assert!(helper.is_ssr(), "{:?} should be SSR", helper);
    }
}

#[test]
fn runtime_helper_core_not_ssr() {
    let core_helpers = [
        RuntimeHelper::Fragment,
        RuntimeHelper::CreateVNode,
        RuntimeHelper::CreateElementVNode,
        RuntimeHelper::OpenBlock,
        RuntimeHelper::CreateBlock,
        RuntimeHelper::ToDisplayString,
        RuntimeHelper::ResolveComponent,
        RuntimeHelper::WithDirectives,
        RuntimeHelper::VShow,
        RuntimeHelper::WithCtx,
    ];
    for helper in &core_helpers {
        assert!(!helper.is_ssr(), "{:?} should not be SSR", helper);
    }
}

// ========================================================================
// Node constructor tests
// ========================================================================

#[test]
fn root_node_new() {
    let allocator = Bump::new();
    let root = RootNode::new(&allocator, "test");
    assert_eq!(root.source.as_str(), "test");
    assert!(root.children.is_empty());
    assert!(root.helpers.is_empty());
    assert!(root.components.is_empty());
    assert_eq!(root.temps, 0);
    assert!(!root.transformed);
    assert!(root.codegen_node.is_none());
    assert_eq!(root.node_type(), NodeType::Root);
}

#[test]
fn element_node_new() {
    let allocator = Bump::new();
    let el = ElementNode::new(&allocator, "div", SourceLocation::STUB);
    assert_eq!(el.tag.as_str(), "div");
    assert_eq!(el.ns, Namespace::Html);
    assert_eq!(el.tag_type, ElementType::Element);
    assert!(el.props.is_empty());
    assert!(el.children.is_empty());
    assert!(!el.is_self_closing);
    assert!(el.codegen_node.is_none());
    assert_eq!(el.node_type(), NodeType::Element);
}

#[test]
fn text_node_new() {
    let node = TextNode::new("hello", SourceLocation::STUB);
    assert_eq!(node.content.as_str(), "hello");
    assert_eq!(node.node_type(), NodeType::Text);
}

#[test]
fn comment_node_new() {
    let node = CommentNode::new("a comment", SourceLocation::STUB);
    assert_eq!(node.content.as_str(), "a comment");
    assert_eq!(node.node_type(), NodeType::Comment);
}

#[test]
fn directive_node_new() {
    let allocator = Bump::new();
    let dir = DirectiveNode::new(&allocator, "if", SourceLocation::STUB);
    assert_eq!(dir.name.as_str(), "if");
    assert!(dir.raw_name.is_none());
    assert!(dir.exp.is_none());
    assert!(dir.arg.is_none());
    assert!(dir.modifiers.is_empty());
    assert!(dir.for_parse_result.is_none());
    assert_eq!(dir.node_type(), NodeType::Directive);
}

#[test]
fn attribute_node_new() {
    let attr = AttributeNode::new("id", SourceLocation::STUB);
    assert_eq!(attr.name.as_str(), "id");
    assert!(attr.value.is_none());
    assert_eq!(attr.node_type(), NodeType::Attribute);
}

#[test]
fn simple_expression_static() {
    let expr = SimpleExpressionNode::new("hello", true, SourceLocation::STUB);
    assert_eq!(expr.content.as_str(), "hello");
    assert!(expr.is_static);
    assert_eq!(expr.const_type, ConstantType::CanStringify);
    assert!(expr.js_ast.is_none());
    assert!(!expr.is_handler_key);
    assert!(!expr.is_ref_transformed);
    assert_eq!(expr.node_type(), NodeType::SimpleExpression);
}

#[test]
fn simple_expression_dynamic() {
    let expr = SimpleExpressionNode::new("foo", false, SourceLocation::STUB);
    assert!(!expr.is_static);
    assert_eq!(expr.const_type, ConstantType::NotConstant);
}

#[test]
fn compound_expression_new() {
    let allocator = Bump::new();
    let compound = CompoundExpressionNode::new(&allocator, SourceLocation::STUB);
    assert!(compound.children.is_empty());
    assert!(compound.identifiers.is_none());
    assert!(!compound.is_handler_key);
    assert_eq!(compound.node_type(), NodeType::CompoundExpression);
}

#[test]
fn if_node_new() {
    let allocator = Bump::new();
    let if_node = IfNode::new(&allocator, SourceLocation::STUB);
    assert!(if_node.branches.is_empty());
    assert!(if_node.codegen_node.is_none());
    assert_eq!(if_node.node_type(), NodeType::If);
}

#[test]
fn if_branch_node_new() {
    let allocator = Bump::new();
    let branch = IfBranchNode::new(&allocator, None, SourceLocation::STUB);
    assert!(branch.condition.is_none());
    assert!(branch.children.is_empty());
    assert!(branch.user_key.is_none());
    assert!(!branch.is_template_if);
    assert_eq!(branch.node_type(), NodeType::IfBranch);
}

#[test]
fn call_expression_new() {
    let allocator = Bump::new();
    let call = CallExpression::new(
        &allocator,
        Callee::Symbol(RuntimeHelper::CreateVNode),
        SourceLocation::STUB,
    );
    assert!(call.arguments.is_empty());
    assert_eq!(call.node_type(), NodeType::JsCallExpression);
}

#[test]
fn object_expression_new() {
    let allocator = Bump::new();
    let obj = ObjectExpression::new(&allocator, SourceLocation::STUB);
    assert!(obj.properties.is_empty());
    assert_eq!(obj.node_type(), NodeType::JsObjectExpression);
}

#[test]
fn array_expression_new() {
    let allocator = Bump::new();
    let arr = ArrayExpression::new(&allocator, SourceLocation::STUB);
    assert!(arr.elements.is_empty());
    assert_eq!(arr.node_type(), NodeType::JsArrayExpression);
}

#[test]
fn block_statement_new() {
    let allocator = Bump::new();
    let block = BlockStatement::new(&allocator, SourceLocation::STUB);
    assert!(block.body.is_empty());
    assert_eq!(block.node_type(), NodeType::JsBlockStatement);
}

// ========================================================================
// TemplateChildNode::loc() tests
// ========================================================================

#[test]
fn template_child_text_loc() {
    let allocator = Bump::new();
    let loc = SourceLocation::new(Position::new(5, 1, 6), Position::new(10, 1, 11), "hello");
    let text = TextNode::new("hello", loc.clone());
    let child = TemplateChildNode::Text(vize_carton::Box::new_in(text, &allocator));
    assert_eq!(*child.loc(), loc);
    assert_eq!(child.node_type(), NodeType::Text);
}

#[test]
fn template_child_hoisted_loc() {
    let child = TemplateChildNode::Hoisted(0);
    // Hoisted nodes use the STUB_LOCATION
    assert_eq!(child.loc().start.offset, 0);
    assert_eq!(child.loc().start.line, 1);
    assert_eq!(child.loc().start.column, 1);
    assert_eq!(child.node_type(), NodeType::SimpleExpression);
}

// ========================================================================
// SourceLocation / Position tests
// ========================================================================

#[test]
fn source_location_stub() {
    let stub = SourceLocation::STUB;
    assert_eq!(stub.start.offset, 0);
    assert_eq!(stub.start.line, 1);
    assert_eq!(stub.start.column, 1);
    assert_eq!(stub.end.offset, 0);
    assert_eq!(stub.source.as_str(), "");
}

#[test]
fn source_location_default_is_stub() {
    let default_loc = SourceLocation::default();
    assert_eq!(default_loc, SourceLocation::STUB);
}

#[test]
fn source_location_new() {
    let loc = SourceLocation::new(Position::new(0, 1, 1), Position::new(5, 1, 6), "hello");
    assert_eq!(loc.start.offset, 0);
    assert_eq!(loc.end.offset, 5);
    assert_eq!(loc.source.as_str(), "hello");
}

#[test]
fn position_new() {
    let pos = Position::new(42, 3, 10);
    assert_eq!(pos.offset, 42);
    assert_eq!(pos.line, 3);
    assert_eq!(pos.column, 10);
}

#[test]
fn position_default() {
    let pos = Position::default();
    assert_eq!(pos.offset, 0);
    assert_eq!(pos.line, 0);
    assert_eq!(pos.column, 0);
}
