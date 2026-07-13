//! Persistent Atlas ownership for open editor documents.

#[cfg(feature = "native")]
#[path = "atlas/canon.rs"]
mod canon;
#[cfg(feature = "glyph")]
#[path = "atlas/format.rs"]
mod format;
#[path = "atlas/script.rs"]
mod script;
#[path = "atlas/typecheck.rs"]
mod typecheck;

use std::sync::atomic::Ordering;

use tower_lsp::lsp_types::Url;
use vize_atlas::{Compilation, Product, Shared, SourceId};

use super::ServerState;

impl ServerState {
    pub(super) fn new_artifact_compilation() -> Compilation {
        let mut compilation = Compilation::new();
        vize_atelier_sfc::register_atlas_providers(&mut compilation)
            .expect("Maestro must register the SFC artifact providers once");
        #[cfg(feature = "glyph")]
        vize_glyph::register_glyph_format_provider(&mut compilation)
            .expect("Maestro must register Glyph's formatter root once");
        vize_atelier_jsx::register_atlas_providers(&mut compilation)
            .expect("Maestro must register the JSX artifact providers once");
        compilation
            .register_provider(vize_atelier_template::RawTemplateReliefProvider)
            .expect("Maestro must register the raw-template Relief provider once");
        compilation
            .register_provider(vize_atelier_template::RawTemplateCroquisProvider)
            .expect("Maestro must register the raw-template Croquis provider once");
        vize_canon::register_sfc_typecheck_provider(&mut compilation)
            .expect("Maestro must register Canon's SFC typecheck provider once");
        #[cfg(feature = "native")]
        vize_canon::batch::register_canon_vue_document_provider(&mut compilation)
            .expect("Maestro must register Canon's Vue document root once");
        let linter = Shared::new(vize_patina::Linter::new());
        vize_patina::register_shared_document_lint_recipe(&mut compilation, Shared::clone(&linter))
            .expect("Maestro must register Patina's document root once");
        vize_patina::register_shared_template_lint_recipe(&mut compilation, linter)
            .expect("Maestro must register Patina's raw-template root once");
        crate::virtual_code::register_virtual_documents_provider(&mut compilation)
            .expect("Maestro must register its virtual-document root once");
        compilation
    }

    pub(super) fn refresh_artifact_croquis_mode(&self) {
        let mode = self.artifact_croquis_mode();
        let mut compilation = self.artifact_compilation.write();
        for source in self.artifact_sources.iter().map(|entry| *entry.value()) {
            if !compilation
                .source(source)
                .is_some_and(|snapshot| snapshot.name().ends_with(".vue"))
            {
                continue;
            }
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
            if compilation
                .source(source)
                .is_some_and(|snapshot| snapshot.text() == content)
            {
                return Some(source);
            }
            if let Err(error) = compilation.update_source(source, content) {
                tracing::warn!(%uri, %error, "failed to update Atlas editor source");
                return None;
            }
            return Some(source);
        }

        match compilation.add_source(uri.path(), content) {
            Ok(source) => {
                if let Err(error) =
                    self.install_artifact_source_inputs(&mut compilation, source, uri)
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
        self.upsert_artifact_source(uri, content)
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

    #[cfg(test)]
    pub(crate) fn artifact_product_executions<P: Product>(&self) -> u64 {
        self.artifact_compilation
            .read()
            .counters()
            .for_product::<P>()
            .executions()
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

    pub(crate) fn raw_template_croquis(
        &self,
        uri: &Url,
    ) -> Option<Shared<vize_croquis::CroquisDocument>> {
        self.artifact::<vize_croquis::CroquisDocumentProduct>(uri)
    }

    pub(crate) fn sfc_relief(
        &self,
        uri: &Url,
    ) -> Option<Shared<Option<vize_relief::ReliefArtifact>>> {
        self.artifact::<vize_relief::ReliefProduct>(uri)
    }

    pub(crate) fn raw_template_relief(
        &self,
        uri: &Url,
    ) -> Option<Shared<Option<vize_relief::ReliefArtifact>>> {
        self.artifact::<vize_relief::ReliefProduct>(uri)
    }

    pub(crate) fn virtual_documents(
        &self,
        uri: &Url,
    ) -> Option<Shared<crate::virtual_code::VirtualDocuments>> {
        self.artifact::<crate::virtual_code::VirtualDocumentsProduct>(uri)
    }

    /// Query Patina from the persistent graph with the latest workspace config.
    pub(crate) fn lint_report_for(
        &self,
        uri: &Url,
        content: &str,
        ecosystem_enabled: bool,
    ) -> Option<Shared<vize_patina::LintResult>> {
        let source = self.ensure_artifact_source(uri, content)?;
        let generation = self.linter_generation.load(Ordering::SeqCst);
        let mut compilation = self.artifact_compilation.write();
        if self.artifact_linter_generation.load(Ordering::SeqCst) != generation {
            let linter = Shared::new(self.configured_linter(ecosystem_enabled));
            if let Err(error) = vize_patina::install_document_linter(&mut compilation, linter) {
                tracing::warn!(%uri, %error, "failed to configure Atlas lint root");
                return None;
            }
            self.artifact_linter_generation
                .store(generation, Ordering::SeqCst);
        }
        match compilation.query::<vize_patina::PatinaDocumentReportProduct>(source) {
            Ok(outcome) => Some(outcome.shared()),
            Err(error) => {
                tracing::warn!(%uri, %error, "Atlas lint query failed");
                None
            }
        }
    }

    fn configured_linter(&self, ecosystem_enabled: bool) -> vize_patina::Linter {
        use vize_patina::LintPreset;

        let config = self.get_linter_config();
        let options = self.get_linter_rule_options();
        let preset = config
            .preset
            .as_deref()
            .and_then(LintPreset::parse)
            .unwrap_or_default();
        let linter = if ecosystem_enabled && config.preset.is_none() {
            vize_patina::Linter::with_ecosystem()
        } else {
            vize_patina::Linter::with_preset(preset)
        }
        .with_additional_rules(config.enabled_rules())
        .with_disabled_rules(config.disabled_rules())
        .with_restricted_globals(options.restricted_globals())
        .with_restricted_members(options.restricted_members());

        #[cfg(not(target_arch = "wasm32"))]
        let linter = if config.strict_reactivity_enabled() {
            linter.with_rule(Box::new(
                vize_patina::rules::type_aware::NoReactivityLoss::new(),
            ))
        } else {
            linter
        };
        linter
    }
}

impl ServerState {
    fn install_artifact_source_inputs(
        &self,
        compilation: &mut Compilation,
        source: SourceId,
        uri: &Url,
    ) -> Result<(), vize_atlas::CompilationInputError> {
        if crate::utils::is_standalone_html_path(uri.path()) {
            vize_atelier_template::install_template_compile_request(
                compilation,
                source,
                vize_atelier_template::TemplateCompileRequest::default(),
            )?;
            vize_atelier_template::install_template_parse_mode(
                compilation,
                source,
                vize_atelier_template::TemplateParseMode::Document,
            )?;
            vize_patina::install_template_lint_request(
                compilation,
                source,
                vize_patina::PatinaTemplateLintRequest::standalone_html(uri.path()),
            )?;
            return Ok(());
        }
        compilation
            .set_source_input::<vize_atelier_sfc::SfcCroquisSettingsInput>(
                source,
                croquis_request(self.artifact_croquis_mode()),
            )
            .map(|_| ())
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
