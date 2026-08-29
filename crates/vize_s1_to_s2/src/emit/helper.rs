//! Vue helpers this installment can mention, ranked the way
//! `vue_helper_import_rank` orders the shipped preamble. Same-rank
//! helpers follow transform-first registration, then emit order
//! (`Buf::prefer` then first `use_*`), matching `root.helpers`
//! then `used_helpers`.

#[derive(Clone, Copy)]
pub(super) enum Helper {
    ResolveComponent,
    ResolveDynamicComponent,
    ResolveDirective,
    ResolveFilter,
    VModelText,
    VModelCheckbox,
    VModelRadio,
    VModelSelect,
    VShow,
    WithDirectives,
    WithKeys,
    WithModifiers,
    ToDisplayString,
    RenderSlot,
    CreateElementVNode,
    CreateVNode,
    NormalizeClass,
    NormalizeStyle,
    NormalizeProps,
    GuardReactiveProps,
    MergeProps,
    ToHandlers,
    ToHandlerKey,
    Camelize,
    OpenBlock,
    CreateBlock,
    CreateElementBlock,
    Fragment,
    CreateComment,
    CreateText,
    RenderList,
    CreateSlots,
    SetBlockTracking,
    WithMemo,
    IsMemoSame,
    WithCtx,
    Teleport,
    Suspense,
    KeepAlive,
    BaseTransition,
    Transition,
    TransitionGroup,
}

impl Helper {
    pub(super) const ALL: [Self; 42] = [
        Self::ResolveComponent,
        Self::ResolveDynamicComponent,
        Self::ResolveDirective,
        Self::ResolveFilter,
        Self::VModelText,
        Self::VModelCheckbox,
        Self::VModelRadio,
        Self::VModelSelect,
        Self::VShow,
        Self::WithDirectives,
        Self::WithKeys,
        Self::WithModifiers,
        Self::ToDisplayString,
        Self::RenderSlot,
        Self::CreateElementVNode,
        Self::CreateVNode,
        Self::NormalizeClass,
        Self::NormalizeStyle,
        Self::NormalizeProps,
        Self::GuardReactiveProps,
        Self::MergeProps,
        Self::ToHandlers,
        Self::ToHandlerKey,
        Self::Camelize,
        Self::OpenBlock,
        Self::CreateBlock,
        Self::CreateElementBlock,
        Self::Fragment,
        Self::CreateComment,
        Self::CreateText,
        Self::RenderList,
        Self::CreateSlots,
        Self::SetBlockTracking,
        Self::WithMemo,
        Self::IsMemoSame,
        Self::WithCtx,
        Self::Teleport,
        Self::Suspense,
        Self::KeepAlive,
        Self::BaseTransition,
        Self::Transition,
        Self::TransitionGroup,
    ];

    pub(super) const fn rank(self) -> u8 {
        match self {
            Self::ResolveComponent
            | Self::ResolveDynamicComponent
            | Self::ResolveDirective
            | Self::ResolveFilter => 0,
            Self::VModelText | Self::VModelCheckbox | Self::VModelRadio | Self::VModelSelect => 1,
            Self::WithDirectives | Self::WithKeys | Self::WithModifiers => 2,
            Self::ToDisplayString => 3,
            Self::RenderSlot | Self::CreateElementVNode | Self::CreateVNode => 4,
            Self::NormalizeClass
            | Self::NormalizeStyle
            | Self::NormalizeProps
            | Self::GuardReactiveProps
            | Self::MergeProps
            | Self::ToHandlers
            | Self::ToHandlerKey
            | Self::Camelize => 5,
            Self::OpenBlock => 6,
            Self::CreateBlock | Self::CreateElementBlock => 7,
            Self::Fragment => 8,
            Self::CreateComment | Self::CreateText => 9,
            Self::RenderList
            | Self::CreateSlots
            | Self::SetBlockTracking
            | Self::VShow
            | Self::WithMemo
            | Self::IsMemoSame
            | Self::WithCtx
            | Self::Teleport
            | Self::Suspense
            | Self::KeepAlive
            | Self::BaseTransition
            | Self::Transition
            | Self::TransitionGroup => 10,
        }
    }

    pub(super) const fn bit(self) -> u64 {
        match self {
            Self::ToDisplayString => 1,
            Self::CreateElementVNode => 2,
            Self::OpenBlock => 4,
            Self::CreateElementBlock => 8,
            Self::CreateText => 16,
            Self::NormalizeClass => 32,
            Self::NormalizeStyle => 64,
            Self::WithKeys => 128,
            Self::WithModifiers => 256,
            Self::CreateComment => 512,
            Self::Fragment => 1024,
            Self::RenderList => 2048,
            Self::NormalizeProps => 4096,
            Self::GuardReactiveProps => 8192,
            Self::MergeProps => 16384,
            Self::ResolveComponent => 32768,
            Self::CreateVNode => 65536,
            Self::CreateBlock => 131072,
            Self::ToHandlers => 262144,
            Self::Camelize => 524288,
            Self::CreateSlots => 1048576,
            Self::WithCtx => 2097152,
            Self::RenderSlot => 4194304,
            Self::Teleport => 8388608,
            Self::Suspense => 16777216,
            Self::KeepAlive => 33554432,
            Self::BaseTransition => 67108864,
            Self::Transition => 134217728,
            Self::TransitionGroup => 268435456,
            Self::ResolveDynamicComponent => 536870912,
            Self::VModelText => 1073741824,
            Self::VModelCheckbox => 2147483648,
            Self::VModelRadio => 4294967296,
            Self::VModelSelect => 8589934592,
            Self::VShow => 2199023255552,
            Self::WithDirectives => 17179869184,
            Self::ResolveDirective => 34359738368,
            Self::ResolveFilter => 68719476736,
            Self::ToHandlerKey => 137438953472,
            Self::SetBlockTracking => 274877906944,
            Self::WithMemo => 549755813888,
            Self::IsMemoSame => 1099511627776,
        }
    }

    pub(super) const fn name(self) -> &'static str {
        match self {
            Self::ResolveComponent => "resolveComponent",
            Self::ResolveDynamicComponent => "resolveDynamicComponent",
            Self::ResolveDirective => "resolveDirective",
            Self::ResolveFilter => "resolveFilter",
            Self::VModelText => "vModelText",
            Self::VModelCheckbox => "vModelCheckbox",
            Self::VModelRadio => "vModelRadio",
            Self::VModelSelect => "vModelSelect",
            Self::VShow => "vShow",
            Self::WithDirectives => "withDirectives",
            Self::WithKeys => "withKeys",
            Self::WithModifiers => "withModifiers",
            Self::ToDisplayString => "toDisplayString",
            Self::CreateElementVNode => "createElementVNode",
            Self::CreateVNode => "createVNode",
            Self::RenderSlot => "renderSlot",
            Self::NormalizeClass => "normalizeClass",
            Self::NormalizeStyle => "normalizeStyle",
            Self::NormalizeProps => "normalizeProps",
            Self::GuardReactiveProps => "guardReactiveProps",
            Self::MergeProps => "mergeProps",
            Self::ToHandlers => "toHandlers",
            Self::ToHandlerKey => "toHandlerKey",
            Self::Camelize => "camelize",
            Self::OpenBlock => "openBlock",
            Self::CreateBlock => "createBlock",
            Self::CreateElementBlock => "createElementBlock",
            Self::Fragment => "Fragment",
            Self::CreateText => "createTextVNode",
            Self::CreateComment => "createCommentVNode",
            Self::RenderList => "renderList",
            Self::CreateSlots => "createSlots",
            Self::SetBlockTracking => "setBlockTracking",
            Self::WithMemo => "withMemo",
            Self::IsMemoSame => "isMemoSame",
            Self::WithCtx => "withCtx",
            Self::Teleport => "Teleport",
            Self::Suspense => "Suspense",
            Self::KeepAlive => "KeepAlive",
            Self::BaseTransition => "BaseTransition",
            Self::Transition => "Transition",
            Self::TransitionGroup => "TransitionGroup",
        }
    }

    pub(super) const fn alias(self) -> &'static str {
        match self {
            Self::ResolveComponent => "_resolveComponent",
            Self::ResolveDynamicComponent => "_resolveDynamicComponent",
            Self::ResolveDirective => "_resolveDirective",
            Self::ResolveFilter => "_resolveFilter",
            Self::VModelText => "_vModelText",
            Self::VModelCheckbox => "_vModelCheckbox",
            Self::VModelRadio => "_vModelRadio",
            Self::VModelSelect => "_vModelSelect",
            Self::VShow => "_vShow",
            Self::WithDirectives => "_withDirectives",
            Self::WithKeys => "_withKeys",
            Self::WithModifiers => "_withModifiers",
            Self::ToDisplayString => "_toDisplayString",
            Self::CreateElementVNode => "_createElementVNode",
            Self::CreateVNode => "_createVNode",
            Self::RenderSlot => "_renderSlot",
            Self::NormalizeClass => "_normalizeClass",
            Self::NormalizeStyle => "_normalizeStyle",
            Self::NormalizeProps => "_normalizeProps",
            Self::GuardReactiveProps => "_guardReactiveProps",
            Self::MergeProps => "_mergeProps",
            Self::ToHandlers => "_toHandlers",
            Self::ToHandlerKey => "_toHandlerKey",
            Self::Camelize => "_camelize",
            Self::OpenBlock => "_openBlock",
            Self::CreateBlock => "_createBlock",
            Self::CreateElementBlock => "_createElementBlock",
            Self::Fragment => "_Fragment",
            Self::CreateText => "_createTextVNode",
            Self::CreateComment => "_createCommentVNode",
            Self::RenderList => "_renderList",
            Self::CreateSlots => "_createSlots",
            Self::SetBlockTracking => "_setBlockTracking",
            Self::WithMemo => "_withMemo",
            Self::IsMemoSame => "_isMemoSame",
            Self::WithCtx => "_withCtx",
            Self::Teleport => "_Teleport",
            Self::Suspense => "_Suspense",
            Self::KeepAlive => "_KeepAlive",
            Self::BaseTransition => "_BaseTransition",
            Self::Transition => "_Transition",
            Self::TransitionGroup => "_TransitionGroup",
        }
    }
}
