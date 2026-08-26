//! Closed Vue files announced through workspace file-operation events.

use std::path::Path;

use futures::channel::oneshot;
use ignore::{DirEntry, WalkBuilder};
use tower_lsp::lsp_types::Url;
use vize_s0::FxHashMap;

use super::ServerState;
use super::global_components::is_excluded_directory;

impl ServerState {
    /// Discover and load the complete Vue workspace surface without blocking
    /// the LSP executor. File-operation events are merged because they may race
    /// the directory walk. Open buffers are applied last, so unsaved text wins
    /// over disk and newly-created buffers participate before their first save.
    pub(crate) async fn discover_workspace_vue_sources(&self) -> Vec<(Url, std::string::String)> {
        let mut sources = discover_sources_in_background(
            self.get_workspace_root(),
            self.workspace_vue_file_uris(),
        )
        .await
        .into_iter()
        .collect::<FxHashMap<_, _>>();
        for uri in self.documents.uris().into_iter().filter(is_vue_uri) {
            if let Some(source) = self.documents.text(&uri) {
                sources.insert(uri, source);
            }
        }
        let mut sources = sources.into_iter().collect::<Vec<_>>();
        sources.sort_by(|(left, _), (right, _)| left.as_str().cmp(right.as_str()));
        sources
    }

    /// Track newly-created on-disk Vue files without treating them as open
    /// editor documents. Folder events recursively discover nested SFCs.
    pub(crate) fn track_workspace_vue_files(&self, uri: &str) -> bool {
        let Ok(uri) = Url::parse(uri) else {
            return false;
        };
        let Ok(path) = uri.to_file_path() else {
            return false;
        };
        if path.is_file() {
            return is_vue_file(&path) && self.workspace_vue_files.insert(uri, ()).is_none();
        }
        if !path.is_dir() {
            return false;
        }

        let mut changed = false;
        for entry in vue_files_below(&path) {
            let Ok(uri) = Url::from_file_path(entry.path()) else {
                continue;
            };
            changed |= self.workspace_vue_files.insert(uri, ()).is_none();
        }
        changed
    }

    /// Forget a deleted or renamed on-disk Vue file or directory subtree.
    pub(crate) fn forget_workspace_vue_files(&self, uri: &str) -> bool {
        let Ok(uri) = Url::parse(uri) else {
            return false;
        };
        let Ok(prefix) = uri.to_file_path() else {
            return false;
        };
        let before = self.workspace_vue_files.len();
        self.workspace_vue_files.retain(|candidate, _| {
            candidate
                .to_file_path()
                .map_or(true, |path| !path.starts_with(&prefix))
        });
        self.workspace_vue_files.len() != before
    }

    /// Stable URI snapshot for workspace-wide searches. The caller performs
    /// filesystem reads after all DashMap guards have been released.
    pub(crate) fn workspace_vue_file_uris(&self) -> Vec<Url> {
        let mut uris = self
            .workspace_vue_files
            .iter()
            .map(|entry| entry.key().clone())
            .collect::<Vec<_>>();
        uris.sort_by(|left, right| left.as_str().cmp(right.as_str()));
        uris
    }
}

fn vue_files_below(root: &Path) -> impl Iterator<Item = DirEntry> {
    let mut builder = WalkBuilder::new(root);
    builder
        .hidden(false)
        .ignore(false)
        .git_global(false)
        .git_ignore(false)
        .git_exclude(false)
        .parents(false)
        .follow_links(false)
        .filter_entry(|entry| {
            !entry.file_type().is_some_and(|kind| kind.is_dir())
                || !is_excluded_directory(entry.file_name())
        });
    builder
        .build()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_some_and(|kind| kind.is_file()))
        .filter(|entry| is_vue_file(entry.path()))
}

fn is_vue_file(path: &Path) -> bool {
    path.extension().is_some_and(|extension| extension == "vue")
}

fn is_vue_uri(uri: &Url) -> bool {
    uri.path().ends_with(".vue")
}

async fn discover_sources_in_background(
    root: Option<std::path::PathBuf>,
    mut uris: Vec<Url>,
) -> Vec<(Url, std::string::String)> {
    let (sender, receiver) = oneshot::channel();
    let spawned = std::thread::Builder::new()
        .name("vize-workspace-vue-files".to_string())
        .spawn(move || {
            if let Some(root) = root {
                uris.extend(
                    vue_files_below(&root)
                        .filter_map(|entry| Url::from_file_path(entry.path()).ok()),
                );
            }
            uris.sort_by(|left, right| left.as_str().cmp(right.as_str()));
            uris.dedup();
            let sources = uris
                .into_iter()
                .filter_map(|uri| {
                    let path = uri.to_file_path().ok()?;
                    let source = std::fs::read_to_string(path).ok()?;
                    Some((uri, source))
                })
                .collect();
            let _ = sender.send(sources);
        });
    if let Err(error) = spawned {
        tracing::warn!("failed to spawn workspace Vue discovery: {error}");
        return Vec::new();
    }
    receiver.await.unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::{ServerState, Url};

    #[test]
    fn discovery_loads_closed_sources_and_prefers_open_buffers() {
        crate::runtime::block_on(async {
            let root = tempfile::tempdir().unwrap();
            let open_path = root.path().join("Open.vue");
            let closed_path = root.path().join("Closed.vue");
            std::fs::write(&open_path, "<template>stale disk</template>").unwrap();
            std::fs::write(&closed_path, "<template>closed disk</template>").unwrap();
            let open_uri = Url::from_file_path(open_path).unwrap();
            let closed_uri = Url::from_file_path(closed_path).unwrap();
            let state = ServerState::new();
            state.set_workspace_root(root.path().to_path_buf());
            state.documents.open(
                open_uri.clone(),
                "<template>unsaved buffer</template>".to_string(),
                1,
                "vue".to_string(),
            );

            let sources = state.discover_workspace_vue_sources().await;
            assert_eq!(
                sources,
                vec![
                    (closed_uri, "<template>closed disk</template>".to_string()),
                    (open_uri, "<template>unsaved buffer</template>".to_string(),),
                ],
            );
        });
    }

    #[test]
    fn directory_events_track_nested_vue_files_and_forget_only_that_subtree() {
        let root = tempfile::tempdir().unwrap();
        let components = root.path().join("components");
        let sibling_dir = root.path().join("components-old");
        let child = components.join("nested/Child.vue");
        let ignored = components.join("node_modules/pkg/Ignored.vue");
        let sibling = sibling_dir.join("Sibling.vue");
        std::fs::create_dir_all(child.parent().unwrap()).unwrap();
        std::fs::create_dir_all(ignored.parent().unwrap()).unwrap();
        std::fs::create_dir_all(&sibling_dir).unwrap();
        std::fs::write(&child, "<template />").unwrap();
        std::fs::write(&ignored, "<template />").unwrap();
        std::fs::write(&sibling, "<template />").unwrap();
        let components_uri = Url::from_file_path(&components).unwrap();
        let sibling_dir_uri = Url::from_file_path(&sibling_dir).unwrap();
        let child_uri = Url::from_file_path(&child).unwrap();
        let sibling_uri = Url::from_file_path(&sibling).unwrap();
        let state = ServerState::new();

        assert!(state.track_workspace_vue_files(components_uri.as_str()));
        assert!(state.track_workspace_vue_files(sibling_dir_uri.as_str()));
        assert_eq!(
            state.workspace_vue_file_uris(),
            vec![sibling_uri.clone(), child_uri]
        );

        std::fs::remove_dir_all(&components).unwrap();
        assert!(state.forget_workspace_vue_files(components_uri.as_str()));
        assert_eq!(state.workspace_vue_file_uris(), vec![sibling_uri]);
    }
}
