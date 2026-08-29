use super::CorsaProjectClient;
use crate::{file_uri::file_uri_to_path, lsp_client::virtual_overlay};

pub(super) fn ensure_materialized_project<'a>(
    client: &mut CorsaProjectClient,
    uris: impl IntoIterator<Item = &'a str>,
) -> Result<(), String> {
    if client.has_project_session()
        && !client.materialized_project_session
        && uris
            .into_iter()
            .any(|uri| needs_materialized_project_config(client, uri))
    {
        client.activate_materialized_project_session()?;
    }
    Ok(())
}

pub(super) fn needs_materialized_project_config(client: &CorsaProjectClient, uri: &str) -> bool {
    client.document_texts.contains_key(uri)
        && file_uri_to_path(uri).is_some_and(|path| {
            path.starts_with(&client.project_root)
                && !path.exists()
                && virtual_overlay::target_exists(&path)
        })
}
