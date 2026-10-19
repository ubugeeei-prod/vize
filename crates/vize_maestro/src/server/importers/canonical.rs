//! Reuse Canon's resolved dependency graph instead of reparsing closed barrels.

use std::path::PathBuf;
use tower_lsp::lsp_types::Url;
use vize_l0::FxHashSet;

use super::{ImportIndexData, ServerState, comparable_path, remove_importer};
use crate::server::state::CorsaRequestStamp;

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
        replace_importer(&mut index, importer, dependencies.into_iter().collect());
    }

    /// A detached initial read may only replace its own current graph. The
    /// existing document guard excludes host edits/reopens; the existing index
    /// write guard excludes competing importer replacement through both checks.
    pub(crate) fn record_initial_typecheck_dependencies(
        &self,
        importer: &Url,
        revision: u64,
        paths: &[PathBuf],
        stamp: CorsaRequestStamp,
    ) -> bool {
        self.record_initial_dependencies_while_current(importer, revision, paths, || {
            stamp.is_current(self)
        })
    }

    fn record_initial_dependencies_while_current(
        &self,
        importer: &Url,
        revision: u64,
        paths: &[PathBuf],
        is_current: impl Fn() -> bool,
    ) -> bool {
        let Some(document) = self.documents.get(importer) else {
            return false;
        };
        if document.revision() != revision || !is_current() {
            return false;
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
        if !is_current() {
            return false;
        }
        let previous = index.by_importer.get(importer).cloned().unwrap_or_default();
        replace_importer(&mut index, importer, dependencies.into_iter().collect());
        // Configuration/environment mutation can proceed while this worker
        // owns the index. If it crossed replacement, undo only this importer's
        // own write while still excluding every competing index writer. A
        // later change begins after this valid commit's linearization point.
        if !is_current() {
            replace_importer(&mut index, importer, previous);
            return false;
        }
        true
    }
}

fn replace_importer(index: &mut ImportIndexData, importer: &Url, dependencies: Vec<PathBuf>) {
    remove_importer(index, importer);
    for dependency in &dependencies {
        index
            .by_dependency
            .entry(dependency.clone())
            .or_default()
            .insert(importer.clone());
    }
    if !dependencies.is_empty() {
        index.by_importer.insert(importer.clone(), dependencies);
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

    #[test]
    fn initial_graph_replacement_refuses_stale_stamp_and_restores_own_entry_on_crossing_change() {
        let root = tempfile::tempdir().unwrap();
        let host = Url::from_file_path(root.path().join("Host.vue")).unwrap();
        let other = Url::from_file_path(root.path().join("Other.vue")).unwrap();
        let before = root.path().join("before.ts");
        let after = root.path().join("after.ts");
        let unrelated = root.path().join("unrelated.ts");
        let state = ServerState::new();
        for uri in [&host, &other] {
            state
                .documents
                .open(uri.clone(), "<template />".into(), 1, "vue".into());
        }
        let revision = state.documents.get(&host).unwrap().revision();
        state.record_typecheck_dependencies(&host, revision, std::slice::from_ref(&before));
        state.record_typecheck_dependencies(
            &other,
            state.documents.get(&other).unwrap().revision(),
            std::slice::from_ref(&unrelated),
        );
        let stamp = state.corsa_request_stamp();
        // Editing/reopening a host can reuse its public version, never revision.
        state.documents.open(
            host.clone(),
            "<template>new</template>".into(),
            1,
            "vue".into(),
        );
        assert!(!state.record_initial_typecheck_dependencies(
            &host,
            revision,
            std::slice::from_ref(&after),
            stamp
        ));
        let revision = state.documents.get(&host).unwrap().revision();
        let calls = std::cell::Cell::new(0);
        assert!(!state.record_initial_dependencies_while_current(
            &host,
            revision,
            std::slice::from_ref(&after),
            || {
                calls.set(calls.get() + 1);
                calls.get() < 3 // Authority moves after insertion, before its postguard.
            }
        ));
        assert_eq!(calls.get(), 3);
        let index = state.open_imports.canonical.read();
        assert_eq!(
            index.by_importer.get(&host),
            Some(&vec![comparable_path(&before)])
        );
        assert_eq!(
            index.by_importer.get(&other),
            Some(&vec![comparable_path(&unrelated)])
        );
        assert_eq!(
            index.by_dependency.get(&comparable_path(&before)),
            Some(&[host.clone()].into_iter().collect())
        );
        assert!(!index.by_dependency.contains_key(&comparable_path(&after)));
        assert_eq!(
            index.by_dependency.get(&comparable_path(&unrelated)),
            Some(&[other].into_iter().collect())
        );
        drop(index);
        assert!(state.record_initial_typecheck_dependencies(
            &host,
            revision,
            std::slice::from_ref(&after),
            state.corsa_request_stamp()
        ));
        assert!(
            !state
                .open_imports
                .canonical
                .read()
                .by_dependency
                .contains_key(&comparable_path(&before))
        );
        assert_eq!(
            state.open_imports.canonical.read().by_importer.get(&host),
            Some(&vec![comparable_path(&after)])
        );
    }
}
