//! Authored URI spelling shared by native and import-alias navigation.

use crate::ide::IdeContext;
use std::{path::PathBuf, sync::Arc};
use tower_lsp::lsp_types::Url;

pub(crate) struct AuthoredUriResolver {
    open_uris: Arc<[Url]>,
    open_by_source_path: vize_l0::FxHashMap<PathBuf, Url>,
    open_by_physical_path: vize_l0::FxHashMap<PathBuf, Url>,
    workspace: Option<(PathBuf, PathBuf)>,
}

impl AuthoredUriResolver {
    pub(crate) fn new(ctx: &IdeContext<'_>) -> Self {
        let open_uris = ctx.state.documents.uris();
        let mut open_by_source_path = vize_l0::FxHashMap::default();
        let mut open_by_physical_path = vize_l0::FxHashMap::default();
        for uri in open_uris.iter() {
            let Ok(path) = uri.to_file_path() else {
                continue;
            };
            open_by_source_path.insert(path.clone(), uri.clone());
            open_by_physical_path.insert(ctx.state.canonical_editor_path(&path), uri.clone());
        }
        let workspace = ctx.state.get_workspace_root().map(|root| {
            let physical = ctx.state.canonical_editor_path(&root);
            (root, physical)
        });
        Self {
            open_uris,
            open_by_source_path,
            open_by_physical_path,
            workspace,
        }
    }

    pub(crate) fn resolve(
        &self,
        ctx: &IdeContext<'_>,
        source_path: &std::path::Path,
    ) -> Option<Url> {
        let source_uri = Url::from_file_path(source_path).ok();
        if let Some(uri) = &source_uri
            && let Ok(index) = self.open_uris.binary_search(uri)
        {
            return self.open_uris.get(index).cloned();
        }
        if let Some(uri) = self.open_by_source_path.get(source_path) {
            return Some(uri.clone());
        }
        let physical_path = ctx.state.canonical_editor_path(source_path);
        self.open_by_physical_path
            .get(&physical_path)
            .cloned()
            .or_else(|| {
                let (logical_root, physical_root) = self.workspace.as_ref()?;
                let relative = physical_path.strip_prefix(physical_root).ok()?;
                Url::from_file_path(logical_root.join(relative)).ok()
            })
            .or(source_uri)
    }
}

pub(crate) fn authored_file_uri(ctx: &IdeContext<'_>, path: &std::path::Path) -> Option<Url> {
    AuthoredUriResolver::new(ctx).resolve(ctx, path)
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn closed_foreign_targets_keep_the_import_path_before_an_open_owner_takes_precedence() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        let workspace = root.join("workspace");
        std::fs::create_dir(&workspace).unwrap();
        let physical = root.join("foreign.ts");
        let authored = workspace.join("linked.ts");
        std::fs::write(&physical, "export const value = 1;").unwrap();
        std::os::unix::fs::symlink(&physical, &authored).unwrap();
        let state = crate::server::ServerState::new();
        state.set_workspace_root(workspace.clone());
        let request = Url::from_file_path(workspace.join("App.vue")).unwrap();
        let ctx = IdeContext::testing(&state, &request, 0, "<template />".into());
        assert_eq!(
            authored_file_uri(&ctx, &authored),
            Some(Url::from_file_path(&authored).unwrap())
        );
        let open = Url::from_file_path(&physical).unwrap();
        state.documents.open(
            open.clone(),
            "export const value = 2;".into(),
            1,
            "typescript".into(),
        );
        assert_eq!(authored_file_uri(&ctx, &authored), Some(open.clone()));
        let linked = Url::from_file_path(&authored).unwrap();
        state.documents.open(
            linked.clone(),
            "export const value = 3;".into(),
            1,
            "typescript".into(),
        );
        assert_eq!(authored_file_uri(&ctx, &physical), Some(open.clone()));
        assert_eq!(authored_file_uri(&ctx, &authored), Some(linked.clone()));
        state.documents.close(&open);
        let encoded = Url::parse(&open.as_str().replace("foreign.ts", "fore%69gn.ts")).unwrap();
        assert_eq!(encoded.to_file_path().unwrap(), physical);
        state.documents.open(
            encoded.clone(),
            "export const value = 4;".into(),
            1,
            "typescript".into(),
        );
        assert_eq!(authored_file_uri(&ctx, &physical), Some(encoded));
        assert_eq!(authored_file_uri(&ctx, &authored), Some(linked));
        let unicode = root.join("é.ts");
        std::fs::write(&unicode, "export const value = 5;").unwrap();
        let canonical = Url::from_file_path(&unicode).unwrap();
        let alternate = Url::parse(&canonical.as_str().replace("%C3%A9", "%c3%a9")).unwrap();
        assert_ne!(alternate, canonical);
        assert_eq!(alternate.to_file_path().unwrap(), unicode);
        state
            .documents
            .open(canonical.clone(), "".into(), 1, "typescript".into());
        state
            .documents
            .open(alternate, "".into(), 1, "typescript".into());
        assert_eq!(authored_file_uri(&ctx, &unicode), Some(canonical));
    }
}
