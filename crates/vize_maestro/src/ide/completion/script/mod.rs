//! Script completion provider.
//!
//! Handles completions within script blocks including Vue Composition API,
//! compiler macros, and import suggestions.
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]

mod context;
mod lists;
mod member_access;
mod reactive_infer;

#[cfg(test)]
mod tests;

use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, CompletionItemLabelDetails, Documentation, MarkupContent,
    MarkupKind,
};
use vize_croquis::ScopeKind;
use vize_croquis::{Drawer, DrawerOptions};
use vize_relief::BindingType;

use self::context::{
    is_nested_user_scope, receiver_is_member_chain, scope_kind_short_label,
    script_content_and_offset_for_context,
};
use self::lists::import_completions;
use self::member_access::complete_member_access;
use self::reactive_infer::reactive_completion_info;
use super::items;
use crate::ide::IdeContext;
use crate::ide::cursor_context::CursorContext;

pub(crate) use self::lists::{composition_api_completions, macro_completions};
pub(crate) use self::reactive_infer::infer_reactive_value_type;

/// Get completions for script context.
pub(crate) fn complete_script(ctx: &IdeContext, is_setup: bool) -> Vec<CompletionItem> {
    if is_setup
        && ctx.uri.path().ends_with(".art.vue")
        && let Some(items) = crate::ide::musea::define_art_source_completions(ctx)
    {
        return items;
    }

    // Route by cursor context so trigger characters get a focused list.
    // Member-access sites either return the ref-`.value` shortcut or fall
    // through to an empty response (Corsa's `complete_with_corsa` path is the
    // source of truth when available). All other shapes fall through to the
    // standard composition-API + bindings list below.
    match CursorContext::detect(&ctx.content, ctx.offset) {
        CursorContext::MemberAccess { receiver, .. } => {
            // The shared detector treats `1.` as member access on `1`. In a
            // script context this is almost always a decimal literal in
            // progress, not a member chain — fall through to the standard
            // completion list so the user keeps seeing Composition-API items.
            if !receiver_is_member_chain(receiver) {
                // continue to standard list
            } else if let Some(items) = complete_member_access(ctx, is_setup)
                && !items.is_empty()
            {
                return items;
            } else {
                return Vec::new();
            }
        }
        CursorContext::HtmlComment => {
            // Inside a script block this should not normally fire, but the
            // detector is shared. Fall through to identifier behavior.
        }
        CursorContext::Other | CursorContext::Identifier { .. } => {}
    }

    let mut items_vec = Vec::new();

    // Add Vue Composition API
    items_vec.extend(composition_api_completions());

    // Add Vue macros (script setup only)
    if is_setup {
        items_vec.extend(macro_completions());
    }

    // Add common imports
    items_vec.extend(import_completions());

    // Use vize_croquis for accurate bindings in script
    if let Some((script_content, script_offset)) =
        script_content_and_offset_for_context(ctx, is_setup)
    {
        let mut analyzer = Drawer::with_options(DrawerOptions {
            analyze_script: true,
            ..Default::default()
        });

        if is_setup {
            analyzer.analyze_script_setup(&script_content);
        } else {
            analyzer.analyze_script_plain(&script_content);
        }

        let croquis = analyzer.finish();

        // Scope-aware completion: include nested bindings (closures, blocks,
        // v-for params, etc.) that are visible at the cursor. We avoid
        // duplicating top-level bindings that the loop below already adds.
        let local_offset = ctx.offset.saturating_sub(script_offset) as u32;
        if local_offset <= script_content.len() as u32 {
            for (name, binding, scope_kind) in croquis.scopes.bindings_visible_at(local_offset) {
                if croquis.bindings.contains(name) {
                    continue;
                }
                if !is_nested_user_scope(scope_kind) {
                    // Module / global scopes are surfaced via the existing
                    // composition_api and import completion blocks; skip them
                    // here to avoid duplicating well-known names.
                    continue;
                }
                items_vec.push(inner_scope_completion_item(
                    name,
                    binding.binding_type,
                    scope_kind,
                ));
            }
        }

        // Add bindings with type information
        for (name, binding_type) in croquis.bindings.iter() {
            let (kind, mut type_detail, mut doc) =
                items::binding_type_to_completion_info(binding_type);
            let reactive_source = croquis.reactivity.lookup(name);
            if let Some(source) = reactive_source
                && let Some((reactive_detail, reactive_doc)) =
                    reactive_completion_info(&script_content, name, source.kind)
            {
                type_detail = reactive_detail;
                doc = reactive_doc;
            }

            // For refs in script, add .value hint
            let needs_value = reactive_source
                .map(|source| source.kind.needs_value_access())
                .unwrap_or_else(|| {
                    matches!(
                        binding_type,
                        BindingType::SetupRef | BindingType::SetupMaybeRef
                    )
                });

            #[allow(clippy::disallowed_macros)]
            items_vec.push(CompletionItem {
                label: name.to_string(),
                kind: Some(kind),
                label_details: Some(CompletionItemLabelDetails {
                    detail: Some(type_detail.clone()),
                    description: if needs_value {
                        Some(".value".to_string())
                    } else {
                        None
                    },
                }),
                detail: Some(type_detail),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: doc,
                })),
                sort_text: Some(format!("0{}", name)),
                ..Default::default()
            });
        }

        // Add reactive sources. Reactive bindings (ref/computed/...) are
        // already emitted by the bindings loop above with their reactive type
        // detail, so skip any source whose name is a known binding to avoid
        // listing the same identifier twice.
        for source in croquis.reactivity.sources() {
            if croquis.bindings.contains(source.name.as_str()) {
                continue;
            }
            let needs_value = source.kind.needs_value_access();
            let (type_detail, doc) =
                reactive_completion_info(&script_content, source.name.as_str(), source.kind)
                    .unwrap_or_else(|| {
                        let kind_str = source.kind.to_display().to_string();
                        let doc = if needs_value {
                            "Needs `.value` access in script.".to_string()
                        } else {
                            "Direct access (no `.value` needed).".to_string()
                        };
                        (kind_str, doc)
                    });

            #[allow(clippy::disallowed_macros)]
            items_vec.push(CompletionItem {
                label: source.name.to_string(),
                kind: Some(CompletionItemKind::VARIABLE),
                label_details: Some(CompletionItemLabelDetails {
                    detail: Some(type_detail.clone()),
                    description: if needs_value {
                        Some(".value".to_string())
                    } else {
                        None
                    },
                }),
                detail: Some(type_detail),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: doc,
                })),
                sort_text: Some(format!("0{}", source.name)),
                ..Default::default()
            });
        }
    }

    items_vec
}

#[allow(clippy::disallowed_macros)]
fn inner_scope_completion_item(
    name: &str,
    binding_type: BindingType,
    scope_kind: ScopeKind,
) -> CompletionItem {
    let (kind, type_detail, doc) = items::binding_type_to_completion_info(binding_type);
    let scope_label = scope_kind_short_label(scope_kind);
    let description = format!("local · {scope_label}");
    CompletionItem {
        label: name.to_string(),
        kind: Some(kind),
        label_details: Some(CompletionItemLabelDetails {
            detail: Some(type_detail.clone()),
            description: Some(description.clone()),
        }),
        detail: Some(format!("{type_detail} (in {scope_label})")),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: format!(
                "**Local binding** in `{scope_label}` scope.\n\n```typescript\n{name}: {type_detail}\n```\n\n{doc}",
            ),
        })),
        // `00` is lexicographically smaller than the `0` prefix used for
        // top-level setup bindings, so closer-scope candidates rank higher in
        // the editor's completion list.
        sort_text: Some(format!("00{name}")),
        ..Default::default()
    }
}
