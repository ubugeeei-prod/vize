//! Persistent Glyph queries for open SFC revisions.

use tower_lsp::lsp_types::Url;
use vize_atlas::Shared;

use super::ServerState;

impl ServerState {
    /// Query Glyph on the same URI/revision as the rest of the editor graph.
    pub(crate) fn formatted_sfc_for(
        &self,
        uri: &Url,
        content: &str,
        options: &vize_glyph::FormatOptions,
    ) -> Option<Shared<vize_glyph::FormatResult>> {
        let source = self.ensure_artifact_source(uri, content)?;
        let mut compilation = self.artifact_compilation.write();
        if compilation.source_input::<vize_glyph::GlyphFormatSettingsInput>(source) != Some(options)
            && let Err(error) = compilation
                .set_source_input::<vize_glyph::GlyphFormatSettingsInput>(source, options.clone())
        {
            tracing::warn!(%uri, %error, "failed to configure Atlas formatter request");
            return None;
        }
        match compilation.query::<vize_glyph::GlyphFormatProduct>(source) {
            Ok(outcome) => Some(outcome.shared()),
            Err(error) => {
                tracing::warn!(%uri, %error, "Atlas formatter query failed");
                None
            }
        }
    }
}
