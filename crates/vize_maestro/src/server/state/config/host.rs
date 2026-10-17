//! Reviewed host runtime boundary; portable config models remain owned by L0.

pub(in crate::server::state) use vize_carton::config::LoadedLspConfig;
#[cfg(feature = "native")]
pub(in crate::server::state) use vize_carton::config::ProjectModel;
#[cfg(feature = "glyph")]
pub(in crate::server::state) use vize_carton::config::load_config_with_source;
pub(in crate::server::state) use vize_carton::config::load_lsp_config_snapshot;
