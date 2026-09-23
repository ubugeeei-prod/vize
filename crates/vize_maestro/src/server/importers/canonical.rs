//! Reuse Canon's resolved dependency graph instead of reparsing closed barrels.

use std::path::PathBuf;
use tower_lsp::lsp_types::Url;
use vize_s0::FxHashSet;

use super::{ServerState, comparable_path, remove_importer};

impl ServerState {
    pub(crate) fn record_typecheck_dependencies(
        &self,
        importer: &Url,
        revision: u64,
        paths: &[PathBuf],
    ) {
        // A closed/reopened or edited host must not regain an older graph after
        // the bridge await. Keep the revision guard through the index update.
        let Some(document) = self.documents.get(importer) else {
            return;
        };
        if document.revision() != revision {
            return;
        }
        let host = importer
            .to_file_path()
            .ok()
            .map(|path| comparable_path(&path));
        let dependencies = paths
            .iter()
            .map(|path| comparable_path(path))
            .filter(|path| Some(path) != host.as_ref())
            .collect::<FxHashSet<_>>();
        let mut index = self.open_imports.canonical.write();
        remove_importer(&mut index, importer);
        for dependency in &dependencies {
            index
                .by_dependency
                .entry(dependency.clone())
                .or_default()
                .insert(importer.clone());
        }
        if !dependencies.is_empty() {
            index
                .by_importer
                .insert(importer.clone(), dependencies.into_iter().collect());
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn resolved_graph_is_host_scoped_replaced_and_revision_guarded() {
        let directory = tempfile::tempdir().unwrap();
        let host = Url::from_file_path(directory.path().join("Host.vue")).unwrap();
        let other = Url::from_file_path(directory.path().join("Other.vue")).unwrap();
        let first = directory.path().join("first.ts");
        let second = directory.path().join("second.ts");
        let first_uri = Url::from_file_path(&first).unwrap();
        let second_uri = Url::from_file_path(&second).unwrap();
        let state = ServerState::new();
        for uri in [&host, &other] {
            state
                .documents
                .open(uri.clone(), "<template />".into(), 1, "vue".into());
        }
        let revision = state.documents.get(&host).unwrap().revision();
        state.record_typecheck_dependencies(&host, revision, std::slice::from_ref(&first));
        assert_eq!(state.open_importers(&first_uri), vec![host.clone()]);
        state.record_typecheck_dependencies(&host, revision, std::slice::from_ref(&second));
        assert!(state.open_importers(&first_uri).is_empty());
        assert_eq!(state.open_importers(&second_uri), vec![host.clone()]);

        // A reopen can reuse the client's version number, but not its revision.
        state
            .documents
            .open(host.clone(), "<template />".into(), 1, "vue".into());
        state.update_virtual_docs(&host, "<template />");
        state.record_typecheck_dependencies(&host, revision, &[first]);
        assert!(state.open_importers(&first_uri).is_empty());
        assert!(state.open_importers(&second_uri).is_empty());

        let revision = state.documents.get(&host).unwrap().revision();
        state.record_typecheck_dependencies(&host, revision, &[second]);
        state.close_document(&host);
        state.remove_virtual_docs(&host);
        state.record_typecheck_dependencies(&host, revision, &[]);
        assert!(state.open_importers(&second_uri).is_empty());
    }
}
