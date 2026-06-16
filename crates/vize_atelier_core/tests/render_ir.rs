//! Construction smoke tests for the relocated render/codegen IR (#1760).
//!
//! These types moved out of `vize_relief` into `vize_atelier_core`; the tests
//! moved with them and now exercise the public re-exports.

use bumpalo::Bump;
use vize_atelier_core::{
    ArrayExpression, BlockStatement, CallExpression, Callee, NodeType, ObjectExpression,
    RuntimeHelper, SourceLocation,
};

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
