//! Code generation AST node types.
//!
//! Contains VNodeCall, JavaScript expression nodes, SSR codegen nodes,
//! and all types used during code generation from the template AST.

use vize_carton::{Box, Bump, PatchFlags, String, Vec};

use super::{
    core::{NodeType, SourceLocation},
    expressions::{CompoundExpressionNode, ExpressionNode, SimpleExpressionNode},
    nodes::TemplateChildNode,
    RuntimeHelper,
};

/// VNode call expression
#[derive(Debug)]
pub struct VNodeCall<'a> {
    pub tag: VNodeTag<'a>,
    pub props: Option<PropsExpression<'a>>,
    pub children: Option<VNodeChildren<'a>>,
    pub patch_flag: Option<PatchFlags>,
    pub dynamic_props: Option<DynamicProps<'a>>,
    pub directives: Option<DirectiveArguments<'a>>,
    pub is_block: bool,
    pub disable_tracking: bool,
    pub is_component: bool,
    pub loc: SourceLocation,
}

impl<'a> VNodeCall<'a> {
    pub fn node_type(&self) -> NodeType {
        NodeType::VNodeCall
    }
}

/// VNode tag type
#[derive(Debug)]
pub enum VNodeTag<'a> {
    String(String),
    Symbol(RuntimeHelper),
    Call(Box<'a, CallExpression<'a>>),
}

/// VNode children type
#[derive(Debug)]
pub enum VNodeChildren<'a> {
    Multiple(Vec<'a, TemplateChildNode<'a>>),
    Single(TemplateTextChildNode<'a>),
    Slots(Box<'a, SlotsExpression<'a>>),
    ForRenderList(Box<'a, CallExpression<'a>>),
    Simple(Box<'a, SimpleExpressionNode<'a>>),
    Cache(Box<'a, CacheExpression<'a>>),
}

/// Template text child node
#[derive(Debug)]
pub enum TemplateTextChildNode<'a> {
    Text(Box<'a, super::elements::TextNode>),
    Interpolation(Box<'a, super::elements::InterpolationNode<'a>>),
    Compound(Box<'a, CompoundExpressionNode<'a>>),
}

/// Props expression type
#[derive(Debug)]
pub enum PropsExpression<'a> {
    Object(Box<'a, ObjectExpression<'a>>),
    Call(Box<'a, CallExpression<'a>>),
    Simple(Box<'a, SimpleExpressionNode<'a>>),
}

/// Dynamic props type
#[derive(Debug)]
pub enum DynamicProps<'a> {
    String(String),
    Simple(Box<'a, SimpleExpressionNode<'a>>),
}

/// Directive arguments
#[derive(Debug)]
pub struct DirectiveArguments<'a> {
    pub elements: Vec<'a, DirectiveArgumentNode<'a>>,
    pub loc: SourceLocation,
}

/// Single directive argument
#[derive(Debug)]
pub struct DirectiveArgumentNode<'a> {
    pub directive: String,
    pub exp: Option<ExpressionNode<'a>>,
    pub arg: Option<ExpressionNode<'a>>,
    pub modifiers: Option<Box<'a, ObjectExpression<'a>>>,
}

/// Slots expression
#[derive(Debug)]
pub enum SlotsExpression<'a> {
    Object(Box<'a, ObjectExpression<'a>>),
    Dynamic(Box<'a, CallExpression<'a>>),
}

// ============================================================================
// JavaScript AST Nodes
// ============================================================================

/// All JavaScript child node types for codegen
#[derive(Debug)]
pub enum JsChildNode<'a> {
    VNodeCall(Box<'a, VNodeCall<'a>>),
    Call(Box<'a, CallExpression<'a>>),
    Object(Box<'a, ObjectExpression<'a>>),
    Array(Box<'a, ArrayExpression<'a>>),
    Function(Box<'a, FunctionExpression<'a>>),
    Conditional(Box<'a, ConditionalExpression<'a>>),
    Cache(Box<'a, CacheExpression<'a>>),
    Assignment(Box<'a, AssignmentExpression<'a>>),
    Sequence(Box<'a, SequenceExpression<'a>>),
    SimpleExpression(Box<'a, SimpleExpressionNode<'a>>),
    CompoundExpression(Box<'a, CompoundExpressionNode<'a>>),
}

/// Codegen node union type
#[derive(Debug)]
pub enum CodegenNode<'a> {
    TemplateChild(TemplateChildNode<'a>),
    JsChild(JsChildNode<'a>),
    BlockStatement(Box<'a, BlockStatement<'a>>),
}

/// Call expression
#[derive(Debug)]
pub struct CallExpression<'a> {
    pub callee: Callee,
    pub arguments: Vec<'a, CallArgument<'a>>,
    pub loc: SourceLocation,
}

impl<'a> CallExpression<'a> {
    pub fn new(allocator: &'a Bump, callee: Callee, loc: SourceLocation) -> Self {
        Self {
            callee,
            arguments: Vec::new_in(allocator),
            loc,
        }
    }

    pub fn node_type(&self) -> NodeType {
        NodeType::JsCallExpression
    }
}

/// Callee type
#[derive(Debug)]
pub enum Callee {
    String(String),
    Symbol(RuntimeHelper),
}

/// Call argument type
#[derive(Debug)]
pub enum CallArgument<'a> {
    String(String),
    Symbol(RuntimeHelper),
    JsChild(JsChildNode<'a>),
    TemplateChild(TemplateChildNode<'a>),
    TemplateChildren(Vec<'a, TemplateChildNode<'a>>),
}

/// Object expression
#[derive(Debug)]
pub struct ObjectExpression<'a> {
    pub properties: Vec<'a, Property<'a>>,
    pub loc: SourceLocation,
}

impl<'a> ObjectExpression<'a> {
    pub fn new(allocator: &'a Bump, loc: SourceLocation) -> Self {
        Self {
            properties: Vec::new_in(allocator),
            loc,
        }
    }

    pub fn node_type(&self) -> NodeType {
        NodeType::JsObjectExpression
    }
}

/// Object property
#[derive(Debug)]
pub struct Property<'a> {
    pub key: ExpressionNode<'a>,
    pub value: JsChildNode<'a>,
    pub loc: SourceLocation,
}

impl<'a> Property<'a> {
    pub fn node_type(&self) -> NodeType {
        NodeType::JsProperty
    }
}

/// Array expression
#[derive(Debug)]
pub struct ArrayExpression<'a> {
    pub elements: Vec<'a, ArrayElement<'a>>,
    pub loc: SourceLocation,
}

impl<'a> ArrayExpression<'a> {
    pub fn new(allocator: &'a Bump, loc: SourceLocation) -> Self {
        Self {
            elements: Vec::new_in(allocator),
            loc,
        }
    }

    pub fn node_type(&self) -> NodeType {
        NodeType::JsArrayExpression
    }
}

/// Array element type
#[derive(Debug)]
pub enum ArrayElement<'a> {
    String(String),
    Node(JsChildNode<'a>),
}

/// Function expression
#[derive(Debug)]
pub struct FunctionExpression<'a> {
    pub params: Option<FunctionParams<'a>>,
    pub returns: Option<FunctionReturns<'a>>,
    pub body: Option<FunctionBody<'a>>,
    pub newline: bool,
    pub is_slot: bool,
    pub is_non_scoped_slot: bool,
    pub loc: SourceLocation,
}

impl<'a> FunctionExpression<'a> {
    pub fn node_type(&self) -> NodeType {
        NodeType::JsFunctionExpression
    }
}

/// Function parameters
#[derive(Debug)]
pub enum FunctionParams<'a> {
    Single(ExpressionNode<'a>),
    String(String),
    Multiple(Vec<'a, FunctionParam<'a>>),
}

/// Single function parameter
#[derive(Debug)]
pub enum FunctionParam<'a> {
    Expression(ExpressionNode<'a>),
    String(String),
}

/// Function returns
#[derive(Debug)]
pub enum FunctionReturns<'a> {
    Single(TemplateChildNode<'a>),
    Multiple(Vec<'a, TemplateChildNode<'a>>),
    JsChild(JsChildNode<'a>),
}

/// Function body
#[derive(Debug)]
pub enum FunctionBody<'a> {
    Block(Box<'a, BlockStatement<'a>>),
    If(Box<'a, IfStatement<'a>>),
}

/// Conditional expression (ternary)
#[derive(Debug)]
pub struct ConditionalExpression<'a> {
    pub test: JsChildNode<'a>,
    pub consequent: JsChildNode<'a>,
    pub alternate: JsChildNode<'a>,
    pub newline: bool,
    pub loc: SourceLocation,
}

impl<'a> ConditionalExpression<'a> {
    pub fn node_type(&self) -> NodeType {
        NodeType::JsConditionalExpression
    }
}

/// Cache expression
#[derive(Debug)]
pub struct CacheExpression<'a> {
    pub index: u32,
    pub value: JsChildNode<'a>,
    pub need_pause_tracking: bool,
    pub in_v_once: bool,
    pub need_array_spread: bool,
    pub loc: SourceLocation,
}

impl<'a> CacheExpression<'a> {
    pub fn node_type(&self) -> NodeType {
        NodeType::JsCacheExpression
    }
}

// ============================================================================
// SSR Codegen Nodes
// ============================================================================

/// Block statement
#[derive(Debug)]
pub struct BlockStatement<'a> {
    pub body: Vec<'a, BlockStatementBody<'a>>,
    pub loc: SourceLocation,
}

impl<'a> BlockStatement<'a> {
    pub fn new(allocator: &'a Bump, loc: SourceLocation) -> Self {
        Self {
            body: Vec::new_in(allocator),
            loc,
        }
    }

    pub fn node_type(&self) -> NodeType {
        NodeType::JsBlockStatement
    }
}

/// Block statement body item
#[derive(Debug)]
pub enum BlockStatementBody<'a> {
    JsChild(JsChildNode<'a>),
    If(Box<'a, IfStatement<'a>>),
}

/// Template literal
#[derive(Debug)]
pub struct TemplateLiteral<'a> {
    pub elements: Vec<'a, TemplateLiteralElement<'a>>,
    pub loc: SourceLocation,
}

impl<'a> TemplateLiteral<'a> {
    pub fn node_type(&self) -> NodeType {
        NodeType::JsTemplateLiteral
    }
}

/// Template literal element
#[derive(Debug)]
pub enum TemplateLiteralElement<'a> {
    String(String),
    JsChild(JsChildNode<'a>),
}

/// If statement (SSR)
#[derive(Debug)]
pub struct IfStatement<'a> {
    pub test: ExpressionNode<'a>,
    pub consequent: Box<'a, BlockStatement<'a>>,
    pub alternate: Option<IfStatementAlternate<'a>>,
    pub loc: SourceLocation,
}

impl<'a> IfStatement<'a> {
    pub fn node_type(&self) -> NodeType {
        NodeType::JsIfStatement
    }
}

/// If statement alternate
#[derive(Debug)]
pub enum IfStatementAlternate<'a> {
    If(Box<'a, IfStatement<'a>>),
    Block(Box<'a, BlockStatement<'a>>),
    Return(Box<'a, ReturnStatement<'a>>),
}

/// Assignment expression
#[derive(Debug)]
pub struct AssignmentExpression<'a> {
    pub left: Box<'a, SimpleExpressionNode<'a>>,
    pub right: JsChildNode<'a>,
    pub loc: SourceLocation,
}

impl<'a> AssignmentExpression<'a> {
    pub fn node_type(&self) -> NodeType {
        NodeType::JsAssignmentExpression
    }
}

/// Sequence expression
#[derive(Debug)]
pub struct SequenceExpression<'a> {
    pub expressions: Vec<'a, JsChildNode<'a>>,
    pub loc: SourceLocation,
}

impl<'a> SequenceExpression<'a> {
    pub fn node_type(&self) -> NodeType {
        NodeType::JsSequenceExpression
    }
}

/// Return statement
#[derive(Debug)]
pub struct ReturnStatement<'a> {
    pub returns: ReturnValue<'a>,
    pub loc: SourceLocation,
}

impl<'a> ReturnStatement<'a> {
    pub fn node_type(&self) -> NodeType {
        NodeType::JsReturnStatement
    }
}

/// Return value type
#[derive(Debug)]
pub enum ReturnValue<'a> {
    Single(TemplateChildNode<'a>),
    Multiple(Vec<'a, TemplateChildNode<'a>>),
    JsChild(JsChildNode<'a>),
}
