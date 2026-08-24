//! Vue helpers this installment can mention, ranked the way
//! `vue_helper_import_rank` orders the shipped preamble. Same-rank
//! helpers keep [`Helper::ALL`] order (`createElementVNode` before
//! `createVNode`, `createBlock` before `createElementBlock`,
//! `withCtx` after `renderList`).

#[derive(Clone, Copy)]
pub(super) enum Helper {
    ResolveComponent,
    WithKeys,
    WithModifiers,
    ToDisplayString,
    CreateElementVNode,
    CreateVNode,
    NormalizeClass,
    NormalizeStyle,
    NormalizeProps,
    GuardReactiveProps,
    MergeProps,
    OpenBlock,
    CreateBlock,
    CreateElementBlock,
    Fragment,
    CreateComment,
    CreateText,
    RenderList,
    WithCtx,
}

impl Helper {
    pub(super) const ALL: [Self; 19] = [
        Self::ResolveComponent,
        Self::WithKeys,
        Self::WithModifiers,
        Self::ToDisplayString,
        Self::CreateElementVNode,
        Self::CreateVNode,
        Self::NormalizeClass,
        Self::NormalizeStyle,
        Self::NormalizeProps,
        Self::GuardReactiveProps,
        Self::MergeProps,
        Self::OpenBlock,
        Self::CreateBlock,
        Self::CreateElementBlock,
        Self::Fragment,
        Self::CreateComment,
        Self::CreateText,
        Self::RenderList,
        Self::WithCtx,
    ];

    pub(super) const fn bit(self) -> u32 {
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
            Self::WithCtx => 262144,
        }
    }

    pub(super) const fn name(self) -> &'static str {
        match self {
            Self::ResolveComponent => "resolveComponent",
            Self::WithKeys => "withKeys",
            Self::WithModifiers => "withModifiers",
            Self::ToDisplayString => "toDisplayString",
            Self::CreateElementVNode => "createElementVNode",
            Self::CreateVNode => "createVNode",
            Self::NormalizeClass => "normalizeClass",
            Self::NormalizeStyle => "normalizeStyle",
            Self::NormalizeProps => "normalizeProps",
            Self::GuardReactiveProps => "guardReactiveProps",
            Self::MergeProps => "mergeProps",
            Self::OpenBlock => "openBlock",
            Self::CreateBlock => "createBlock",
            Self::CreateElementBlock => "createElementBlock",
            Self::Fragment => "Fragment",
            Self::CreateText => "createTextVNode",
            Self::CreateComment => "createCommentVNode",
            Self::RenderList => "renderList",
            Self::WithCtx => "withCtx",
        }
    }

    pub(super) const fn alias(self) -> &'static str {
        match self {
            Self::ResolveComponent => "_resolveComponent",
            Self::WithKeys => "_withKeys",
            Self::WithModifiers => "_withModifiers",
            Self::ToDisplayString => "_toDisplayString",
            Self::CreateElementVNode => "_createElementVNode",
            Self::CreateVNode => "_createVNode",
            Self::NormalizeClass => "_normalizeClass",
            Self::NormalizeStyle => "_normalizeStyle",
            Self::NormalizeProps => "_normalizeProps",
            Self::GuardReactiveProps => "_guardReactiveProps",
            Self::MergeProps => "_mergeProps",
            Self::OpenBlock => "_openBlock",
            Self::CreateBlock => "_createBlock",
            Self::CreateElementBlock => "_createElementBlock",
            Self::Fragment => "_Fragment",
            Self::CreateText => "_createTextVNode",
            Self::CreateComment => "_createCommentVNode",
            Self::RenderList => "_renderList",
            Self::WithCtx => "_withCtx",
        }
    }
}
