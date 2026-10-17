//! Attach the real native project during grace; diagnostic queries stay deferred.

use tower_lsp::lsp_types::Url;

use super::MaestroServer;
use crate::ide::{IdeContext, prepare_canonical_virtual_document};

impl MaestroServer {
    pub(super) async fn prepare_initial_native_document(&self, uri: &Url, version: i32) {
        if !self.state.is_lsp_typecheck_enabled()
            || self.state.project_context_retired()
            || self.state.documents.version(uri) != Some(version)
            || !uri.path().ends_with(".vue")
            || uri.path().ends_with(".art.vue")
        {
            return;
        }
        let Some(scope) = self.state.try_corsa_request_scope() else {
            return;
        };
        if !scope.is_current() {
            return;
        }
        let Some(ctx) = IdeContext::new(&self.state, uri, 0) else {
            return;
        };
        tracing::info!("preparing initial native project for {uri} version {version}");
        let Some(bridge) = self.state.get_corsa_bridge().await else {
            return;
        };
        if !scope.is_current() || self.state.documents.version(uri) != Some(version) {
            return;
        }
        // Both configured-project attestations and normal mirror ownership run
        // unchanged. This opens no diagnostic query and publishes no result.
        match prepare_canonical_virtual_document(&ctx, &bridge).await {
            Ok(true) => {}
            Ok(false) => return,
            Err(error) => {
                tracing::debug!("initial native preparation deferred after {error}");
                return;
            }
        }
        if scope.is_current() && self.state.documents.version(uri) == Some(version) {
            tracing::info!("prepared initial native project for {uri} version {version}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower_lsp::LspService;

    #[test]
    fn preparation_refuses_stale_closed_disabled_retired_and_busy_projects() {
        for control in ["stale", "closed", "disabled", "retired", "busy"] {
            let root = tempfile::tempdir().unwrap();
            let uri = Url::from_file_path(root.path().join("App.vue")).unwrap();
            let (service, _socket) = LspService::new(MaestroServer::new);
            let server = service.inner();
            server.state.set_workspace_root(root.path().to_path_buf());
            server
                .state
                .documents
                .open(uri.clone(), "<template />".into(), 2, "vue".into());
            match control {
                "closed" => {
                    server.state.documents.close(&uri);
                }
                "disabled" => server.state.apply_lsp_initialization_options(Some(
                    &serde_json::json!({"typecheck":false}),
                )),
                "retired" => server.state.retire_project_contexts(),
                _ => {}
            }
            crate::runtime::block_on(async {
                let _active = if control == "busy" {
                    Some(server.state.corsa_request_scope().await)
                } else {
                    None
                };
                server
                    .prepare_initial_native_document(&uri, if control == "stale" { 1 } else { 2 })
                    .await;
            });
            assert!(
                !server.state.has_corsa_bridge(),
                "{control} must not start a backend"
            );
            assert!(
                server.state.corsa_init_failure().is_none(),
                "{control} must not attempt a backend startup"
            );
        }
    }
}
