//! Revision-bound lint diagnostics used by hover.

use std::sync::Arc;

use tower_lsp::lsp_types::{Diagnostic, Url};

use super::ServerState;
use crate::ide::DiagnosticService;

impl ServerState {
    pub(crate) fn cache_lint_hover_diagnostics(
        &self,
        uri: &Url,
        version: i32,
        diagnostics: &[Diagnostic],
    ) {
        if self.documents.version(uri) != Some(version) {
            return;
        }
        let lint = diagnostics
            .iter()
            .filter(|diagnostic| {
                diagnostic
                    .source
                    .as_deref()
                    .is_some_and(|source| matches!(source, "vize/lint" | "vize/musea"))
            })
            .cloned()
            .collect();
        self.lint_hover_cache
            .insert(uri.clone(), (version, Arc::new(lint)));
    }

    pub(crate) fn lint_hover_diagnostics(&self, uri: &Url) -> Arc<Vec<Diagnostic>> {
        let Some(version) = self.documents.version(uri) else {
            return Arc::new(Vec::new());
        };
        if let Some(cached) = self.lint_hover_cache.get(uri)
            && cached.value().0 == version
        {
            return Arc::clone(&cached.value().1);
        }
        let diagnostics = Arc::new(DiagnosticService::collect_lint_only(self, uri));
        if self.documents.version(uri) == Some(version) {
            self.lint_hover_cache
                .insert(uri.clone(), (version, Arc::clone(&diagnostics)));
        }
        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_follows_document_versions_and_close() {
        let state = ServerState::new();
        let uri = Url::parse("file:///workspace/App.vue").unwrap();
        state.documents.open(
            uri.clone(),
            "<template><div /></template>".into(),
            1,
            "vue".into(),
        );
        let first = state.lint_hover_diagnostics(&uri);
        let second = state.lint_hover_diagnostics(&uri);
        assert!(Arc::ptr_eq(&first, &second));

        state.documents.open(
            uri.clone(),
            "<template><span /></template>".into(),
            2,
            "vue".into(),
        );
        let changed = state.lint_hover_diagnostics(&uri);
        assert!(!Arc::ptr_eq(&first, &changed));
        state.close_document(&uri);
        assert!(!state.lint_hover_cache.contains_key(&uri));
    }
}
