//! The SFC descriptor as a resident query (P5-6a, parse failures in P5-6b).
//!
//! Maestro's request paths used to call `parse_sfc` on the whole buffer once
//! per request. [`ResidentDocuments`] serves them one parse per buffer
//! revision: each document is a [`SourceFile`] input of a
//! [`ResidentDatabase`], and [`sfc_descriptor`] is the memoized query every
//! request path reads. A request whose text equals the stored buffer starts
//! no revision, so every request between two keystrokes shares the one memo.
//! A rejected parse is stored with its error, so the diagnostics path can
//! publish that one parser diagnostic without parsing again.

use core::ops::Deref;

use vize_croquis::sfc::{SfcDescriptor, SfcError, SfcParseOptions, parse_sfc};
use vize_s0::{FxHashMap, String};

use crate::accounting::Accounting;
use crate::db::{ResidentDatabase, SourceFile};

#[expect(
    clippy::disallowed_types,
    reason = "one revision's descriptor is shared by every request that reads it, and a \
              request may outlive the lock it was fetched under"
)]
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

/// Line/column span of a rejected parse, one-based, as the parser reported it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DescriptorParseLoc {
    /// One-based line of the first byte.
    pub start_line: usize,
    /// One-based column of the first byte.
    pub start_column: usize,
    /// One-based line just after the span.
    pub end_line: usize,
    /// One-based column just after the span.
    pub end_column: usize,
}

/// The parser's rejection of one buffer, retained so a later request does not
/// parse again to recover the diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DescriptorParseError {
    /// Parser message, unchanged.
    pub message: String,
    /// Parser code, when it reported one.
    pub code: Option<String>,
    /// Location, when the parser reported one.
    pub loc: Option<DescriptorParseLoc>,
}

impl DescriptorParseError {
    fn from_sfc(error: &SfcError) -> Self {
        Self {
            message: error.message.clone(),
            code: error.code.clone(),
            loc: error.loc.as_ref().map(|loc| DescriptorParseLoc {
                start_line: loc.start_line,
                start_column: loc.start_column,
                end_line: loc.end_line,
                end_column: loc.end_column,
            }),
        }
    }
}

/// One revision's parse: the descriptor, or the single error that rejected it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedSfc {
    /// The buffer parsed.
    Descriptor(SharedDescriptor),
    /// The buffer was rejected. The error is the memo — not a reason to parse
    /// again.
    Failed(DescriptorParseError),
}

impl ParsedSfc {
    /// The descriptor, when this revision parsed.
    #[must_use]
    pub fn descriptor(&self) -> Option<&SharedDescriptor> {
        match self {
            Self::Descriptor(descriptor) => Some(descriptor),
            Self::Failed(_) => None,
        }
    }

    /// The descriptor, when this revision parsed.
    #[must_use]
    pub fn into_descriptor(self) -> Option<SharedDescriptor> {
        match self {
            Self::Descriptor(descriptor) => Some(descriptor),
            Self::Failed(_) => None,
        }
    }
}

/// Parse `text` as an SFC named `filename` — the clean path the query
/// memoizes. `None` when the parser rejects the source; the error is kept by
/// [`parse_outcome`].
#[must_use]
pub fn parse_descriptor(filename: &str, text: &str) -> Option<SharedDescriptor> {
    parse_outcome(filename, text).into_descriptor()
}

/// Parse `text` as an SFC named `filename`, keeping a rejection.
#[must_use]
pub fn parse_outcome(filename: &str, text: &str) -> ParsedSfc {
    let options = SfcParseOptions {
        filename: String::from(filename),
        ..Default::default()
    };
    match parse_sfc(text, options) {
        Ok(descriptor) => {
            ParsedSfc::Descriptor(SharedDescriptor(Shared::new(descriptor.into_owned())))
        }
        Err(error) => ParsedSfc::Failed(DescriptorParseError::from_sfc(&error)),
    }
}

/// The file's SFC parse, with its path as the filename. A rejection is part
/// of the memo.
#[salsa::tracked(returns(ref))]
pub fn sfc_descriptor(db: &dyn salsa::Database, file: SourceFile) -> ParsedSfc {
    parse_outcome(file.path(db).as_str(), file.text(db).as_str())
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
    /// new revision; an equal one reads the memo. `None` when this revision
    /// was rejected — the error itself is [`parsed`](Self::parsed).
    pub fn descriptor(
        &mut self,
        key: &str,
        filename: &str,
        text: &str,
    ) -> Option<SharedDescriptor> {
        self.parsed(key, filename, text).into_descriptor()
    }

    /// The parse of document `key` whose current text is `text`, parsed as
    /// `filename`, including a rejection. A text that differs from the stored
    /// buffer is a new revision; an equal one reads the memo.
    pub fn parsed(&mut self, key: &str, filename: &str, text: &str) -> ParsedSfc {
        self.lookups += 1;
        let file = self.file(key, filename, text);
        let parsed = sfc_descriptor(&self.db, file).clone();
        // Drain the event records on every lookup: a long-lived process
        // must not accumulate them.
        self.parses += executions(&self.db.take_accounting());
        parsed
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
