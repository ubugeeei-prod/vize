use super::CorsaProjectClient;
use crate::{file_uri::file_uri_to_path, lsp_client::virtual_overlay};

pub(super) fn ensure_materialized_project<'a>(
    client: &mut CorsaProjectClient,
    uris: impl IntoIterator<Item = &'a str>,
) -> Result<(), String> {
    if client.materialized_project_session {
        return Ok(());
    }

    let requested_virtual_overlay = uris
        .into_iter()
        .any(|uri| needs_materialized_project_config(client, uri));
    let open_virtual_overlay = !requested_virtual_overlay
        && client
            .document_texts
            .keys()
            .any(|uri| open_uri_needs_materialized_project_config(uri));
    if requested_virtual_overlay || open_virtual_overlay {
        client.activate_materialized_project_session()?;
    }
    Ok(())
}

pub(super) fn needs_materialized_project_config(client: &CorsaProjectClient, uri: &str) -> bool {
    client.document_texts.contains_key(uri) && open_uri_needs_materialized_project_config(uri)
}

fn open_uri_needs_materialized_project_config(uri: &str) -> bool {
    file_uri_to_path(uri)
        .is_some_and(|path| !path.exists() && virtual_overlay::target_exists(&path))
}
