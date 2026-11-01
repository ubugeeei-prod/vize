//! Frozen base template dispatcher; no payload normalization.

use super::super::{
    component_event, corsa_support, helpers, import_target, is_component_tag, template,
};
use crate::ide::{DefinitionService, IdeContext};
use std::sync::Arc;
use tower_lsp::lsp_types::GotoDefinitionResponse;
use vize_canon::CorsaBridge;

impl DefinitionService {
    // Complete actual-base template dispatcher; only the function name changes.
    #[cfg(feature = "native")]
    pub(super) async fn original_template_definition(
        ctx: &IdeContext<'_>,
        corsa_bridge: Option<Arc<CorsaBridge>>,
    ) -> Option<GotoDefinitionResponse> {
        // A successful manual import walk is deterministic and follows barrel
        // re-exports to their source instead of returning the barrel module.
        if let Some(def) = import_target::component_tag_definition(ctx) {
            return Some(def);
        }
        if import_target::component_tag_import_target_is_deleted(ctx) {
            return None;
        }

        if let Some(definition) = component_event::definition(ctx) {
            return Some(definition);
        }

        if let Some(tag_name) = helpers::get_tag_at_offset(&ctx.content, ctx.offset)
            && tag_name == "Self"
            && let Some(def) = template::find_component_definition(ctx, &tag_name)
        {
            return Some(def);
        }

        if let Some(definition) =
            Self::definition_for_html_attribute_with_corsa(ctx, corsa_bridge.as_ref()).await
        {
            return Some(definition);
        }

        if let Some(definition) =
            Self::definition_for_html_tag_with_corsa(ctx, corsa_bridge.as_ref()).await
        {
            return Some(definition);
        }

        if let Some(tag_name) = helpers::get_tag_at_offset(&ctx.content, ctx.offset)
            && is_component_tag(&tag_name)
            && let Some(def) = template::find_component_definition(ctx, &tag_name)
        {
            return Some(def);
        }

        if let Some(def) = template::find_component_prop_definition(ctx) {
            return Some(def);
        }

        if let Some(definition) =
            Self::definition_via_canonical_corsa(ctx, corsa_bridge.as_ref()).await
        {
            return definition.and_then(|definition| {
                import_target::normalize_bound_name_definition(ctx, definition)
            });
        }

        if corsa_support::is_component_attribute_query(ctx) {
            return None;
        }
        let word = helpers::get_word_at_offset(&ctx.content, ctx.offset)?;

        if word.is_empty() {
            return None;
        }

        if !crate::ide::is_in_vue_template_expression(&ctx.content, ctx.offset) {
            return Self::definition_in_template_sync(ctx);
        }

        // Check if this is a props property access
        if let Some(def) = template::find_props_property_definition(ctx, &word) {
            return Some(def);
        }

        // Check if this is a prop name used directly in template
        if helpers::is_in_vue_directive_expression(ctx)
            && let Some(descriptor) = ctx.descriptor()
            && let Some(def) = template::find_prop_definition_by_name(ctx, descriptor, &word)
        {
            return Some(def);
        }

        // Fall back to synchronous definition, unwrapping only an actual
        // import-alias answer so template-local shadowing remains intact.
        let definition = Self::definition_in_template_sync(ctx)?;
        import_target::normalize_bound_name_definition(ctx, definition)
    }
}
