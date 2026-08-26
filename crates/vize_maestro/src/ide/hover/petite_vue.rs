use tower_lsp::lsp_types::Hover;
use vize_croquis::{Drawer, DrawerOptions, ScopeKind};

use super::{HoverBuilder, HoverService};
use crate::ide::IdeContext;

impl HoverService {
    /// Get hover for a petite-vue `v-scope` binding in a standalone HTML
    /// document.
    pub(super) fn hover_petite_vue_scope_binding(ctx: &IdeContext, word: &str) -> Option<Hover> {
        if !crate::utils::is_standalone_html_path(ctx.uri.path()) || !ctx.dialect().is_petite_vue()
        {
            return None;
        }

        let allocator = vize_s0::Allocator::new();
        let (root, _errors) = vize_armature::parse_document(&allocator, &ctx.content);

        let mut drawer = Drawer::with_options(DrawerOptions::full());
        drawer.draw_template(&root);
        let croquis = drawer.finish();

        let offset = ctx.offset.min(ctx.content.len()) as u32;
        let (binding, scope_kind) = croquis
            .scopes
            .bindings_visible_at(offset)
            .into_iter()
            .find(|(name, _, scope_kind)| {
                *name == word
                    && matches!(
                        scope_kind,
                        ScopeKind::VSlot
                            | ScopeKind::VFor
                            | ScopeKind::EventHandler
                            | ScopeKind::Callback
                    )
            })
            .map(|(_, binding, scope_kind)| (binding, scope_kind))?;

        let inferred_type = Self::binding_type_to_ts_display(binding.binding_type);
        #[allow(clippy::disallowed_macros)]
        let signature = format!("{word}: {inferred_type}");

        let scope_note = match scope_kind {
            ScopeKind::VFor => {
                "Local binding introduced by a `v-for` inside the `v-scope` subtree."
            }
            ScopeKind::EventHandler | ScopeKind::Callback => {
                "Local binding visible inside the `v-scope` subtree."
            }
            _ => "Reactive key declared by the enclosing `v-scope` object.",
        };

        Some(
            HoverBuilder::new()
                .title(word)
                .meta("petite-vue scope binding")
                .code("typescript", &signature)
                .description(
                    "Resolved from the petite-vue `v-scope` chain of this standalone HTML document.",
                )
                .bullets(
                    "Behavior",
                    &[
                        scope_note,
                        "Visible only inside its `v-scope` subtree's expressions and directives.",
                    ],
                )
                .build(),
        )
    }
}
