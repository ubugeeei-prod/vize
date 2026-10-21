//! Reviewed host runtime boundary; portable config models remain owned by L0.

#[cfg(feature = "native")]
pub(in crate::server::state) use vize_carton::config::ConfigLoadFailureIdentity;
pub(in crate::server::state) use vize_carton::config::LoadedLspConfig;
pub(in crate::server::state) use vize_carton::config::ProjectModel;
pub(in crate::server::state) use vize_carton::config::load_lsp_config_snapshot;
#[cfg(feature = "glyph")]
pub(in crate::server::state) use vize_carton::config::load_project_config_with_source;
#[cfg(feature = "glyph")]
pub(in crate::server::state) use vize_carton::config::matcher::CompilerWhitespacePlan;

#[cfg(feature = "native")]
pub(in crate::server::state) use vize_carton::config::ConfigEvaluationSession;

/// Raw policy and actual observations from before its one host evaluation.
#[cfg(feature = "native")]
#[derive(Clone)]
pub(in crate::server::state) struct ObservedLspConfig {
    pub loaded: LoadedLspConfig,
    pub before: vize_canon::ConfigurationObservations,
    pub needs_confirmation: bool,
}

#[cfg(feature = "native")]
pub(in crate::server::state) fn load_observed_lsp_config_snapshot(
    path: Option<&std::path::Path>,
    session: &ConfigEvaluationSession,
) -> ObservedLspConfig {
    let phase =
        vize_canon::corsa_bridge::EditorPreparationPhase::start("public_configuration_capture", 1);
    let mut before = vize_canon::ConfigurationObservations::default();
    let loaded = session.load_snapshot_observed(path, &mut |paths| {
        before.extend(&vize_canon::ConfigurationObservations::capture(paths));
    });
    phase.finish();
    ObservedLspConfig {
        loaded,
        before,
        needs_confirmation: false,
    }
}
