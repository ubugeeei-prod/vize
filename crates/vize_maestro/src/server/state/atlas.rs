//! Persistent Atlas ownership for open editor documents.

use tower_lsp::lsp_types::Url;
use vize_atlas::{Compilation, Product, Shared, SourceId};

use super::ServerState;

impl ServerState {
    pub(super) fn new_artifact_compilation() -> Compilation {
        let mut compilation = Compilation::new();
        vize_atelier_sfc::register_atlas_providers(&mut compilation)
            .expect("Maestro must register the SFC artifact providers once");
        vize_atelier_jsx::register_atlas_providers(&mut compilation)
            .expect("Maestro must register the JSX artifact providers once");
        vize_canon::register_sfc_typecheck_provider(&mut compilation)
            .expect("Maestro must register Canon's SFC typecheck provider once");
        compilation
    }

    pub(super) fn refresh_artifact_croquis_mode(&self) {
        let mode = self.artifact_croquis_mode();
        let mut compilation = self.artifact_compilation.write();
        for source in self.artifact_sources.iter().map(|entry| *entry.value()) {
            if let Err(error) = compilation
                .set_source_input::<vize_atelier_sfc::SfcCroquisSettingsInput>(
                    source,
                    croquis_request(mode),
                )
            {
                tracing::warn!(%source, %error, "failed to update Atlas Croquis compatibility mode");
            }
        }
    }

    fn artifact_croquis_mode(&self) -> vize_atelier_sfc::SfcCroquisMode {
        if self.legacy_vue2_enabled() {
            vize_atelier_sfc::SfcCroquisMode::LegacyVue2
        } else if self.options_api_enabled() {
            vize_atelier_sfc::SfcCroquisMode::OptionsApi
        } else {
            vize_atelier_sfc::SfcCroquisMode::Full
        }
    }

    /// Add or revise one open document while preserving its source identity.
    pub(crate) fn upsert_artifact_source(&self, uri: &Url, content: &str) -> Option<SourceId> {
        let mut compilation = self.artifact_compilation.write();
        if let Some(source) = self.artifact_sources.get(uri).map(|entry| *entry) {
            if let Err(error) = compilation.update_source(source, content) {
                tracing::warn!(%uri, %error, "failed to update Atlas editor source");
                return None;
            }
            return Some(source);
        }

        match compilation.add_source(uri.path(), content) {
            Ok(source) => {
                let mode = self.artifact_croquis_mode();
                if let Err(error) = compilation
                    .set_source_input::<vize_atelier_sfc::SfcCroquisSettingsInput>(
                        source,
                        croquis_request(mode),
                    )
                {
                    tracing::warn!(%uri, %error, "failed to install Atlas editor semantics");
                    let _ = compilation.remove_source(source);
                    return None;
                }
                self.artifact_sources.insert(uri.clone(), source);
                Some(source)
            }
            Err(error) => {
                tracing::warn!(%uri, %error, "failed to add Atlas editor source");
                None
            }
        }
    }

    /// Ensure detached test/tool contexts join the same persistent graph.
    pub(crate) fn ensure_artifact_source(&self, uri: &Url, content: &str) -> Option<SourceId> {
        self.artifact_sources
            .get(uri)
            .map(|entry| *entry)
            .or_else(|| self.upsert_artifact_source(uri, content))
    }

    /// Remove an editor document and all memoized products derived from it.
    pub(crate) fn remove_artifact_source(&self, uri: &Url) {
        let Some((_, source)) = self.artifact_sources.remove(uri) else {
            return;
        };
        if let Err(error) = self.artifact_compilation.write().remove_source(source) {
            tracing::warn!(%uri, %error, "failed to remove Atlas editor source");
        }
    }

    /// Move an open document without losing its stable Atlas identity.
    pub(crate) fn rename_artifact_source(&self, old_uri: &Url, new_uri: &Url) {
        let Some((_, source)) = self.artifact_sources.remove(old_uri) else {
            return;
        };
        if let Err(error) = self
            .artifact_compilation
            .write()
            .rename_source(source, new_uri.path())
        {
            tracing::warn!(%old_uri, %new_uri, %error, "failed to rename Atlas editor source");
            self.artifact_sources.insert(old_uri.clone(), source);
            return;
        }
        self.artifact_sources.insert(new_uri.clone(), source);
    }

    /// Query a product from the persistent graph and clone only its shared handle.
    fn artifact<P: Product>(&self, uri: &Url) -> Option<Shared<P::Value>> {
        let source = self.artifact_sources.get(uri).map(|entry| *entry)?;
        match self.artifact_compilation.write().query::<P>(source) {
            Ok(outcome) => Some(outcome.shared()),
            Err(error) => {
                tracing::warn!(%uri, product = P::NAME, %error, "Atlas editor query failed");
                None
            }
        }
    }

    pub(crate) fn jsx_syntax(
        &self,
        uri: &Url,
    ) -> Option<Shared<vize_atelier_jsx::JsxSyntaxSnapshot>> {
        self.artifact::<vize_atelier_jsx::JsxSyntaxProduct>(uri)
    }

    pub(crate) fn sfc_descriptor(
        &self,
        uri: &Url,
    ) -> Option<Shared<vize_atelier_sfc::SfcDescriptorArtifact>> {
        self.artifact::<vize_atelier_sfc::SfcDescriptorProduct>(uri)
    }

    /// Join a file-backed SFC to the persistent graph and return its cached descriptor.
    pub(crate) fn sfc_descriptor_for(
        &self,
        uri: &Url,
        content: &str,
    ) -> Option<Shared<vize_atelier_sfc::SfcDescriptorArtifact>> {
        self.ensure_artifact_source(uri, content)?;
        self.sfc_descriptor(uri)
    }

    pub(crate) fn sfc_croquis(&self, uri: &Url) -> Option<Shared<vize_croquis::CroquisDocument>> {
        self.artifact::<vize_croquis::CroquisDocumentProduct>(uri)
    }

    /// Query Canon diagnostics from the same persistent editor compilation.
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

fn croquis_request(mode: vize_atelier_sfc::SfcCroquisMode) -> vize_atelier_sfc::SfcCroquisRequest {
    vize_atelier_sfc::SfcCroquisRequest {
        mode,
        ..Default::default()
    }
}

#[cfg(test)]
#[path = "atlas/tests.rs"]
mod tests;
