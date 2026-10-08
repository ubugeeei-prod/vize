//! Keep current diagnostic work when yielding invalidates or cancels a pass.

use tower_lsp::lsp_types::Url;

use super::MaestroServer;

pub(in crate::server) struct RetainedDiagnostics<'a> {
    server: &'a MaestroServer,
    uri: &'a Url,
    finished: bool,
}

impl<'a> RetainedDiagnostics<'a> {
    pub(in crate::server) fn new(server: &'a MaestroServer, uri: &'a Url) -> Self {
        Self {
            server,
            uri,
            finished: false,
        }
    }

    pub(in crate::server) fn finish(mut self) {
        self.finished = true;
    }
}

impl Drop for RetainedDiagnostics<'_> {
    fn drop(&mut self) {
        if !self.finished {
            self.server.retry_current_diagnostics(self.uri);
        }
    }
}

impl MaestroServer {
    pub(in crate::server) fn retry_current_diagnostics(&self, uri: &Url) -> bool {
        if !self.state.is_lsp_typecheck_enabled() || !self.state.lsp_features().has_diagnostics() {
            return false;
        }
        let Some(version) = self.state.documents.version(uri) else {
            return false;
        };
        self.initial_diagnostics
            .as_ref()
            .is_some_and(|scheduler| scheduler.retry(uri.clone(), version))
    }
}
