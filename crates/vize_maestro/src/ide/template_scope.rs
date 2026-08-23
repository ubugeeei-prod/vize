//! Authored template-scope bindings such as `v-for` aliases.
#![allow(clippy::disallowed_types)]

mod v_for;
mod v_slot;

#[cfg(test)]
mod tests;

use tower_lsp::lsp_types::{GotoDefinitionResponse, Hover, Location, Position, Range};

use super::{HoverBuilder, IdeContext, offset_to_position};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TemplateScopeBindingKind {
    Value,
    Key,
    Index,
    SlotProp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TemplateScopeBinding {
    pub(crate) name: String,
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) kind: TemplateScopeBindingKind,
}

impl TemplateScopeBinding {
    pub(crate) fn location(&self, ctx: &IdeContext<'_>) -> Location {
        let (start_line, start_character) = offset_to_position(&ctx.content, self.start);
        let (end_line, end_character) = offset_to_position(&ctx.content, self.end);
        Location {
            uri: ctx.uri.clone(),
            range: Range {
                start: Position {
                    line: start_line,
                    character: start_character,
                },
                end: Position {
                    line: end_line,
                    character: end_character,
                },
            },
        }
    }
}

pub(crate) fn v_for_binding_at(ctx: &IdeContext<'_>, word: &str) -> Option<TemplateScopeBinding> {
    v_for::binding_at(ctx, word)
}

pub(crate) fn v_for_definition(ctx: &IdeContext<'_>, word: &str) -> Option<GotoDefinitionResponse> {
    v_for_binding_at(ctx, word).map(|binding| GotoDefinitionResponse::Scalar(binding.location(ctx)))
}

pub(crate) fn definition(ctx: &IdeContext<'_>, word: &str) -> Option<GotoDefinitionResponse> {
    v_for_definition(ctx, word).or_else(|| v_slot_definition(ctx, word))
}

pub(crate) fn v_for_hover(ctx: &IdeContext<'_>, word: &str) -> Option<Hover> {
    let binding = v_for_binding_at(ctx, word)?;
    let description = match binding.kind {
        TemplateScopeBindingKind::Value => {
            "Loop value alias declared by the nearest `v-for` and available to this template scope."
        }
        TemplateScopeBindingKind::Key => {
            "Loop key/index alias declared by the nearest `v-for` and available to this template scope."
        }
        TemplateScopeBindingKind::Index => {
            "Loop index alias declared by the nearest `v-for` and available to this template scope."
        }
        TemplateScopeBindingKind::SlotProp => return None,
    };

    Some(
        HoverBuilder::new()
            .title(&binding.name)
            .meta("v-for scope binding")
            .description(description)
            .bullets(
                "Behavior",
                &[
                    "Shadows outer template and script bindings with the same name.",
                    "Type-backed sessions should refine this with the iterable item type when the backend answers.",
                ],
            )
            .build(),
    )
}

pub(crate) fn v_slot_binding_at(ctx: &IdeContext<'_>, word: &str) -> Option<TemplateScopeBinding> {
    v_slot::binding_at(ctx, word)
}

pub(crate) fn v_slot_definition(
    ctx: &IdeContext<'_>,
    word: &str,
) -> Option<GotoDefinitionResponse> {
    v_slot_binding_at(ctx, word)
        .map(|binding| GotoDefinitionResponse::Scalar(binding.location(ctx)))
}

pub(crate) fn v_slot_hover(ctx: &IdeContext<'_>, word: &str) -> Option<Hover> {
    let binding = v_slot_binding_at(ctx, word)?;

    Some(
        HoverBuilder::new()
            .title(&binding.name)
            .meta("v-slot scope binding")
            .description(
                "Slot prop binding declared by the nearest scoped slot and available to this template subtree.",
            )
            .bullets(
                "Behavior",
                &[
                    "Shadows outer template and script bindings with the same name.",
                    "Type-backed sessions should refine this with the child component slot prop type when the backend answers.",
                ],
            )
            .build(),
    )
}

pub(crate) fn hover(ctx: &IdeContext<'_>, word: &str) -> Option<Hover> {
    v_for_hover(ctx, word).or_else(|| v_slot_hover(ctx, word))
}
