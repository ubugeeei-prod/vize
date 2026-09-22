//! The SFC descriptor as a resident query (P5-6a).
//!
//! Maestro's hover, completion and definition paths used to call `parse_sfc`
//! on the whole buffer once per request — several times per request on some
//! paths. [`ResidentDocuments`] serves them one parse per buffer revision:
//! each document is a [`SourceFile`] input of a [`ResidentDatabase`], and
//! [`sfc_descriptor`] is the memoized query every request path reads. A
//! request whose text equals the stored buffer starts no revision, so every
//! request between two keystrokes shares the one memo.

use core::ops::Deref;

use vize_croquis::sfc::{SfcDescriptor, SfcParseOptions, parse_sfc};
use vize_s0::{FxHashMap, String};

use crate::accounting::Accounting;
use crate::db::{ResidentDatabase, SourceFile};

// One revision's descriptor is shared by every request that reads it, and a
// request may outlive the lock it was fetched under.
#[allow(clippy::disallowed_types)]
type Shared<T> = std::sync::Arc<T>;

/// One revision's parsed descriptor, shared by every reader of that
/// revision. Dereferences to the descriptor.
#[derive(Debug, Clone)]
pub struct SharedDescriptor(Shared<SfcDescriptor<'static>>);

impl Deref for SharedDescriptor {
    type Target = SfcDescriptor<'static>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// `parse_sfc` is a pure function of the source and the filename, so two
/// descriptors of equal inputs are equal — the equality salsa backdates on.
impl PartialEq for SharedDescriptor {
    fn eq(&self, other: &Self) -> bool {
        Shared::ptr_eq(&self.0, &other.0)
            || (self.0.filename == other.0.filename && self.0.source == other.0.source)
    }
}

impl Eq for SharedDescriptor {}

/// Parse `text` as an SFC named `filename` — the clean path the query
/// memoizes. `None` when the parser rejects the source.
#[must_use]
pub fn parse_descriptor(filename: &str, text: &str) -> Option<SharedDescriptor> {
    let options = SfcParseOptions {
        filename: String::from(filename),
        ..Default::default()
    };
    let descriptor = parse_sfc(text, options).ok()?;
    Some(SharedDescriptor(Shared::new(descriptor.into_owned())))
}

/// The file's SFC descriptor, parsed with its path as the filename.
#[salsa::tracked(returns(ref))]
pub fn sfc_descriptor(db: &dyn salsa::Database, file: SourceFile) -> Option<SharedDescriptor> {
    parse_descriptor(file.path(db).as_str(), file.text(db).as_str())
}

/// Lookups served and parses run since the last
/// [`take_stats`](ResidentDocuments::take_stats).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DescriptorStats {
    /// Requests served.
    pub lookups: u32,
    /// `sfc_descriptor` bodies run — one per buffer revision read.
    pub parses: u32,
}

/// Open documents of one long-lived process, keyed by the client's document
/// identity (a URI, or a path for a file read from disk).
#[derive(Default)]
pub struct ResidentDocuments {
    db: ResidentDatabase,
    files: FxHashMap<String, SourceFile>,
    lookups: u32,
    parses: u32,
}

impl ResidentDocuments {
    /// The descriptor of document `key` whose current text is `text`,
    /// parsed as `filename`. A text that differs from the stored buffer is a
    /// new revision; an equal one reads the memo.
    pub fn descriptor(
        &mut self,
        key: &str,
        filename: &str,
        text: &str,
    ) -> Option<SharedDescriptor> {
        self.lookups += 1;
        let file = self.file(key, filename, text);
        let descriptor = sfc_descriptor(&self.db, file).clone();
        // Drain the event records on every lookup: a long-lived process
        // must not accumulate them.
        self.parses += executions(&self.db.take_accounting());
        descriptor
    }

    /// Release document `key`'s buffer: its text becomes empty and its memo
    /// is recomputed over the empty text, so the old descriptor is dropped.
    pub fn close(&mut self, key: &str) {
        if let Some(&file) = self.files.get(key) {
            if !file.text(&self.db).is_empty() {
                self.db.edit(file, "");
            }
            let _released = sfc_descriptor(&self.db, file);
            self.parses += executions(&self.db.take_accounting());
        }
    }

    /// Lookups and parses since the last call.
    pub fn take_stats(&mut self) -> DescriptorStats {
        DescriptorStats {
            lookups: core::mem::take(&mut self.lookups),
            parses: core::mem::take(&mut self.parses),
        }
    }

    fn file(&mut self, key: &str, filename: &str, text: &str) -> SourceFile {
        if let Some(&file) = self.files.get(key) {
            if file.path(&self.db).as_str() != filename {
                self.db.rename(file, filename);
            }
            if file.text(&self.db).as_str() != text {
                self.db.edit(file, text);
            }
            return file;
        }
        let file = self.db.open(filename, text);
        self.files.insert(String::from(key), file);
        file
    }
}

/// `sfc_descriptor` bodies run in `accounting`.
fn executions(accounting: &Accounting) -> u32 {
    accounting
        .get("sfc_descriptor")
        .map_or(0, |counts| counts.executed)
}
