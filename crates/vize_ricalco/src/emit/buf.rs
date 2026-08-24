//! Indenting JS buffer. Helper names live in [`super::helper`].

use alloc::vec::Vec as StdVec;

use vize_carton::{String, ToCompactString};

use super::helper::Helper;

/// Growing JS text plus the helpers the body mentioned.
pub(super) struct Buf {
    pub code: String,
    indent: u32,
    used: u32,
    /// Transform-analogue registration order (`root.helpers`).
    preferred: StdVec<Helper>,
    /// Compact static-props / vnode RHS values, `_hoisted_1` first.
    hoists: StdVec<String>,
}

impl Buf {
    pub(super) fn new() -> Self {
        Self {
            code: String::default(),
            indent: 0,
            used: 0,
            preferred: StdVec::new(),
            hoists: StdVec::new(),
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

    pub(super) fn prefer(&mut self, helper: Helper) {
        if self.preferred.iter().any(|seen| seen.bit() == helper.bit()) {
            return;
        }
        self.preferred.push(helper);
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
    pub(super) fn use_render_slot(&mut self) {
        self.mark(Helper::RenderSlot);
    }
    pub(super) fn use_create_block(&mut self) {
        self.mark(Helper::CreateBlock);
    }
    pub(super) fn use_with_ctx(&mut self) {
        self.mark(Helper::WithCtx);
    }
    pub(super) fn use_create_slots(&mut self) {
        self.mark(Helper::CreateSlots);
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

    pub(super) fn render_slot_alias() -> &'static str {
        Helper::RenderSlot.alias()
    }

    pub(super) fn create_block_alias() -> &'static str {
        Helper::CreateBlock.alias()
    }

    pub(super) fn with_ctx_alias() -> &'static str {
        Helper::WithCtx.alias()
    }

    pub(super) fn create_slots_alias() -> &'static str {
        Helper::CreateSlots.alias()
    }

    pub(super) fn push_hoist(&mut self, rhs: String) -> String {
        self.hoists.push(rhs);
        let mut alias = String::from("_hoisted_");
        alias.push_str(self.hoists.len().to_compact_string().as_str());
        alias
    }

    pub(super) fn hoist_root_props(&mut self, object: String) {
        self.push_hoist(object);
    }

    pub(super) fn hoisted_props_alias() -> &'static str {
        "_hoisted_1"
    }

    fn ordered_helpers(&self) -> StdVec<Helper> {
        let mut listed = StdVec::new();
        let mut bits = 0u32;
        let mut push = |helper: Helper| {
            if self.used & helper.bit() == 0 || bits & helper.bit() != 0 {
                return;
            }
            bits |= helper.bit();
            listed.push(helper);
        };
        for helper in self.preferred.iter().copied() {
            push(helper);
        }
        for helper in Helper::ALL {
            push(helper);
        }
        listed.sort_by_key(|helper| helper.rank());
        listed
    }

    /// Function-mode preamble, helpers in import-rank order, then any
    /// root static-props hoist (the shipped codegen appends hoists to
    /// the helper preamble).
    pub(super) fn preamble(&self) -> String {
        let listed = self.ordered_helpers();
        if listed.is_empty() {
            return String::default();
        }
        let mut preamble = String::from("const { ");
        for (i, helper) in listed.iter().enumerate() {
            if i > 0 {
                preamble.push_str(", ");
            }
            preamble.push_str(helper.name());
            preamble.push_str(": ");
            preamble.push_str(helper.alias());
        }
        preamble.push_str(" } = Vue\n");
        if !self.hoists.is_empty() {
            preamble.push('\n');
            for (i, rhs) in self.hoists.iter().enumerate() {
                preamble.push_str("const _hoisted_");
                preamble.push_str((i + 1).to_compact_string().as_str());
                preamble.push_str(" = ");
                preamble.push_str(rhs.as_str());
                preamble.push('\n');
            }
        }
        preamble
    }
}
