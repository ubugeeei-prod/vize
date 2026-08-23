//! Indenting JS buffer and the tiny helper set this installment uses.

use vize_carton::String;

/// Vue helpers this installment can mention, ranked the way
/// `vue_helper_import_rank` orders the shipped preamble (`resolveComponent`
/// first, then `withKeys` / `withModifiers`, `toDisplayString`, vnode
/// creates, class/style / props normalizers, `openBlock`, block creates,
/// `Fragment`, `createTextVNode` / `createCommentVNode`, `renderList`).
/// Same-rank helpers keep this array order (`createElementVNode` before
/// `createVNode`, `createBlock` before `createElementBlock`).
#[derive(Clone, Copy)]
enum Helper {
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
}

impl Helper {
    const ALL: [Self; 18] = [
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
    ];

    const fn bit(self) -> u32 {
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
        }
    }

    const fn name(self) -> &'static str {
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
        }
    }

    const fn alias(self) -> &'static str {
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
        }
    }
}

/// Growing JS text plus the helpers the body mentioned.
pub(super) struct Buf {
    pub code: String,
    indent: u32,
    used: u32,
    /// Compact static-props object hoisted as `_hoisted_1` (root only).
    hoisted_props: Option<String>,
}

impl Buf {
    pub(super) fn new() -> Self {
        Self {
            code: String::default(),
            indent: 0,
            used: 0,
            hoisted_props: None,
        }
    }

    pub(super) fn push(&mut self, text: &str) {
        self.code.push_str(text);
    }

    pub(super) fn newline(&mut self) {
        self.code.push('\n');
        for _ in 0..self.indent {
            self.code.push_str("  ");
        }
    }

    pub(super) fn indent(&mut self) {
        self.indent += 1;
    }

    pub(super) fn deindent(&mut self) {
        if self.indent > 0 {
            self.indent -= 1;
        }
    }

    fn mark(&mut self, helper: Helper) {
        self.used |= helper.bit();
    }

    pub(super) fn use_to_display_string(&mut self) {
        self.mark(Helper::ToDisplayString);
    }
    pub(super) fn use_with_keys(&mut self) {
        self.mark(Helper::WithKeys);
    }
    pub(super) fn use_with_modifiers(&mut self) {
        self.mark(Helper::WithModifiers);
    }
    pub(super) fn use_open_block(&mut self) {
        self.mark(Helper::OpenBlock);
    }
    pub(super) fn use_create_element_block(&mut self) {
        self.mark(Helper::CreateElementBlock);
    }
    pub(super) fn use_create_element_vnode(&mut self) {
        self.mark(Helper::CreateElementVNode);
    }
    pub(super) fn use_create_text(&mut self) {
        self.mark(Helper::CreateText);
    }
    pub(super) fn use_normalize_class(&mut self) {
        self.mark(Helper::NormalizeClass);
    }
    pub(super) fn use_normalize_style(&mut self) {
        self.mark(Helper::NormalizeStyle);
    }
    pub(super) fn use_normalize_props(&mut self) {
        self.mark(Helper::NormalizeProps);
    }
    pub(super) fn use_guard_reactive_props(&mut self) {
        self.mark(Helper::GuardReactiveProps);
    }
    pub(super) fn use_merge_props(&mut self) {
        self.mark(Helper::MergeProps);
    }
    pub(super) fn use_create_comment(&mut self) {
        self.mark(Helper::CreateComment);
    }
    pub(super) fn use_fragment(&mut self) {
        self.mark(Helper::Fragment);
    }
    pub(super) fn use_render_list(&mut self) {
        self.mark(Helper::RenderList);
    }
    pub(super) fn use_resolve_component(&mut self) {
        self.mark(Helper::ResolveComponent);
    }
    pub(super) fn use_create_vnode(&mut self) {
        self.mark(Helper::CreateVNode);
    }
    pub(super) fn use_create_block(&mut self) {
        self.mark(Helper::CreateBlock);
    }

    pub(super) fn to_display_string_alias() -> &'static str {
        Helper::ToDisplayString.alias()
    }

    pub(super) fn with_keys_alias() -> &'static str {
        Helper::WithKeys.alias()
    }

    pub(super) fn with_modifiers_alias() -> &'static str {
        Helper::WithModifiers.alias()
    }

    pub(super) fn open_block_alias() -> &'static str {
        Helper::OpenBlock.alias()
    }

    pub(super) fn create_element_block_alias() -> &'static str {
        Helper::CreateElementBlock.alias()
    }

    pub(super) fn create_element_vnode_alias() -> &'static str {
        Helper::CreateElementVNode.alias()
    }

    pub(super) fn create_text_alias() -> &'static str {
        Helper::CreateText.alias()
    }

    pub(super) fn normalize_class_alias() -> &'static str {
        Helper::NormalizeClass.alias()
    }

    pub(super) fn normalize_style_alias() -> &'static str {
        Helper::NormalizeStyle.alias()
    }

    pub(super) fn normalize_props_alias() -> &'static str {
        Helper::NormalizeProps.alias()
    }

    pub(super) fn guard_reactive_props_alias() -> &'static str {
        Helper::GuardReactiveProps.alias()
    }

    pub(super) fn merge_props_alias() -> &'static str {
        Helper::MergeProps.alias()
    }

    pub(super) fn create_comment_alias() -> &'static str {
        Helper::CreateComment.alias()
    }

    pub(super) fn fragment_alias() -> &'static str {
        Helper::Fragment.alias()
    }

    pub(super) fn render_list_alias() -> &'static str {
        Helper::RenderList.alias()
    }

    pub(super) fn resolve_component_alias() -> &'static str {
        Helper::ResolveComponent.alias()
    }

    pub(super) fn create_vnode_alias() -> &'static str {
        Helper::CreateVNode.alias()
    }

    pub(super) fn create_block_alias() -> &'static str {
        Helper::CreateBlock.alias()
    }

    pub(super) fn hoist_root_props(&mut self, object: String) {
        self.hoisted_props = Some(object);
    }

    pub(super) fn hoisted_props_alias() -> &'static str {
        "_hoisted_1"
    }

    /// Function-mode preamble, helpers in import-rank order, then any
    /// root static-props hoist (the shipped codegen appends hoists to
    /// the helper preamble).
    pub(super) fn preamble(&self) -> String {
        let listed: [Helper; 18] = Helper::ALL;
        let mut n = 0;
        for helper in listed {
            if self.used & helper.bit() != 0 {
                n += 1;
            }
        }
        if n == 0 {
            return String::default();
        }
        let mut preamble = String::from("const { ");
        let mut first = true;
        for helper in listed {
            if self.used & helper.bit() == 0 {
                continue;
            }
            if !first {
                preamble.push_str(", ");
            }
            first = false;
            preamble.push_str(helper.name());
            preamble.push_str(": ");
            preamble.push_str(helper.alias());
        }
        preamble.push_str(" } = Vue\n");
        if let Some(object) = &self.hoisted_props {
            preamble.push('\n');
            preamble.push_str("const ");
            preamble.push_str(Self::hoisted_props_alias());
            preamble.push_str(" = ");
            preamble.push_str(object.as_str());
            preamble.push('\n');
        }
        preamble
    }
}
