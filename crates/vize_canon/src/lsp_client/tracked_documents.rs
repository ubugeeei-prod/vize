//! Mutation-owned editor deltas; read-only map access cannot bypass tracking.

use std::ops::Deref;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Serialize, Serializer};
use sha2::{Digest, Sha256};
use vize_l0::{FxHashMap, FxHashSet, String};

static NEXT_OWNER: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct DocumentCheckpoint {
    owner: u64,
    revision: u64,
}

#[derive(Debug)]
pub(super) struct TrackedDocumentTexts {
    documents: FxHashMap<String, String>,
    desired_digests: FxHashMap<String, [u8; 32]>,
    #[cfg(test)]
    digest_computations: usize,
    owner: u64,
    revision: u64,
    baseline: u64,
    baseline_len: usize,
    /// None means that only a complete map reconciliation is authoritative.
    pending: Option<FxHashSet<String>>,
}

impl Default for TrackedDocumentTexts {
    fn default() -> Self {
        Self::from(FxHashMap::default())
    }
}

impl From<FxHashMap<String, String>> for TrackedDocumentTexts {
    fn from(documents: FxHashMap<String, String>) -> Self {
        Self {
            documents,
            desired_digests: FxHashMap::default(),
            #[cfg(test)]
            digest_computations: 0,
            owner: NEXT_OWNER.fetch_add(1, Ordering::Relaxed),
            revision: 0,
            baseline: 0,
            baseline_len: 0,
            pending: None,
        }
    }
}

impl TrackedDocumentTexts {
    pub(super) fn checkpoint(&self) -> DocumentCheckpoint {
        DocumentCheckpoint {
            owner: self.owner,
            revision: self.revision,
        }
    }

    pub(super) fn is_dirty(&self) -> bool {
        self.pending
            .as_ref()
            .is_none_or(|pending| !pending.is_empty())
    }

    pub(super) fn insert(&mut self, uri: String, text: String) -> Option<String> {
        let changed = self.documents.get(uri.as_str()) != Some(&text);
        if changed {
            self.desired_digests.remove(uri.as_str());
        }
        let previous = self.documents.insert(uri.clone(), text);
        if changed {
            self.record_change(uri);
        }
        previous
    }

    pub(super) fn remove(&mut self, uri: &str) -> Option<String> {
        self.desired_digests.remove(uri);
        let previous = self.documents.remove(uri);
        if previous.is_some() {
            self.record_change(uri.into());
        }
        previous
    }

    pub(super) fn clear(&mut self) {
        self.desired_digests.clear();
        if self.documents.is_empty() {
            return;
        }
        if let Some(pending) = &mut self.pending {
            pending.extend(self.documents.keys().cloned());
        }
        self.documents.clear();
        self.advance_revision();
    }

    pub(super) fn insert_if_absent(&mut self, uri: String, text: String) {
        if !self.documents.contains_key(uri.as_str()) {
            self.insert(uri, text);
        }
    }

    /// Hash a retained revision once, only when a physical observation needs
    /// its digest. The expected row can precede another row for the same URI
    /// in a batch; that earlier text must never borrow the final row's digest.
    pub(super) fn desired_digest(&mut self, uri: &str, expected: &str) -> [u8; 32] {
        let retained = self.documents.get(uri).map(String::as_str) == Some(expected);
        if retained && let Some(digest) = self.desired_digests.get(uri) {
            return *digest;
        }
        #[cfg(test)]
        {
            self.digest_computations += 1;
        }
        let digest = Sha256::digest(expected.as_bytes()).into();
        if retained {
            self.desired_digests.insert(uri.into(), digest);
        }
        digest
    }

    pub(super) fn changes_since(
        &self,
        checkpoint: Option<DocumentCheckpoint>,
    ) -> Option<&FxHashSet<String>> {
        let expected = DocumentCheckpoint {
            owner: self.owner,
            revision: self.baseline,
        };
        (checkpoint == Some(expected))
            .then_some(self.pending.as_ref())
            .flatten()
    }

    /// Called only after every selected overlay operation has succeeded.
    pub(super) fn acknowledge(&mut self) -> DocumentCheckpoint {
        self.baseline = self.revision;
        self.baseline_len = self.documents.len();
        self.pending = Some(FxHashSet::default());
        DocumentCheckpoint {
            owner: self.owner,
            revision: self.revision,
        }
    }

    fn record_change(&mut self, uri: String) {
        if let Some(pending) = &mut self.pending {
            pending.insert(uri);
        }
        self.advance_revision();
    }

    fn advance_revision(&mut self) {
        self.revision = self.revision.wrapping_add(1);
        // URI churn must not retain an unbounded journal between requests.
        // More names than the two source sets can contain requires a full sync.
        if self.pending.as_ref().is_some_and(|pending| {
            pending.len() > self.documents.len().saturating_add(self.baseline_len)
        }) {
            self.pending = None;
        }
    }
}

impl Deref for TrackedDocumentTexts {
    type Target = FxHashMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.documents
    }
}

impl Clone for TrackedDocumentTexts {
    fn clone(&self) -> Self {
        Self::from(self.documents.clone())
    }
}

impl PartialEq for TrackedDocumentTexts {
    fn eq(&self, other: &Self) -> bool {
        self.documents == other.documents
    }
}

impl PartialEq<FxHashMap<String, String>> for TrackedDocumentTexts {
    fn eq(&self, other: &FxHashMap<String, String>) -> bool {
        &self.documents == other
    }
}

impl Serialize for TrackedDocumentTexts {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.documents.serialize(serializer)
    }
}

impl IntoIterator for TrackedDocumentTexts {
    type Item = (String, String);
    type IntoIter = std::collections::hash_map::IntoIter<String, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.documents.into_iter()
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod digest_tests;
