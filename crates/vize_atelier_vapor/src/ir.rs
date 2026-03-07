//! Vapor Intermediate Representation (IR) types.

use serde::{Deserialize, Serialize};
use vize_atelier_core::{Namespace, RootNode, SimpleExpressionNode, TemplateChildNode};
use vize_carton::{Box, Bump, FxHashMap, FxHashSet, String, Vec};

/// IR node type discriminant
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum IRNodeType {
    Root = 0,
    Block = 1,
    SetProp = 2,
    SetDynamicProps = 3,
    SetText = 4,
    SetEvent = 5,
    SetDynamicEvents = 6,
    SetHtml = 7,
    SetTemplateRef = 8,
    InsertNode = 9,
    PrependNode = 10,
    CreateComponentNode = 11,
    SlotOutletNode = 12,
    Directive = 13,
    DeclareOldRef = 14,
    If = 15,
    For = 16,
    GetTextChild = 17,
}

/// Dynamic flags for IR nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum DynamicFlag {
    #[default]
    None = 0,
    Referenced = 1,
    NonTemplate = 2,
    Insert = 4,
}

/// Root IR node for Vapor mode
#[derive(Debug)]
pub struct RootIRNode<'a> {
    pub node: RootNode<'a>,
    pub source: String,
    pub template: FxHashMap<String, Namespace>,
    pub template_index_map: FxHashMap<String, usize>,
    pub root_template_indexes: Vec<'a, usize>,
    pub component: Vec<'a, String>,
    pub directive: Vec<'a, String>,
    pub block: BlockIRNode<'a>,
    pub has_template_ref: bool,
    pub has_deferred_v_show: bool,
    /// Template strings for static parts
    pub templates: Vec<'a, String>,
    /// Mapping from element ID to template index
    pub element_template_map: FxHashMap<usize, usize>,
    /// Element IDs that are standalone text nodes (interpolations with their own template)
    pub standalone_text_elements: FxHashSet<usize>,
}

/// Block IR node - unit of reactive computation
#[derive(Debug)]
pub struct BlockIRNode<'a> {
    pub node: Option<TemplateChildNode<'a>>,
    pub dynamic: IRDynamicInfo,
    pub temp_id: usize,
    pub effect: Vec<'a, IREffect<'a>>,
    pub operation: Vec<'a, OperationNode<'a>>,
    pub returns: Vec<'a, usize>,
}

impl<'a> BlockIRNode<'a> {
    pub fn new(allocator: &'a Bump) -> Self {
        Self {
            node: None,
            dynamic: IRDynamicInfo::default(),
            temp_id: 0,
            effect: Vec::new_in(allocator),
            operation: Vec::new_in(allocator),
            returns: Vec::new_in(allocator),
        }
    }
}

/// Dynamic info for IR nodes
#[derive(Debug, Default)]
pub struct IRDynamicInfo {
    pub flags: u8,
    pub children: std::vec::Vec<IRDynamicInfo>,
    pub id: Option<usize>,
}

/// IR effect
#[derive(Debug)]
pub struct IREffect<'a> {
    pub operations: Vec<'a, OperationNode<'a>>,
}

/// All operation node variants
#[derive(Debug)]
pub enum OperationNode<'a> {
    SetProp(SetPropIRNode<'a>),
    SetDynamicProps(SetDynamicPropsIRNode<'a>),
    SetText(SetTextIRNode<'a>),
    SetEvent(SetEventIRNode<'a>),
    SetHtml(SetHtmlIRNode<'a>),
    SetTemplateRef(SetTemplateRefIRNode<'a>),
    InsertNode(InsertNodeIRNode),
    PrependNode(PrependNodeIRNode),
    Directive(DirectiveIRNode<'a>),
    If(Box<'a, IfIRNode<'a>>),
    For(Box<'a, ForIRNode<'a>>),
    CreateComponent(CreateComponentIRNode<'a>),
    SlotOutlet(SlotOutletIRNode<'a>),
    GetTextChild(GetTextChildIRNode),
    ChildRef(ChildRefIRNode),
    NextRef(NextRefIRNode),
}

/// Set prop operation
#[derive(Debug)]
pub struct SetPropIRNode<'a> {
    pub element: usize,
    pub prop: IRProp<'a>,
    pub tag: String,
    /// `.camel` modifier was used
    pub camel: bool,
    /// `.prop` modifier was used
    pub prop_modifier: bool,
}

/// IR prop
#[derive(Debug)]
pub struct IRProp<'a> {
    pub key: Box<'a, SimpleExpressionNode<'a>>,
    pub values: Vec<'a, Box<'a, SimpleExpressionNode<'a>>>,
    pub is_component: bool,
}

/// Set dynamic props operation
#[derive(Debug)]
pub struct SetDynamicPropsIRNode<'a> {
    pub element: usize,
    pub props: Vec<'a, Box<'a, SimpleExpressionNode<'a>>>,
    pub is_event: bool,
}

/// Set text operation
#[derive(Debug)]
pub struct SetTextIRNode<'a> {
    pub element: usize,
    pub values: Vec<'a, Box<'a, SimpleExpressionNode<'a>>>,
}

/// Set event operation
#[derive(Debug)]
pub struct SetEventIRNode<'a> {
    pub element: usize,
    pub key: Box<'a, SimpleExpressionNode<'a>>,
    pub value: Option<Box<'a, SimpleExpressionNode<'a>>>,
    pub modifiers: EventModifiers,
    pub delegate: bool,
    pub effect: bool,
}

/// Event modifiers
#[derive(Debug, Default)]
pub struct EventModifiers {
    pub keys: std::vec::Vec<String>,
    pub non_keys: std::vec::Vec<String>,
    pub options: EventOptions,
}

/// Event options
#[derive(Debug, Default)]
pub struct EventOptions {
    pub capture: bool,
    pub once: bool,
    pub passive: bool,
}

/// Set HTML operation
#[derive(Debug)]
pub struct SetHtmlIRNode<'a> {
    pub element: usize,
    pub value: Box<'a, SimpleExpressionNode<'a>>,
}

/// Set template ref operation
#[derive(Debug)]
pub struct SetTemplateRefIRNode<'a> {
    pub element: usize,
    pub value: Box<'a, SimpleExpressionNode<'a>>,
    pub ref_for: bool,
}

/// Insert node operation
#[derive(Debug)]
pub struct InsertNodeIRNode {
    pub elements: std::vec::Vec<usize>,
    pub parent: usize,
    pub anchor: Option<usize>,
}

/// Prepend node operation
#[derive(Debug)]
pub struct PrependNodeIRNode {
    pub elements: std::vec::Vec<usize>,
    pub parent: usize,
}

/// Directive operation
#[derive(Debug)]
pub struct DirectiveIRNode<'a> {
    pub element: usize,
    pub dir: Box<'a, vize_atelier_core::DirectiveNode<'a>>,
    pub name: String,
    pub builtin: bool,
    /// Element tag name (for v-model type detection)
    pub tag: String,
    /// Input type attribute (for v-model checkbox/radio detection)
    pub input_type: String,
}

/// If operation
#[derive(Debug)]
pub struct IfIRNode<'a> {
    pub id: usize,
    pub condition: Box<'a, SimpleExpressionNode<'a>>,
    pub positive: BlockIRNode<'a>,
    pub negative: Option<NegativeBranch<'a>>,
    pub once: bool,
    pub parent: Option<usize>,
    pub anchor: Option<usize>,
}

/// Negative branch of if
#[derive(Debug)]
pub enum NegativeBranch<'a> {
    Block(BlockIRNode<'a>),
    If(Box<'a, IfIRNode<'a>>),
}

/// For operation
#[derive(Debug)]
pub struct ForIRNode<'a> {
    pub id: usize,
    pub source: Box<'a, SimpleExpressionNode<'a>>,
    pub value: Option<Box<'a, SimpleExpressionNode<'a>>>,
    pub key: Option<Box<'a, SimpleExpressionNode<'a>>>,
    pub index: Option<Box<'a, SimpleExpressionNode<'a>>>,
    pub key_prop: Option<Box<'a, SimpleExpressionNode<'a>>>,
    pub render: BlockIRNode<'a>,
    pub once: bool,
    pub component: bool,
    pub only_child: bool,
}

/// Component kind for code generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentKind {
    /// Regular user component: resolveComponent + createComponentWithFallback
    Regular,
    /// Teleport: VaporTeleport + createComponent
    Teleport,
    /// KeepAlive: VaporKeepAlive + createComponent
    KeepAlive,
    /// Suspense: resolveComponent("Suspense") + createComponentWithFallback, slots wrapped with withVaporCtx
    Suspense,
    /// Dynamic component: createDynamicComponent
    Dynamic,
}

/// Create component operation
#[derive(Debug)]
pub struct CreateComponentIRNode<'a> {
    pub id: usize,
    pub tag: String,
    pub props: Vec<'a, IRProp<'a>>,
    pub slots: Vec<'a, IRSlot<'a>>,
    pub asset: bool,
    pub once: bool,
    pub dynamic_slots: bool,
    /// Component kind for built-in / dynamic handling
    pub kind: ComponentKind,
    /// Dynamic component `:is` expression (for ComponentKind::Dynamic)
    pub is_expr: Option<Box<'a, SimpleExpressionNode<'a>>>,
    /// v-show expression to apply after component creation
    pub v_show: Option<Box<'a, SimpleExpressionNode<'a>>>,
}

/// IR slot
#[derive(Debug)]
pub struct IRSlot<'a> {
    pub name: Box<'a, SimpleExpressionNode<'a>>,
    pub fn_exp: Option<Box<'a, SimpleExpressionNode<'a>>>,
    pub block: BlockIRNode<'a>,
}

/// Slot outlet operation
#[derive(Debug)]
pub struct SlotOutletIRNode<'a> {
    pub id: usize,
    pub name: Box<'a, SimpleExpressionNode<'a>>,
    pub props: Vec<'a, IRProp<'a>>,
    pub fallback: Option<BlockIRNode<'a>>,
}

/// Get text child operation
#[derive(Debug)]
pub struct GetTextChildIRNode {
    pub parent: usize,
}

/// Child reference operation (_child helper)
#[derive(Debug)]
pub struct ChildRefIRNode {
    pub child_id: usize,
    pub parent_id: usize,
}

/// Next sibling reference operation (_next helper)
#[derive(Debug)]
pub struct NextRefIRNode {
    pub child_id: usize,
    pub prev_id: usize,
    pub offset: usize,
}
