//! Bounded exact-answer caches for repeated native navigation requests.

use parking_lot::RwLock;
use std::sync::atomic::Ordering;
use tower_lsp::lsp_types::{
    GotoDefinitionResponse, Location, Position, PrepareRenameResponse, Url, WorkspaceEdit,
};

use super::{CorsaRequestStamp, ServerState};

struct Entry<Q, R> {
    uri: Url,
    position: Position,
    qualifier: Q,
    stamp: CorsaRequestStamp,
    response: Option<R>,
}

struct ExactAnswer<Q, R> {
    entry: RwLock<Option<Entry<Q, R>>>,
}

impl<Q, R> Default for ExactAnswer<Q, R> {
    fn default() -> Self {
        Self {
            entry: RwLock::new(None),
        }
    }
}

impl<Q, R: Clone> ExactAnswer<Q, R> {
    fn get<T: ?Sized>(
        &self,
        uri: &Url,
        position: Position,
        qualifier: &T,
        stamp: CorsaRequestStamp,
    ) -> Option<Option<R>>
    where
        Q: PartialEq<T>,
    {
        let entry = self.entry.read();
        let entry = entry.as_ref()?;
        (entry.uri == *uri
            && entry.position == position
            && entry.qualifier.eq(qualifier)
            && entry.stamp == stamp)
            .then(|| entry.response.clone())
    }

    fn insert(
        &self,
        uri: &Url,
        position: Position,
        qualifier: Q,
        stamp: CorsaRequestStamp,
        response: Option<R>,
    ) {
        *self.entry.write() = Some(Entry {
            uri: uri.clone(),
            position,
            qualifier,
            stamp,
            response,
        });
    }

    fn remove(&self, uri: &Url) {
        let mut entry = self.entry.write();
        if entry.as_ref().is_some_and(|entry| entry.uri == *uri) {
            *entry = None;
        }
    }
}

/// Each request kind retains at most one response, matching editor locality
/// without growing with workspace size or cursor history.
#[derive(Default)]
pub(super) struct NativeNavigationAnswers {
    definition: ExactAnswer<(), GotoDefinitionResponse>,
    references: ExactAnswer<bool, Vec<Location>>,
    prepare_rename: ExactAnswer<(), PrepareRenameResponse>,
    rename: ExactAnswer<String, WorkspaceEdit>,
}

impl NativeNavigationAnswers {
    pub(super) fn remove(&self, uri: &Url) {
        self.definition.remove(uri);
        self.references.remove(uri);
        self.prepare_rename.remove(uri);
        self.rename.remove(uri);
    }
}

impl ServerState {
    fn can_reuse_native_answer(&self, stamp: CorsaRequestStamp) -> bool {
        self.corsa_disk_watcher_registered.load(Ordering::Acquire) && stamp.is_current(self)
    }

    pub(crate) fn cached_native_definition(
        &self,
        uri: &Url,
        position: Position,
        stamp: CorsaRequestStamp,
    ) -> Option<Option<GotoDefinitionResponse>> {
        self.can_reuse_native_answer(stamp)
            .then(|| {
                self.native_navigation_answers
                    .definition
                    .get(uri, position, &(), stamp)
            })
            .flatten()
    }

    pub(crate) fn cache_native_definition(
        &self,
        uri: &Url,
        position: Position,
        stamp: CorsaRequestStamp,
        response: Option<GotoDefinitionResponse>,
    ) {
        if self.can_reuse_native_answer(stamp) {
            self.native_navigation_answers
                .definition
                .insert(uri, position, (), stamp, response);
        }
    }

    pub(crate) fn cached_native_references(
        &self,
        uri: &Url,
        position: Position,
        include_declaration: bool,
        stamp: CorsaRequestStamp,
    ) -> Option<Option<Vec<Location>>> {
        self.can_reuse_native_answer(stamp)
            .then(|| {
                self.native_navigation_answers.references.get(
                    uri,
                    position,
                    &include_declaration,
                    stamp,
                )
            })
            .flatten()
    }

    pub(crate) fn cache_native_references(
        &self,
        uri: &Url,
        position: Position,
        include_declaration: bool,
        stamp: CorsaRequestStamp,
        response: Option<Vec<Location>>,
    ) {
        if self.can_reuse_native_answer(stamp) {
            self.native_navigation_answers.references.insert(
                uri,
                position,
                include_declaration,
                stamp,
                response,
            );
        }
    }

    pub(crate) fn cached_native_prepare_rename(
        &self,
        uri: &Url,
        position: Position,
        stamp: CorsaRequestStamp,
    ) -> Option<Option<PrepareRenameResponse>> {
        self.can_reuse_native_answer(stamp)
            .then(|| {
                self.native_navigation_answers
                    .prepare_rename
                    .get(uri, position, &(), stamp)
            })
            .flatten()
    }

    pub(crate) fn cache_native_prepare_rename(
        &self,
        uri: &Url,
        position: Position,
        stamp: CorsaRequestStamp,
        response: Option<PrepareRenameResponse>,
    ) {
        if self.can_reuse_native_answer(stamp) {
            self.native_navigation_answers.prepare_rename.insert(
                uri,
                position,
                (),
                stamp,
                response,
            );
        }
    }

    pub(crate) fn cached_native_rename(
        &self,
        uri: &Url,
        position: Position,
        new_name: &str,
        stamp: CorsaRequestStamp,
    ) -> Option<Option<WorkspaceEdit>> {
        self.can_reuse_native_answer(stamp)
            .then(|| {
                self.native_navigation_answers
                    .rename
                    .get(uri, position, new_name, stamp)
            })
            .flatten()
    }

    pub(crate) fn cache_native_rename(
        &self,
        uri: &Url,
        position: Position,
        new_name: String,
        stamp: CorsaRequestStamp,
        response: Option<WorkspaceEdit>,
    ) {
        if self.can_reuse_native_answer(stamp) {
            self.native_navigation_answers
                .rename
                .insert(uri, position, new_name, stamp, response);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_answer_is_bounded_and_keys_qualifiers_and_revisions() {
        let state = ServerState::new();
        let cache = ExactAnswer::<bool, Vec<Location>>::default();
        let uri = Url::parse("file:///workspace/App.vue").unwrap();
        let other = Url::parse("file:///workspace/Other.vue").unwrap();
        let position = Position::new(4, 8);
        state.documents.open(
            uri.clone(),
            "<template>{{ value }}</template>".into(),
            1,
            "vue".into(),
        );
        let stamp = state.corsa_request_stamp();

        assert_eq!(cache.get(&uri, position, &true, stamp), None);
        cache.insert(&uri, position, true, stamp, Some(Vec::new()));
        assert_eq!(
            cache.get(&uri, position, &true, stamp),
            Some(Some(Vec::new()))
        );
        assert_eq!(cache.get(&uri, position, &false, stamp), None);

        state
            .documents
            .open(other.clone(), "<template />".into(), 1, "vue".into());
        assert_eq!(
            cache.get(&uri, position, &true, state.corsa_request_stamp()),
            None
        );

        cache.insert(&other, position, false, stamp, None);
        assert_eq!(cache.get(&other, position, &false, stamp), Some(None));
        assert_eq!(cache.get(&uri, position, &true, stamp), None);

        let rename_cache = ExactAnswer::<String, Vec<Location>>::default();
        rename_cache.insert(&uri, position, "first".to_string(), stamp, Some(Vec::new()));
        assert_eq!(
            rename_cache.get(&uri, position, "first", stamp),
            Some(Some(Vec::new()))
        );
        assert_eq!(rename_cache.get(&uri, position, "second", stamp), None);
    }
}
