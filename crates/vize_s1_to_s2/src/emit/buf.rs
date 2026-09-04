//! Indenting JS buffer. Helper names live in [`super::helper`].

use alloc::vec::Vec as StdVec;

use vize_s0::{String, ToCompactString};

use super::helper::Helper;

mod alias;
mod call_position;
mod helper_order;
mod preamble;

/// Growing JS text plus the helpers the body mentioned.
pub(super) struct Buf {
    pub code: String,
    indent: u32,
    used: u64,
    /// Transform-analogue registration order (`root.helpers`).
    preferred: StdVec<Helper>,
    /// The op-visit count each [`Buf::preferred`] entry was registered
    /// at, so a helper the *emit* only learns about mid-walk can still
    /// take the transform's place in that order
    /// ([`Buf::prefer_at_visit`]). Kept only while [`Buf::track_visits`]
    /// is on — `_unref` is its one reader and an inline-only spelling, so
    /// the default lane never spends the allocation.
    preferred_visits: StdVec<u32>,
    /// Whether [`Buf::preferred_visits`] is being kept.
    track_visits: bool,
    /// The visit count [`Buf::prefer`] records; the preference walk sets
    /// it per op.
    prefer_visit: u32,
    /// First `use_*` order (`used_helpers`), with modifier and normalize
    /// helpers reordered by final alias use to match shipped output.
    used_order: StdVec<Helper>,
    /// Compact static-props / vnode RHS values, `_hoisted_1` first.
    hoists: StdVec<String>,
}

impl Buf {
    /// `track_visits` keeps [`Buf::preferred_visits`]; the emit turns it
    /// on for `inline`, whose `_unref` is its only reader.
    pub(super) fn new(track_visits: bool) -> Self {
        Self {
            code: String::default(),
            indent: 0,
            used: 0,
            preferred: StdVec::new(),
            preferred_visits: StdVec::new(),
            track_visits,
            prefer_visit: 0,
            used_order: StdVec::new(),
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

    pub(super) fn indent_width(&self) -> usize {
        self.indent as usize * 2
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
        if self.used & helper.bit() != 0 {
            return;
        }
        self.used |= helper.bit();
        self.used_order.push(helper);
    }

    pub(super) fn use_helper(&mut self, helper: Helper) {
        self.mark(helper);
    }

    pub(super) fn prefer(&mut self, helper: Helper) {
        if self.is_preferred(helper) {
            return;
        }
        self.preferred.push(helper);
        if self.track_visits {
            self.preferred_visits.push(self.prefer_visit);
        }
    }

    fn is_preferred(&self, helper: Helper) -> bool {
        self.preferred.iter().any(|seen| seen.bit() == helper.bit())
    }

    /// Which op the following [`Buf::prefer`] calls stand at, as the
    /// preference walk's op-visit count. The emit walks the same ops in
    /// the same order, so the two counts name the same op.
    pub(super) fn set_prefer_visit(&mut self, visit: u32) {
        self.prefer_visit = visit;
    }

    /// Register `helper` in the transform-analogue order at the op the
    /// emit reached it on, rather than at the end. The shipped lane
    /// registers `_unref` from `process_expression` — a *transform* call
    /// the emit only makes while writing the body — so its place in
    /// `root.helpers` is the op whose expression needed it, ahead of the
    /// helpers that op's own transform step registers afterwards
    /// (`v-for` registers `renderList` *after* processing its source).
    pub(super) fn prefer_at_visit(&mut self, helper: Helper, visit: u32) {
        if self.is_preferred(helper) {
            return;
        }
        let at = self
            .preferred_visits
            .iter()
            .position(|seen| *seen >= visit)
            .unwrap_or(self.preferred.len());
        self.preferred.insert(at, helper);
        if self.track_visits {
            self.preferred_visits.insert(at, visit);
        }
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
    pub(super) fn use_to_handlers(&mut self) {
        self.mark(Helper::ToHandlers);
    }
    pub(super) fn use_to_handler_key(&mut self) {
        self.mark(Helper::ToHandlerKey);
    }
    pub(super) fn use_camelize(&mut self) {
        self.mark(Helper::Camelize);
    }
    pub(super) fn use_with_directives(&mut self) {
        self.mark(Helper::WithDirectives);
    }
    pub(super) fn use_v_show(&mut self) {
        self.mark(Helper::VShow);
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
    pub(super) fn use_resolve_directive(&mut self) {
        self.mark(Helper::ResolveDirective);
    }
    pub(super) fn use_resolve_filter(&mut self) {
        self.mark(Helper::ResolveFilter);
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
    pub(super) fn use_set_block_tracking(&mut self) {
        self.mark(Helper::SetBlockTracking);
    }

    pub(super) fn push_hoist(&mut self, rhs: String) -> String {
        let alias_index = self.hoists.len() + 1;
        self.hoists.push(rhs);
        let mut alias = String::from("_hoisted_");
        alias.push_str(alias_index.to_compact_string().as_str());
        alias
    }

    pub(super) fn hoist_root_props(&mut self, object: String) -> String {
        self.push_hoist(object)
    }
}

#[cfg(test)]
mod tests;
