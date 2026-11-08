//! Revision-scoped unsaved buffers shared by editor requests.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use vize_carton::FxHashMap;

/// One immutable set of open editor buffers.
///
/// Callers can cheaply clone this handle for every language feature. Canonical
/// paths and content identities are computed once when the document-store
/// revision changes instead of once per request.
#[derive(Clone)]
pub struct CorsaOverlaySnapshot {
    revision: Arc<OverlayRevision>,
}

struct OverlayRevision {
    entries: Arc<[(PathBuf, Arc<str>)]>,
    canonical: FxHashMap<PathBuf, Arc<str>>,
    fingerprint: Arc<OverlayFingerprint>,
}

impl CorsaOverlaySnapshot {
    /// Capture one complete open-buffer revision without copying its texts.
    pub fn from_shared_entries(entries: Vec<(PathBuf, Arc<str>)>) -> Self {
        let entries: Arc<[(PathBuf, Arc<str>)]> = entries.into();
        let canonical = entries
            .iter()
            .map(|(path, content)| {
                (
                    vize_carton::path::canonicalize_non_verbatim(path),
                    Arc::clone(content),
                )
            })
            .collect::<FxHashMap<_, _>>();
        let fingerprint = Arc::new(OverlayFingerprint::capture_shared(&canonical));
        Self {
            revision: Arc::new(OverlayRevision {
                entries,
                canonical,
                fingerprint,
            }),
        }
    }

    /// Original path/text pairs retained for legacy bridge entrypoints.
    pub fn iter(&self) -> impl Iterator<Item = (&PathBuf, &Arc<str>)> {
        self.revision
            .entries
            .iter()
            .map(|(path, content)| (path, content))
    }

    pub fn len(&self) -> usize {
        self.revision.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.revision.entries.is_empty()
    }
}

impl Default for CorsaOverlaySnapshot {
    fn default() -> Self {
        Self::from_shared_entries(Vec::new())
    }
}

pub(crate) trait OverlayAccess: Send + Sync {
    fn get(&self, path: &Path) -> Option<&str>;
    fn len(&self) -> usize;
    fn iter(&self) -> Box<dyn Iterator<Item = (&PathBuf, &str)> + '_>;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn contains_key(&self, path: &Path) -> bool {
        self.get(path).is_some()
    }

    fn fingerprint(&self) -> Arc<OverlayFingerprint> {
        Arc::new(OverlayFingerprint::capture(self))
    }
}

impl OverlayAccess for FxHashMap<PathBuf, &str> {
    fn get(&self, path: &Path) -> Option<&str> {
        FxHashMap::get(self, path).copied()
    }

    fn len(&self) -> usize {
        FxHashMap::len(self)
    }

    fn iter(&self) -> Box<dyn Iterator<Item = (&PathBuf, &str)> + '_> {
        Box::new(FxHashMap::iter(self).map(|(path, content)| (path, *content)))
    }
}

impl OverlayAccess for CorsaOverlaySnapshot {
    fn get(&self, path: &Path) -> Option<&str> {
        self.revision.canonical.get(path).map(AsRef::as_ref)
    }

    fn len(&self) -> usize {
        self.revision.canonical.len()
    }

    fn iter(&self) -> Box<dyn Iterator<Item = (&PathBuf, &str)> + '_> {
        Box::new(
            self.revision
                .canonical
                .iter()
                .map(|(path, content)| (path, content.as_ref())),
        )
    }

    fn fingerprint(&self) -> Arc<OverlayFingerprint> {
        Arc::clone(&self.revision.fingerprint)
    }
}

pub(crate) struct OverlayFingerprint {
    hash: u64,
    inputs: FxHashMap<PathBuf, u64>,
}

impl OverlayFingerprint {
    fn capture_shared(overlays: &FxHashMap<PathBuf, Arc<str>>) -> Self {
        Self::capture_entries(
            overlays
                .iter()
                .map(|(path, content)| (path, content.as_ref())),
        )
    }

    fn capture(overlays: &(impl OverlayAccess + ?Sized)) -> Self {
        Self::capture_entries(overlays.iter())
    }

    fn capture_entries<'a>(entries: impl Iterator<Item = (&'a PathBuf, &'a str)>) -> Self {
        use std::hash::{Hash, Hasher};

        let mut entries = entries.collect::<Vec<_>>();
        entries.sort_by(|left, right| left.0.cmp(right.0));
        let mut hash = vize_l0::micro_default_hasher!("canon.context.overlays");
        let mut inputs = FxHashMap::default();
        for (path, text) in entries {
            path.hash(&mut hash);
            text.hash(&mut hash);
            inputs.insert(path.clone(), overlay_content_identity(text));
        }
        Self {
            hash: hash.finish(),
            inputs,
        }
    }

    pub(crate) fn hash(&self) -> u64 {
        self.hash
    }

    pub(crate) fn inputs(&self) -> &FxHashMap<PathBuf, u64> {
        &self.inputs
    }
}

pub(crate) fn overlay_content_identity(text: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut input = vize_l0::micro_default_hasher!("canon.context.overlay_input");
    text.hash(&mut input);
    input.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cloned_revision_reuses_canonical_paths_and_content_identities() {
        let snapshot = CorsaOverlaySnapshot::from_shared_entries(vec![
            (
                PathBuf::from("workspace/./App.vue"),
                Arc::from("<template />"),
            ),
            (
                PathBuf::from("workspace/dep.ts"),
                Arc::from("export const dep = 1"),
            ),
        ]);
        let clone = snapshot.clone();

        let first = OverlayAccess::fingerprint(&snapshot);
        let second = OverlayAccess::fingerprint(&clone);
        assert!(Arc::ptr_eq(&first, &second));
        assert_eq!(first.hash(), second.hash());
        assert_eq!(first.inputs(), second.inputs());
        assert_eq!(OverlayAccess::len(&snapshot), 2);
        assert_eq!(
            OverlayAccess::get(&snapshot, Path::new("workspace/App.vue")),
            Some("<template />")
        );
    }

    #[test]
    fn changed_revision_gets_a_distinct_content_identity() {
        let first = CorsaOverlaySnapshot::from_shared_entries(vec![(
            PathBuf::from("App.vue"),
            Arc::from("<template />"),
        )]);
        let second = CorsaOverlaySnapshot::from_shared_entries(vec![(
            PathBuf::from("App.vue"),
            Arc::from("<template><p /></template>"),
        )]);

        let first = OverlayAccess::fingerprint(&first);
        let second = OverlayAccess::fingerprint(&second);
        assert!(!Arc::ptr_eq(&first, &second));
        assert_ne!(first.hash(), second.hash());
        assert_ne!(first.inputs(), second.inputs());
    }
}
