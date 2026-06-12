//! Core AST types and enumerations.
//!
//! Fundamental types used throughout the template AST including
//! node type discriminants, source locations, and constant types.

use serde::{Deserialize, Serialize};
use vize_carton::String;

/// Node type discriminant
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum NodeType {
    Root = 0,
    Element = 1,
    Text = 2,
    Comment = 3,
    SimpleExpression = 4,
    Interpolation = 5,
    Attribute = 6,
    Directive = 7,
    CompoundExpression = 8,
    If = 9,
    IfBranch = 10,
    For = 11,
    TextCall = 12,
    // Codegen nodes
    VNodeCall = 13,
    JsCallExpression = 14,
    JsObjectExpression = 15,
    JsProperty = 16,
    JsArrayExpression = 17,
    JsFunctionExpression = 18,
    JsConditionalExpression = 19,
    JsCacheExpression = 20,
    // SSR codegen nodes
    JsBlockStatement = 21,
    JsTemplateLiteral = 22,
    JsIfStatement = 23,
    JsAssignmentExpression = 24,
    JsSequenceExpression = 25,
    JsReturnStatement = 26,
}

/// Element type discriminant
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum ElementType {
    #[default]
    Element = 0,
    Component = 1,
    Slot = 2,
    Template = 3,
}

/// Namespace for elements
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum Namespace {
    #[default]
    Html = 0,
    Svg = 1,
    MathMl = 2,
}

/// Constant type levels for static analysis
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
#[repr(u8)]
pub enum ConstantType {
    #[default]
    NotConstant = 0,
    CanSkipPatch = 1,
    CanCache = 2,
    CanStringify = 3,
}

/// Source position in the template
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Position {
    /// Byte offset from start of file
    pub offset: u32,
    /// 1-indexed line number
    pub line: u32,
    /// 1-indexed column number
    pub column: u32,
}

impl Position {
    pub const fn new(offset: u32, line: u32, column: u32) -> Self {
        Self {
            offset,
            line,
            column,
        }
    }
}

/// Source location span [start, end)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SourceLocation {
    pub start: Position,
    pub end: Position,
    pub source: String,
}

impl Default for SourceLocation {
    fn default() -> Self {
        Self::STUB
    }
}

/// Static stub location for returning references
pub(crate) static STUB_LOCATION: SourceLocation = SourceLocation {
    start: Position {
        offset: 0,
        line: 1,
        column: 1,
    },
    end: Position {
        offset: 0,
        line: 1,
        column: 1,
    },
    source: String::const_new(""),
};

impl SourceLocation {
    /// Stub location for generated nodes
    pub const STUB: Self = Self {
        start: Position {
            offset: 0,
            line: 1,
            column: 1,
        },
        end: Position {
            offset: 0,
            line: 1,
            column: 1,
        },
        source: String::const_new(""),
    };

    pub fn new(start: Position, end: Position, source: impl Into<String>) -> Self {
        Self {
            start,
            end,
            source: source.into(),
        }
    }
}

/// Runtime helper symbols
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum RuntimeHelper {
    // Core helpers
    Fragment,
    Teleport,
    Suspense,
    KeepAlive,
    BaseTransition,
    Transition,
    TransitionGroup,
    OpenBlock,
    CreateBlock,
    CreateElementBlock,
    CreateVNode,
    CreateElementVNode,
    CreateComment,
    CreateText,
    CreateStatic,
    ResolveComponent,
    ResolveDynamicComponent,
    ResolveDirective,
    ResolveFilter,
    WithDirectives,
    RenderList,
    RenderSlot,
    CreateSlots,
    ToDisplayString,
    MergeProps,
    NormalizeClass,
    NormalizeStyle,
    NormalizeProps,
    GuardReactiveProps,
    ToHandlers,
    Camelize,
    Capitalize,
    ToHandlerKey,
    SetBlockTracking,
    PushScopeId,
    PopScopeId,
    WithCtx,
    Unref,
    IsRef,
    WithMemo,
    IsMemoSame,
    VShow,
    VModelText,
    VModelCheckbox,
    VModelRadio,
    VModelSelect,
    VModelDynamic,
    WithModifiers,
    WithKeys,

    // SSR helpers
    /// SSR text interpolation with escaping
    SsrInterpolate,
    /// SSR VNode rendering
    SsrRenderVNode,
    /// SSR component rendering
    SsrRenderComponent,
    /// SSR slot rendering (with fragment markers)
    SsrRenderSlot,
    /// SSR slot rendering (without fragment markers)
    SsrRenderSlotInner,
    /// SSR render all attributes
    SsrRenderAttrs,
    /// SSR render single attribute
    SsrRenderAttr,
    /// SSR render dynamic key attribute
    SsrRenderDynamicAttr,
    /// SSR boolean attribute inclusion check
    SsrIncludeBooleanAttr,
    /// SSR class stringification
    SsrRenderClass,
    /// SSR style stringification
    SsrRenderStyle,
    /// SSR dynamic input type model rendering
    SsrRenderDynamicModel,
    /// SSR get dynamic v-model props
    SsrGetDynamicModelProps,
    /// SSR v-for list rendering
    SsrRenderList,
    /// SSR loose equality check (for v-model)
    SsrLooseEqual,
    /// SSR array membership check (for v-model)
    SsrLooseContain,
    /// SSR get directive props
    SsrGetDirectiveProps,
    /// SSR teleport rendering
    SsrRenderTeleport,
    /// SSR suspense rendering
    SsrRenderSuspense,
}

impl RuntimeHelper {
    pub fn name(&self) -> &'static str {
        match self {
            // Core helpers
            Self::Fragment => "Fragment",
            Self::Teleport => "Teleport",
            Self::Suspense => "Suspense",
            Self::KeepAlive => "KeepAlive",
            Self::BaseTransition => "BaseTransition",
            Self::Transition => "Transition",
            Self::TransitionGroup => "TransitionGroup",
            Self::OpenBlock => "openBlock",
            Self::CreateBlock => "createBlock",
            Self::CreateElementBlock => "createElementBlock",
            Self::CreateVNode => "createVNode",
            Self::CreateElementVNode => "createElementVNode",
            Self::CreateComment => "createCommentVNode",
            Self::CreateText => "createTextVNode",
            Self::CreateStatic => "createStaticVNode",
            Self::ResolveComponent => "resolveComponent",
            Self::ResolveDynamicComponent => "resolveDynamicComponent",
            Self::ResolveDirective => "resolveDirective",
            Self::ResolveFilter => "resolveFilter",
            Self::WithDirectives => "withDirectives",
            Self::RenderList => "renderList",
            Self::RenderSlot => "renderSlot",
            Self::CreateSlots => "createSlots",
            Self::ToDisplayString => "toDisplayString",
            Self::MergeProps => "mergeProps",
            Self::NormalizeClass => "normalizeClass",
            Self::NormalizeStyle => "normalizeStyle",
            Self::NormalizeProps => "normalizeProps",
            Self::GuardReactiveProps => "guardReactiveProps",
            Self::ToHandlers => "toHandlers",
            Self::Camelize => "camelize",
            Self::Capitalize => "capitalize",
            Self::ToHandlerKey => "toHandlerKey",
            Self::SetBlockTracking => "setBlockTracking",
            Self::PushScopeId => "pushScopeId",
            Self::PopScopeId => "popScopeId",
            Self::WithCtx => "withCtx",
            Self::Unref => "unref",
            Self::IsRef => "isRef",
            Self::WithMemo => "withMemo",
            Self::IsMemoSame => "isMemoSame",
            Self::VShow => "vShow",
            Self::VModelText => "vModelText",
            Self::VModelCheckbox => "vModelCheckbox",
            Self::VModelRadio => "vModelRadio",
            Self::VModelSelect => "vModelSelect",
            Self::VModelDynamic => "vModelDynamic",
            Self::WithModifiers => "withModifiers",
            Self::WithKeys => "withKeys",

            // SSR helpers
            Self::SsrInterpolate => "ssrInterpolate",
            Self::SsrRenderVNode => "ssrRenderVNode",
            Self::SsrRenderComponent => "ssrRenderComponent",
            Self::SsrRenderSlot => "ssrRenderSlot",
            Self::SsrRenderSlotInner => "ssrRenderSlotInner",
            Self::SsrRenderAttrs => "ssrRenderAttrs",
            Self::SsrRenderAttr => "ssrRenderAttr",
            Self::SsrRenderDynamicAttr => "ssrRenderDynamicAttr",
            Self::SsrIncludeBooleanAttr => "ssrIncludeBooleanAttr",
            Self::SsrRenderClass => "ssrRenderClass",
            Self::SsrRenderStyle => "ssrRenderStyle",
            Self::SsrRenderDynamicModel => "ssrRenderDynamicModel",
            Self::SsrGetDynamicModelProps => "ssrGetDynamicModelProps",
            Self::SsrRenderList => "ssrRenderList",
            Self::SsrLooseEqual => "ssrLooseEqual",
            Self::SsrLooseContain => "ssrLooseContain",
            Self::SsrGetDirectiveProps => "ssrGetDirectiveProps",
            Self::SsrRenderTeleport => "ssrRenderTeleport",
            Self::SsrRenderSuspense => "ssrRenderSuspense",
        }
    }

    /// Check if this is an SSR-specific helper
    pub fn is_ssr(&self) -> bool {
        matches!(
            self,
            Self::SsrInterpolate
                | Self::SsrRenderVNode
                | Self::SsrRenderComponent
                | Self::SsrRenderSlot
                | Self::SsrRenderSlotInner
                | Self::SsrRenderAttrs
                | Self::SsrRenderAttr
                | Self::SsrRenderDynamicAttr
                | Self::SsrIncludeBooleanAttr
                | Self::SsrRenderClass
                | Self::SsrRenderStyle
                | Self::SsrRenderDynamicModel
                | Self::SsrGetDynamicModelProps
                | Self::SsrRenderList
                | Self::SsrLooseEqual
                | Self::SsrLooseContain
                | Self::SsrGetDirectiveProps
                | Self::SsrRenderTeleport
                | Self::SsrRenderSuspense
        )
    }
}

/// Import item for code generation
#[derive(Debug)]
pub struct ImportItem<'a> {
    pub exp: vize_carton::Box<'a, super::SimpleExpressionNode<'a>>,
    pub path: String,
}
