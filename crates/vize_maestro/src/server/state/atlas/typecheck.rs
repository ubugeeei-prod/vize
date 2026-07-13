//! Persistent Canon typecheck queries for open SFC revisions.

use tower_lsp::lsp_types::Url;
use vize_atlas::Shared;

use super::ServerState;

impl ServerState {
    pub(crate) fn sfc_typecheck_for(
        &self,
        uri: &Url,
        content: &str,
        request: vize_canon::SfcTypeCheckRequest,
    ) -> Option<Shared<vize_canon::SfcTypeCheckResult>> {
        let source = self.ensure_artifact_source(uri, content)?;
        let mut compilation = self.artifact_compilation.write();
        let typecheck_changed = compilation
            .source_input::<vize_canon::SfcTypeCheckSettingsInput>(source)
            != Some(&request);
        let croquis_changed = compilation
            .source_input::<vize_atelier_sfc::SfcCroquisSettingsInput>(source)
            .is_none_or(|configured| {
                configured.mode != request.mode
                    || configured.resolved_filename.as_deref() != Some(uri.path())
            });
        if (typecheck_changed || croquis_changed)
            && let Err(error) =
                vize_canon::install_sfc_typecheck_request(&mut compilation, source, request)
        {
            tracing::warn!(%uri, %error, "failed to configure Atlas typecheck request");
            return None;
        }
        match compilation.query::<vize_canon::SfcTypeCheckProduct>(source) {
            Ok(outcome) => Some(outcome.shared()),
            Err(error) => {
                tracing::warn!(%uri, %error, "Atlas typecheck query failed");
                None
            }
        }
    }
}
