//! The registry owns navigation beside state; the state keeps only a weak owner.
use super::{Arc, ServerState, Weak};
use crate::source_project::navigation::NativeNavigationProject;

impl ServerState {
    pub(crate) fn cached_project_navigation(
        self: &Arc<Self>,
    ) -> Option<Arc<NativeNavigationProject<'static>>> {
        let owner = self
            .project_contexts
            .owner
            .read()
            .as_ref()
            .and_then(Weak::upgrade)?;
        let boundary = self
            .project_contexts
            .boundary
            .read()
            .clone()
            .or_else(|| self.get_workspace_root())?;
        let context = owner
            .project_contexts
            .contexts
            .lock()
            .get(&boundary)
            .cloned()?;
        context.navigation_for_state(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        runtime::block_on,
        source_project::{SnapshotRefusal, navigation::NavigationRefusal},
    };
    use tower_lsp::lsp_types::{Position, Url};

    fn fixture() -> (tempfile::TempDir, Arc<ServerState>, Url) {
        let root = tempfile::tempdir().unwrap();
        let package = root.path().join("package");
        std::fs::create_dir_all(&package).unwrap();
        std::fs::write(
            root.path().join("vite.config.mjs"),
            "export default { vue: {} };",
        )
        .unwrap();
        std::fs::write(package.join("package.json"), "{}").unwrap();
        std::fs::write(
            package.join("vite.config.mjs"),
            "export default { vue: {} };",
        )
        .unwrap();
        let owner = Arc::new(ServerState::new());
        owner.set_workspace_root(root.path().to_path_buf());
        owner.set_workspace_folders(vec![root.path().to_path_buf()]);
        owner.load_workspace_config(root.path());
        let uri = Url::from_file_path(package.join("local.ts")).unwrap();
        owner.documents.open(
            uri.clone(),
            "const value=1;value;".into(),
            1,
            "typescript".into(),
        );
        (root, owner, uri)
    }

    #[test]
    fn context_navigation_admits_only_its_exact_initialized_state() {
        let (_fixture, owner, uri) = fixture();
        let child = owner.document_project_state(&uri).unwrap();
        let context = owner.document_project_context(&uri).unwrap();
        let navigation = context.navigation_for_state(&child).unwrap();
        assert!(Arc::ptr_eq(
            &navigation,
            &child.cached_project_navigation().unwrap()
        ));
        assert!(context.navigation_for_state(&owner).is_none());
        assert!(
            context
                .navigation_for_state(&Arc::new(ServerState::new()))
                .is_none()
        );
        context.mark_retired();
        assert!(context.navigation_for_state(&child).is_none());
        assert!(child.cached_project_navigation().is_none());
        assert_eq!(
            block_on(navigation.definition(&uri, Position::new(0, 15))),
            Err(NavigationRefusal::Host(SnapshotRefusal::Superseded))
        );
        context.retire();
    }

    #[test]
    fn cached_navigation_and_in_flight_arcs_do_not_keep_the_root_or_child_alive() {
        let (_fixture, owner, uri) = fixture();
        let child = owner.document_project_state(&uri).unwrap();
        let navigation = child.cached_project_navigation().unwrap();
        assert!(
            block_on(navigation.definition(&uri, Position::new(0, 15)))
                .unwrap()
                .is_some()
        );
        let root_weak = Arc::downgrade(&owner);
        let child_weak = Arc::downgrade(&child);
        drop(owner);
        assert!(root_weak.upgrade().is_none());
        assert!(child.cached_project_navigation().is_none());
        assert_eq!(
            block_on(navigation.definition(&uri, Position::new(0, 15))),
            Err(NavigationRefusal::Host(SnapshotRefusal::Superseded))
        );
        assert!(navigation.cached_program_worker_for_test(&uri).is_none());
        drop(child);
        assert!(child_weak.upgrade().is_some());
        drop(navigation);
        assert!(child_weak.upgrade().is_none());
    }
}
