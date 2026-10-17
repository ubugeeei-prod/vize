//! Reviewed host runtime boundary; portable config models remain owned by L0.

pub(in crate::server::state) use vize_carton::config::LoadedLspConfig;
#[cfg(feature = "native")]
pub(in crate::server::state) use vize_carton::config::ProjectModel;
#[cfg(feature = "glyph")]
pub(in crate::server::state) use vize_carton::config::load_config_with_source;
pub(in crate::server::state) use vize_carton::config::load_lsp_config_snapshot;

#[cfg(feature = "native")]
pub(in crate::server::state) use vize_carton::config::load_lsp_config_snapshot_observed;

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
) -> ObservedLspConfig {
    let mut before = vize_canon::ConfigurationObservations::default();
    let loaded = load_lsp_config_snapshot_observed(path, &mut |paths| {
        before.extend(&vize_canon::ConfigurationObservations::capture(paths));
    });
    ObservedLspConfig {
        loaded,
        before,
        needs_confirmation: false,
    }
}
