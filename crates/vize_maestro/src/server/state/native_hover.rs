//! One-entry typed-hover cache bound to the complete native request revision.

use parking_lot::RwLock;
use std::sync::atomic::Ordering;
use tower_lsp::lsp_types::{Hover, Position, Url};

use super::{CorsaRequestStamp, ServerState};

struct Entry {
    uri: Url,
    position: Position,
    stamp: CorsaRequestStamp,
    response: Option<Hover>,
}

/// Pointer motion repeatedly requests the same token. One entry captures that
/// locality without retaining hover results for every position in a workspace.
#[derive(Default)]
pub(super) struct NativeHoverCache {
    entry: RwLock<Option<Entry>>,
}

impl NativeHoverCache {
    fn get(
        &self,
        uri: &Url,
        position: Position,
        stamp: CorsaRequestStamp,
    ) -> Option<Option<Hover>> {
        let entry = self.entry.read();
        let entry = entry.as_ref()?;
        (entry.uri == *uri && entry.position == position && entry.stamp == stamp)
            .then(|| entry.response.clone())
    }

    fn insert(
        &self,
        uri: &Url,
        position: Position,
        stamp: CorsaRequestStamp,
        response: Option<Hover>,
    ) {
        *self.entry.write() = Some(Entry {
            uri: uri.clone(),
            position,
            stamp,
            response,
        });
    }

    pub(super) fn remove(&self, uri: &Url) {
        let mut entry = self.entry.write();
        if entry.as_ref().is_some_and(|entry| entry.uri == *uri) {
            *entry = None;
        }
    }
}

impl ServerState {
    pub(crate) fn cached_native_hover(
        &self,
        uri: &Url,
        position: Position,
        stamp: CorsaRequestStamp,
    ) -> Option<Option<Hover>> {
        // Without a successful dependency watcher registration, Canon must
        // revalidate disk inputs on every request; bypassing it could retain a
        // hover after an unobserved dependency edit.
        if !self.corsa_disk_watcher_registered.load(Ordering::Acquire) || !stamp.is_current(self) {
            return None;
        }
        self.native_hover.get(uri, position, stamp)
    }

    pub(crate) fn cache_native_hover(
        &self,
        uri: &Url,
        position: Position,
        stamp: CorsaRequestStamp,
        response: Option<Hover>,
    ) {
        if self.corsa_disk_watcher_registered.load(Ordering::Acquire) && stamp.is_current(self) {
            self.native_hover.insert(uri, position, stamp, response);
        }
    }
}

#[cfg(test)]
mod tests {
    use tower_lsp::lsp_types::{HoverContents, MarkedString};

    use super::*;

    fn response(value: &str) -> Option<Hover> {
        Some(Hover {
            contents: HoverContents::Scalar(MarkedString::String(value.into())),
            range: None,
        })
    }

    #[test]
    fn cache_is_exact_bounded_and_distinguishes_an_empty_answer() {
        let state = ServerState::new();
        let cache = NativeHoverCache::default();
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

        assert_eq!(cache.get(&uri, position, stamp), None);
        cache.insert(&uri, position, stamp, response("first"));
        assert_eq!(cache.get(&uri, position, stamp), Some(response("first")));
        assert_eq!(cache.get(&uri, Position::new(4, 9), stamp), None);

        state
            .documents
            .open(other.clone(), "<template />".into(), 1, "vue".into());
        let next = state.corsa_request_stamp();
        assert_eq!(cache.get(&uri, position, next), None);

        cache.insert(&other, position, stamp, None);
        assert_eq!(cache.get(&other, position, stamp), Some(None));
        assert_eq!(cache.get(&uri, position, stamp), None);
    }
}
