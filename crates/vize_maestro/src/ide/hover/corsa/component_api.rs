//! Component API names that only the type project can describe: slot names
//! and listeners whose contract the resident metadata cannot read.

use std::sync::Arc;

use tower_lsp::lsp_types::Hover;
use vize_canon::CorsaBridge;

use super::canonical::{self, Answer};
use crate::ide::IdeContext;
use crate::ide::definition::helpers;

/// The argument of `#name` / `v-slot:name` on a `<template>` or component.
pub(in crate::ide::hover) fn is_slot_name_query(ctx: &IdeContext<'_>) -> bool {
    let Some(attribute) = helpers::raw_attribute_with_source_span_at_offset(ctx) else {
        return false;
    };
    if attribute.tag_name != "template" && !crate::ide::is_component_tag(&attribute.tag_name) {
        return false;
    }
    let raw = attribute.raw_name.as_str();
    let Some(prefix) = ["#", "v-slot:"]
        .into_iter()
        .find(|prefix| raw.starts_with(prefix))
    else {
        return false;
    };
    let argument = raw.get(prefix.len()..).unwrap_or_default();
    // Dynamic names have no declaration to show.
    !argument.is_empty()
        && !argument.starts_with('[')
        && ctx.offset >= attribute.name_start + prefix.len()
}

/// `@event` / `v-on:event` on a component tag.
pub(in crate::ide::hover) fn is_component_event_query(ctx: &IdeContext<'_>) -> bool {
    helpers::raw_attribute_with_source_span_at_offset(ctx).is_some_and(|attribute| {
        crate::ide::is_component_tag(&attribute.tag_name)
            && (attribute.raw_name.starts_with('@') || attribute.raw_name.starts_with("v-on:"))
    })
}

/// Declared slot type and JSDoc; `None` leaves the directive card to the caller.
pub(super) async fn slot_hover(
    ctx: &IdeContext<'_>,
    bridge: Option<&Arc<CorsaBridge>>,
) -> Option<Hover> {
    if !is_slot_name_query(ctx) {
        return None;
    }
    match canonical::hover(ctx, bridge, true).await? {
        Answer::Hover(hover) => Some(hover),
        Answer::Empty => None,
    }
}
