//! Reuse the canonical edit transform with ranges from the open document's borrowed File.
use super::{IdeContext, RewriteRanges, TextEdit};
use crate::ide::rename::{ComponentEventEditRefusal, RenameService};
use std::sync::Arc;
use vize_l1_to_l2::native_file::SetupComponentImports;

impl RenameService {
    /// Apply the canonical event-name casing and full authored name range to a
    /// batch of semantic rename edits for this open document. Each edit keeps its
    /// original order and unrelated edits pass through intact.
    pub fn rewrite_component_event_edits(
        ctx: &IdeContext<'_>,
        imports: &SetupComponentImports<'_, '_>,
        mut edits: Vec<TextEdit>,
        semantic_name: &str,
    ) -> Result<Vec<TextEdit>, ComponentEventEditRefusal> {
        let current = || {
            let open = ctx
                .state
                .documents
                .text_snapshot(ctx.uri)
                .ok_or(ComponentEventEditRefusal::Closed)?;
            if !Arc::ptr_eq(&open, &ctx.content) {
                return Err(ComponentEventEditRefusal::Superseded);
            }
            Ok(())
        };
        current()?;
        if !core::ptr::eq(ctx.content.as_str(), imports.source()) {
            return Err(ComponentEventEditRefusal::ForeignFile);
        }
        let mut events = Vec::new();
        for site in imports.sites() {
            site.visit_static_listeners(|event| {
                events.push(event.name_span.start as usize..event.name_span.end as usize)
            })
            .map_err(ComponentEventEditRefusal::Listener)?;
        }
        let ranges = RewriteRanges {
            events,
            model_usages: Vec::new(),
            model_declarations: Vec::new(),
        };
        for edit in &mut edits {
            ranges.rewrite(&ctx.content, edit, semantic_name);
        }
        current()?;
        Ok(edits)
    }
}

#[cfg(test)]
mod tests;
