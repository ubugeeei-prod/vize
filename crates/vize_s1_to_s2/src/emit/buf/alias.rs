//! The helper alias name table: `Buf::<helper>_alias()` for every helper
//! the emitter spells by name. Split out of `buf.rs` under the 350-line
//! source budget; these are pure name lookups on [`Helper`], with no
//! buffer state of their own.

use super::super::helper::Helper;
use super::Buf;

impl Buf {
    pub(in crate::emit) fn to_display_string_alias() -> &'static str {
        Helper::ToDisplayString.alias()
    }

    pub(in crate::emit) fn with_keys_alias() -> &'static str {
        Helper::WithKeys.alias()
    }

    pub(in crate::emit) fn with_modifiers_alias() -> &'static str {
        Helper::WithModifiers.alias()
    }

    pub(in crate::emit) fn open_block_alias() -> &'static str {
        Helper::OpenBlock.alias()
    }

    pub(in crate::emit) fn create_element_block_alias() -> &'static str {
        Helper::CreateElementBlock.alias()
    }

    pub(in crate::emit) fn create_element_vnode_alias() -> &'static str {
        Helper::CreateElementVNode.alias()
    }

    pub(in crate::emit) fn create_text_alias() -> &'static str {
        Helper::CreateText.alias()
    }

    pub(in crate::emit) fn normalize_class_alias() -> &'static str {
        Helper::NormalizeClass.alias()
    }

    pub(in crate::emit) fn normalize_style_alias() -> &'static str {
        Helper::NormalizeStyle.alias()
    }

    pub(in crate::emit) fn normalize_props_alias() -> &'static str {
        Helper::NormalizeProps.alias()
    }

    pub(in crate::emit) fn guard_reactive_props_alias() -> &'static str {
        Helper::GuardReactiveProps.alias()
    }

    pub(in crate::emit) fn merge_props_alias() -> &'static str {
        Helper::MergeProps.alias()
    }

    pub(in crate::emit) fn to_handlers_alias() -> &'static str {
        Helper::ToHandlers.alias()
    }

    pub(in crate::emit) fn to_handler_key_alias() -> &'static str {
        Helper::ToHandlerKey.alias()
    }

    pub(in crate::emit) fn camelize_alias() -> &'static str {
        Helper::Camelize.alias()
    }

    pub(in crate::emit) fn with_directives_alias() -> &'static str {
        Helper::WithDirectives.alias()
    }

    pub(in crate::emit) fn v_show_alias() -> &'static str {
        Helper::VShow.alias()
    }

    pub(in crate::emit) fn create_comment_alias() -> &'static str {
        Helper::CreateComment.alias()
    }

    pub(in crate::emit) fn fragment_alias() -> &'static str {
        Helper::Fragment.alias()
    }

    pub(in crate::emit) fn render_list_alias() -> &'static str {
        Helper::RenderList.alias()
    }

    pub(in crate::emit) fn resolve_component_alias() -> &'static str {
        Helper::ResolveComponent.alias()
    }

    pub(in crate::emit) fn resolve_directive_alias() -> &'static str {
        Helper::ResolveDirective.alias()
    }

    pub(in crate::emit) fn resolve_filter_alias() -> &'static str {
        Helper::ResolveFilter.alias()
    }

    pub(in crate::emit) fn create_vnode_alias() -> &'static str {
        Helper::CreateVNode.alias()
    }

    pub(in crate::emit) fn render_slot_alias() -> &'static str {
        Helper::RenderSlot.alias()
    }

    pub(in crate::emit) fn create_block_alias() -> &'static str {
        Helper::CreateBlock.alias()
    }

    pub(in crate::emit) fn with_ctx_alias() -> &'static str {
        Helper::WithCtx.alias()
    }

    pub(in crate::emit) fn create_slots_alias() -> &'static str {
        Helper::CreateSlots.alias()
    }

    pub(in crate::emit) fn set_block_tracking_alias() -> &'static str {
        Helper::SetBlockTracking.alias()
    }
}
