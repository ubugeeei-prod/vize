//! Indenting JS buffer and the tiny helper set this installment uses.

use vize_carton::String;

/// Vue helpers this installment can mention, ranked the way
/// `vue_helper_import_rank` orders the shipped preamble (`withKeys` /
/// `withModifiers` before `toDisplayString` before vnode creates before
/// class/style normalizers before `openBlock` before block creates
/// before `createTextVNode` / `createCommentVNode`). Same-rank helpers
/// keep this array order (`withKeys` before `withModifiers`).
#[derive(Clone, Copy)]
enum Helper {
    WithKeys,
    WithModifiers,
    ToDisplayString,
    CreateElementVNode,
    NormalizeClass,
    NormalizeStyle,
    OpenBlock,
    CreateElementBlock,
    CreateComment,
    CreateText,
}

impl Helper {
    const ALL: [Self; 10] = [
        Self::WithKeys,
        Self::WithModifiers,
        Self::ToDisplayString,
        Self::CreateElementVNode,
        Self::NormalizeClass,
        Self::NormalizeStyle,
        Self::OpenBlock,
        Self::CreateElementBlock,
        Self::CreateComment,
        Self::CreateText,
    ];

    const fn bit(self) -> u16 {
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
        }
    }

    const fn name(self) -> &'static str {
        match self {
            Self::WithKeys => "withKeys",
            Self::WithModifiers => "withModifiers",
            Self::ToDisplayString => "toDisplayString",
            Self::CreateElementVNode => "createElementVNode",
            Self::NormalizeClass => "normalizeClass",
            Self::NormalizeStyle => "normalizeStyle",
            Self::OpenBlock => "openBlock",
            Self::CreateElementBlock => "createElementBlock",
            Self::CreateText => "createTextVNode",
            Self::CreateComment => "createCommentVNode",
        }
    }

    const fn alias(self) -> &'static str {
        match self {
            Self::WithKeys => "_withKeys",
            Self::WithModifiers => "_withModifiers",
            Self::ToDisplayString => "_toDisplayString",
            Self::CreateElementVNode => "_createElementVNode",
            Self::NormalizeClass => "_normalizeClass",
            Self::NormalizeStyle => "_normalizeStyle",
            Self::OpenBlock => "_openBlock",
            Self::CreateElementBlock => "_createElementBlock",
            Self::CreateText => "_createTextVNode",
            Self::CreateComment => "_createCommentVNode",
        }
    }
}

/// Growing JS text plus the helpers the body mentioned.
pub(super) struct Buf {
    pub code: String,
    indent: u32,
    used: u16,
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

    pub(super) fn use_to_display_string(&mut self) {
        self.used |= Helper::ToDisplayString.bit();
    }

    pub(super) fn use_with_keys(&mut self) {
        self.used |= Helper::WithKeys.bit();
    }

    pub(super) fn use_with_modifiers(&mut self) {
        self.used |= Helper::WithModifiers.bit();
    }

    pub(super) fn use_open_block(&mut self) {
        self.used |= Helper::OpenBlock.bit();
    }

    pub(super) fn use_create_element_block(&mut self) {
        self.used |= Helper::CreateElementBlock.bit();
    }

    pub(super) fn use_create_element_vnode(&mut self) {
        self.used |= Helper::CreateElementVNode.bit();
    }

    pub(super) fn use_create_text(&mut self) {
        self.used |= Helper::CreateText.bit();
    }

    pub(super) fn use_normalize_class(&mut self) {
        self.used |= Helper::NormalizeClass.bit();
    }

    pub(super) fn use_normalize_style(&mut self) {
        self.used |= Helper::NormalizeStyle.bit();
    }

    pub(super) fn use_create_comment(&mut self) {
        self.used |= Helper::CreateComment.bit();
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

    pub(super) fn create_comment_alias() -> &'static str {
        Helper::CreateComment.alias()
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
        let listed: [Helper; 10] = Helper::ALL;
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
