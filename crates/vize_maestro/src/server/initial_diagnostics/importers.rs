//! Refresh dependent open documents after a complete native pass.

use super::MaestroServer;
use tower_lsp::lsp_types::Url;

pub(super) async fn refresh_initial_importers(
    project: &MaestroServer,
    uri: &Url,
    version: i32,
) -> Vec<Url> {
    // didOpen/didChange or the event that retained this pass prepared its
    // current owner already. Only other importer owners still need marking.
    project
        .publish_importer_diagnostics_with_current_owner(uri, Some(version), true)
        .await
}
