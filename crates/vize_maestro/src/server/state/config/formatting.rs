//! Resolve a config file's formatter options into native formatter options.

use super::host::{self, CompilerWhitespacePlan, FormatterLinterPlan};
use crate::server::{ServerState, format::FormatterContext};
use std::path::{Path, PathBuf};
use vize_l0::{
    String, ToCompactString,
    config::{ConfigDocument, FormatterProfile},
};

pub(super) fn formatter_config_dir(project_root: Option<&Path>, source_path: &Path) -> PathBuf {
    project_root
        .map(Path::to_path_buf)
        .or_else(|| source_path.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf()))
}

/// Options, validated sorting and scoped whitespace belong to one config read.
pub(in crate::server::state) struct FormatterSnapshot {
    options: vize_glyph::FormatOptions,
    sorting: Result<Option<vize_glyph::ImportSortOptions>, String>,
    whitespace: CompilerWhitespacePlan,
    lint: Option<FormatterLinterPlan>,
}

impl Default for FormatterSnapshot {
    fn default() -> Self {
        Self {
            options: vize_glyph::FormatOptions::default(),
            sorting: Ok(None),
            whitespace: CompilerWhitespacePlan::default(),
            lint: None,
        }
    }
}

impl ServerState {
    /// Keep all formatter projections together without rereading configuration.
    pub(in crate::server::state) fn install_project_formatter_snapshot(
        &self,
        document: &ConfigDocument,
        project_root: Option<&Path>,
        source_path: &Path,
        compiler_whitespace: Option<&'static str>,
    ) {
        let config_dir = formatter_config_dir(project_root, source_path);
        let whitespace = CompilerWhitespacePlan::new(
            compiler_whitespace,
            &document.compiler_entries(),
            &config_dir,
        );
        self.install_formatter_snapshot(document, &config_dir, whitespace);
    }

    pub(in crate::server::state) fn install_formatter_snapshot(
        &self,
        document: &ConfigDocument,
        config_dir: &Path,
        whitespace: CompilerWhitespacePlan,
    ) {
        let snapshot = FormatterSnapshot {
            options: format_options_from_config(document.formatter_profile()),
            sorting: vize_glyph::resolve_sort_imports(document.formatter_sort_imports())
                .map_err(|error| error.to_compact_string()),
            whitespace,
            lint: Some(FormatterLinterPlan::new(
                document.linter_plan_with_config_rule_options(),
                config_dir,
            )),
        };
        if let Err(error) = &snapshot.sorting {
            tracing::warn!("Invalid formatter import sorting: {error}");
        }
        *self.format_options.write() = snapshot;
    }

    /// Get a clone of the current format options.
    #[inline]
    pub fn get_format_options(&self) -> vize_glyph::FormatOptions {
        self.format_options.read().options.clone()
    }

    /// Preserve the existing options/whitespace inspection API.
    #[cfg(test)]
    pub(crate) fn get_formatter_context(&self) -> (vize_glyph::FormatOptions, bool) {
        let snapshot = self.format_options.read();
        (
            snapshot.options.clone(),
            snapshot.whitespace.root_preserves_text(),
        )
    }

    #[cfg(test)]
    pub(crate) fn get_formatter_context_for_path(
        &self,
        path: &Path,
    ) -> (vize_glyph::FormatOptions, bool) {
        let snapshot = self.format_options.read();
        (
            snapshot.options.clone(),
            snapshot.whitespace.preserves_text_for(path),
        )
    }

    /// Refuse unsupported sorting instead of silently applying default output.
    pub(crate) fn try_formatter_context_for_path(
        &self,
        path: &Path,
    ) -> Result<FormatterContext, String> {
        let snapshot = self.format_options.read();
        let sorting = snapshot.sorting.clone()?;
        if path
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("vue"))
            && let Some(resolved) = snapshot.lint.as_ref().and_then(|plan| plan.resolve(path))
        {
            vize_patina::Linter::validate_editor_directive_normalization(
                &resolved.config,
                &resolved.rule_options,
                snapshot.options.normalize_directive_shorthands,
            )
            .inspect_err(|error| {
                warn_formatter_refusal(path, error.as_str());
            })?;
        }
        Ok(FormatterContext {
            options: snapshot.options.clone(),
            sorting,
            preserve_template_whitespace: snapshot.whitespace.preserves_text_for(path),
        })
    }

    /// Load all formatting policies from the same shared project config.
    pub fn load_format_config(&self, dir: &Path) {
        let loaded = host::load_project_config_with_source(Some(dir));
        if let Some(source_path) = loaded.source_path.as_deref() {
            self.install_project_formatter_snapshot(
                &loaded.document,
                loaded.project_root.as_deref(),
                source_path,
                loaded.document.compiler_whitespace(),
            );
            tracing::info!("Loaded format config from {}", source_path.display());
        }
    }
}

/// Record the refusal on the current subscriber.
///
/// `warn!` reads the process-wide callsite interest and max level. A rebuild
/// from a thread whose dispatcher is `NoSubscriber` stores `Interest::never`
/// while `has_just_one` is set, so the handler still returns null and the
/// WARN event never reaches the thread-local subscriber.
fn warn_formatter_refusal(path: &Path, error: &str) {
    use tracing::callsite::Callsite as _;

    static __CALLSITE: tracing::callsite::DefaultCallsite = tracing::callsite2! {
        name: "formatter_refusal",
        kind: tracing::metadata::Kind::EVENT,
        target: module_path!(),
        level: tracing::Level::WARN,
        fields: message
    };
    let meta = __CALLSITE.metadata();
    tracing::dispatcher::get_default(|dispatch| {
        if !dispatch.enabled(meta) {
            return;
        }
        let record = |values: tracing::field::ValueSet<'_>| {
            dispatch.event(&tracing::event::Event::new(meta, &values));
        };
        record(tracing::valueset!(
            meta.fields(),
            message = format_args!("Refusing formatter request for {}: {error}", path.display())
        ));
    });
}

fn format_options_from_config(profile: &FormatterProfile) -> vize_glyph::FormatOptions {
    vize_glyph::FormatOptions::from_profile(profile).unwrap_or_else(|error| {
        tracing::warn!("Invalid formatter configuration: {error}");
        vize_glyph::FormatOptions::default()
    })
}
