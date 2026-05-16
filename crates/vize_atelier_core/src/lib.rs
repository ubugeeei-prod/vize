//! # vize_atelier_core
//!
//! Atelier Core - The core workshop for Vize.
//! Vue template transforms and code generation.
//!
//! ## Name Origin
//!
//! **Atelier** (/ˌætəlˈjeɪ/) is an artist's workshop or studio where creative work
//! is produced. The "core" atelier is the foundational workshop where the essential
//! Vue template processing happens - transforming and code generation.
//! `vize_atelier_core` provides the foundational infrastructure
//! that all other Vize compilers build upon.

pub mod codegen;
pub mod runtime_helpers;
#[macro_use]
pub mod test_macros;
pub mod transform;
pub mod transforms;

// Re-export from vize_relief (AST, errors, options)
pub use vize_relief::ast::{
    ArrayElement, ArrayExpression, AssignmentExpression, AttributeNode, BlockStatement,
    BlockStatementBody, CacheExpression, CallArgument, CallExpression, Callee, CodegenNode,
    CommentNode, CompoundExpressionChild, CompoundExpressionNode, ConditionalExpression,
    ConstantType, DirectiveArgumentNode, DirectiveArguments, DirectiveNode, DynamicProps,
    ElementCodegenNode, ElementNode, ElementType, ExpressionNode, ForNode, ForParseResult,
    FunctionBody, FunctionExpression, FunctionParam, FunctionParams, FunctionReturns, IfBranchNode,
    IfCodegenNode, IfNode, IfStatement, IfStatementAlternate, ImportItem, InterpolationNode,
    JsChildNode, JsExpression, Namespace, NodeType, ObjectExpression, Position, PropNode, Property,
    PropsExpression, ReturnStatement, ReturnValue, RootNode, RuntimeHelper, SequenceExpression,
    SimpleExpressionNode, SlotsExpression, SourceLocation, TemplateChildNode, TemplateLiteral,
    TemplateLiteralElement, TemplateTextChildNode, TextCallCodegenNode, TextCallContent,
    TextCallNode, TextNode, VNodeCall, VNodeChildren, VNodeTag,
};
pub use vize_relief::errors::{CompilerError, CompilerResult, ErrorCode};
pub use vize_relief::options::{
    BindingMetadata, BindingType, CodegenMode, CodegenOptions, CompilerOptions, ParseMode,
    ParserOptions, TextMode, TransformOptions, WhitespaceStrategy,
};
pub use vize_relief::{ast, errors, options};

// Re-export from vize_armature (parser, tokenizer)
pub use vize_armature as parser;
pub use vize_armature::tokenizer;
pub use vize_armature::{parse, parse_with_options, Parser};

pub use codegen::{generate, CodegenContext, CodegenResult};
pub use runtime_helpers::{get_vnode_block_helper, get_vnode_helper, RuntimeHelpers};
pub use transform::{
    transform, DirectiveTransform, DirectiveTransformResult, ExitFn, NodeTransform, ParentNode,
    StructuralDirectiveTransform, TransformContext,
};
pub use transforms::{
    build_element_codegen, build_props, build_text_call, camelize, collect_slots,
    condense_whitespace, count_dynamic_children, create_on_name, generate_memo_check,
    generate_v_memo_wrapper, generate_v_once_wrapper, get_bind_name, get_bind_value,
    get_event_name, get_for_expression, get_handler_expression, get_if_condition, get_memo_deps,
    get_memo_exp, get_model_event_prop, get_slot_name, get_slot_props_string, get_static_type,
    get_vmodel_helper, has_attr_modifier, has_camel_modifier, has_dynamic_slots, has_prop_modifier,
    has_v_else, has_v_else_if, has_v_for, has_v_if, has_v_memo, has_v_once, has_v_slot,
    hoist_static, is_condensible_whitespace, is_dynamic_binding, is_dynamic_event, is_dynamic_slot,
    is_simple_identifier, is_static_node, is_whitespace_only, needs_guard, parse_event_modifiers,
    parse_for_expression, parse_model_modifiers, prefix_identifiers_in_expression,
    process_expression, process_inline_handler, process_v_bind, process_v_for, process_v_if,
    process_v_memo, process_v_on, remove_for_directive, remove_if_directive, remove_v_memo,
    remove_v_once, resolve_element_type, should_use_block, strip_typescript_from_expression,
    supports_v_model, transform_slot_outlet, transform_text_children, transform_v_model,
    transform_v_once, ChildrenType, EventModifiers, MemoInfo, PropItem, SlotInfo, SlotOutletInfo,
    StaticType, TextCallExpression, TextPart, TransformPropsExpression, TransformVNodeCall,
    VModelModifiers,
};

/// Re-export allocator types for convenience
pub use vize_carton::{Allocator, Box as AllocBox, CloneIn, Vec as AllocVec};
