//! Owned lexical facts, admitted only behind a fresh complete projection.

use std::mem::size_of;
use std::sync::{
    Arc, OnceLock,
    atomic::{AtomicUsize, Ordering},
};

use oxc_semantic::Scoping;
use oxc_syntax::reference::ReferenceId;
use parking_lot::Mutex;
use tower_lsp::lsp_types::Url;
use vize_canon::virtual_ts::{VizeMapping, VizeSemanticLink};
use vize_l0::String as CompactString;

use super::Occurrence;

mod registry;
use registry::Registry;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Mode {
    Standard,
    OptionsApi,
    LegacyVue2,
}

/// Fresh generator selection and its complete structural output witness.
/// External inputs stay fresh before lookup. The structural demand fixes the
/// remaining SFC options, default TS/TSX recovery and SemanticBuilder nodes.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct Projection {
    pub(super) filename: CompactString,
    pub(super) mode: Mode,
    pub(super) patterned: bool,
    pub(super) code: CompactString,
    pub(super) mappings: Vec<VizeMapping>,
    pub(super) links: Vec<VizeSemanticLink>,
}

pub(super) struct NamedReference {
    pub(super) name: CompactString,
    pub(super) reference: ReferenceId,
}

pub(super) struct Facts {
    pub(super) scoping: Scoping,
    pub(super) occurrences: Vec<Occurrence>,
    pub(super) references: Vec<NamedReference>,
    pub(super) css_free_names: Box<[CompactString]>,
}

pub(super) struct Entry {
    source: Arc<std::string::String>,
    revision: Option<u64>,
    uri: Url,
    pub(super) projection: Projection,
    pub(super) facts: Facts,
    lease: Option<Lease>,
}

pub(crate) struct Cell {
    entry: Mutex<Option<Arc<Entry>>>,
    budget: Arc<Budget>,
    #[cfg(test)]
    hits: AtomicUsize,
}

impl Default for Cell {
    fn default() -> Self {
        Self::new(Arc::clone(budget()))
    }
}

impl Cell {
    fn new(budget: Arc<Budget>) -> Self {
        Self {
            entry: Mutex::new(None),
            budget,
            #[cfg(test)]
            hits: AtomicUsize::new(0),
        }
    }

    #[cfg(test)]
    pub(crate) fn isolated() -> Self {
        Self::new(Arc::new(Budget::default()))
    }

    #[cfg(test)]
    pub(crate) fn hit_count(&self) -> usize {
        self.hits.load(Ordering::Relaxed)
    }

    #[cfg(test)]
    pub(crate) fn retained(&self) -> bool {
        self.entry.lock().is_some()
    }
}

impl std::fmt::Debug for Cell {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("StructuralSnapshotCell")
            .finish_non_exhaustive()
    }
}

pub(crate) struct Owner {
    cell: Arc<Cell>,
    source: Arc<std::string::String>,
    revision: u64,
    uri: Url,
}

impl Owner {
    pub(crate) fn new(
        cell: Arc<Cell>,
        revision: u64,
        uri: Url,
        source: Arc<std::string::String>,
    ) -> Self {
        Self {
            cell,
            source,
            revision,
            uri,
        }
    }

    #[cfg(test)]
    pub(crate) fn same_generation(&self, other: &Self) -> bool {
        self.revision == other.revision
            && self.uri == other.uri
            && Arc::ptr_eq(&self.source, &other.source)
            && Arc::ptr_eq(&self.cell, &other.cell)
    }

    pub(super) fn get(&self, projection: &Projection) -> Option<Arc<Entry>> {
        let mut cached = self.cell.entry.lock();
        if let Some(entry) = cached.as_ref()
            && entry.revision == Some(self.revision)
            && entry.uri == self.uri
            && Arc::ptr_eq(&entry.source, &self.source)
            && entry.projection == *projection
        {
            #[cfg(test)]
            self.cell.hits.fetch_add(1, Ordering::Relaxed);
            return Some(Arc::clone(entry));
        }
        // Retired readers own their Arc; they never revalidate another source.
        *cached = None;
        None
    }

    pub(super) fn publish(&self, projection: Projection, facts: Facts) -> Arc<Entry> {
        let mut entry = Entry::uncached(
            Arc::clone(&self.source),
            self.uri.clone(),
            projection,
            facts,
        );
        entry.revision = Some(self.revision);
        entry.lease = entry
            .charge()
            .and_then(|bytes| self.cell.budget.admit(bytes));
        let entry = Arc::new(entry);
        if entry.lease.is_some() {
            *self.cell.entry.lock() = Some(Arc::clone(&entry));
            let registered = self
                .cell
                .budget
                .registry
                .lock()
                .register(&self.cell, &entry);
            if !registered {
                // A refused registration must not leave an untracked slot.
                let mut cached = self.cell.entry.lock();
                if cached
                    .as_ref()
                    .is_some_and(|current| Arc::ptr_eq(current, &entry))
                {
                    *cached = None;
                }
            }
        }
        entry
    }
}

impl Entry {
    pub(super) fn uncached(
        source: Arc<std::string::String>,
        uri: Url,
        projection: Projection,
        facts: Facts,
    ) -> Self {
        Self {
            source,
            revision: None,
            uri,
            projection,
            facts,
            lease: None,
        }
    }

    /// Account owned buffers by their actual capacities. Scoping's private
    /// arena has no public byte-size API: bound its retained instances and
    /// actual source/symbol/reference/scope counts separately, never claim this
    /// ledger measures the opaque arena, allocator overhead, or process RSS.
    fn charge(&self) -> Option<usize> {
        let facts = &self.facts;
        if self.source.len() > 256 * 1024
            || self.projection.code.len() > 128 * 1024
            || facts.scoping.symbols_len() > 4096
            || facts.scoping.references_len() > 8192
            || facts.scoping.scopes_len() > 2048
            || facts.occurrences.len() > 16384
        {
            return None;
        }
        // Enum constant strings can grow exponentially despite small source
        // and symbol counts. They remain on the original uncached path.
        if facts
            .scoping
            .symbol_ids()
            .any(|symbol| facts.scoping.get_enum_member_value(symbol).is_some())
        {
            return None;
        }
        let mut bytes = size_of::<Self>().checked_add(2 * size_of::<usize>())?;
        let mut add = |value: usize| -> Option<()> {
            bytes = bytes.checked_add(value)?;
            Some(())
        };
        add(self.source.capacity())?;
        add(self.uri.as_str().len())?;
        add(self.projection.filename.capacity())?;
        add(self.projection.code.capacity())?;
        add(self
            .projection
            .mappings
            .capacity()
            .checked_mul(size_of::<VizeMapping>())?)?;
        for mapping in &self.projection.mappings {
            add(mapping
                .sub_spans
                .capacity()
                .checked_mul(size_of::<vize_canon::virtual_ts::mapping::VizeSubSpan>())?)?;
        }
        add(self
            .projection
            .links
            .capacity()
            .checked_mul(size_of::<VizeSemanticLink>())?)?;
        add(facts
            .occurrences
            .capacity()
            .checked_mul(size_of::<Occurrence>())?)?;
        add(facts
            .references
            .capacity()
            .checked_mul(size_of::<NamedReference>())?)?;
        for reference in &facts.references {
            add(reference.name.capacity())?;
        }
        add(facts
            .css_free_names
            .len()
            .checked_mul(size_of::<CompactString>())?)?;
        for name in &facts.css_free_names {
            add(name.capacity())?;
        }
        Some(bytes)
    }
}

/// Process-wide admission applies across every open document and server.
/// An in-flight retired Arc keeps its lease until its last reader drops it.
#[derive(Default)]
struct Budget {
    entries: AtomicUsize,
    known_bytes: AtomicUsize,
    registry: Mutex<Registry>,
}
const MAX_ENTRIES: usize = 2;
const MAX_KNOWN_BYTES: usize = 4 * 1024 * 1024;
fn budget() -> &'static Arc<Budget> {
    static BUDGET: OnceLock<Arc<Budget>> = OnceLock::new();
    BUDGET.get_or_init(|| Arc::new(Budget::default()))
}
impl Budget {
    fn admit(self: &Arc<Self>, bytes: usize) -> Option<Lease> {
        if bytes > MAX_KNOWN_BYTES {
            return None;
        }
        if let Some(lease) = self.lease(bytes) {
            return Some(lease);
        }
        // Clone only bounded weak identities, releasing the registry before
        // any cell lock or Entry drop. Active and retired readers keep leases.
        let publications = self.registry.lock().snapshot();
        if Registry::evict_oldest_idle(publications) {
            return self.lease(bytes);
        }
        None
    }

    fn lease(self: &Arc<Self>, bytes: usize) -> Option<Lease> {
        self.entries
            .try_update(Ordering::AcqRel, Ordering::Acquire, |current| {
                (current < MAX_ENTRIES).then(|| current + 1)
            })
            .ok()?;
        if self
            .known_bytes
            .try_update(Ordering::AcqRel, Ordering::Acquire, |current| {
                current
                    .checked_add(bytes)
                    .filter(|next| *next <= MAX_KNOWN_BYTES)
            })
            .is_err()
        {
            self.entries.fetch_sub(1, Ordering::AcqRel);
            return None;
        }
        Some(Lease {
            budget: Arc::clone(self),
            bytes,
        })
    }
}
struct Lease {
    budget: Arc<Budget>,
    bytes: usize,
}
impl Drop for Lease {
    fn drop(&mut self) {
        self.budget
            .known_bytes
            .fetch_sub(self.bytes, Ordering::AcqRel);
        self.budget.entries.fetch_sub(1, Ordering::AcqRel);
    }
}

#[cfg(test)]
mod tests;
