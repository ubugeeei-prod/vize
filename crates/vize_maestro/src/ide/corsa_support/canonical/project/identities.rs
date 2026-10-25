//! Preserve accepted URI owners while avoiding repeated pairwise URI parsing.

use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use tower_lsp::lsp_types::Url;
use vize_l0::{FxHashMap, String};

use super::super::location_matches_uri_cached;
use crate::server::ServerState;

struct FileIdentity {
    // The serialized path deliberately retains percent spelling and ignores
    // authority/query/fragment, exactly like virtual_document_path.
    virtual_path: String,
    path: PathBuf,
}

impl FileIdentity {
    fn parse(raw: &str) -> Option<Self> {
        let uri = Url::parse(raw).ok()?;
        if uri.host_str().is_some_and(|host| host != "localhost") {
            return None;
        }
        Some(Self {
            path: uri.to_file_path().ok()?,
            virtual_path: uri.path().into(),
        })
    }
}

struct Accepted {
    raw: String,
    file: Option<FileIdentity>,
}

#[derive(Clone, Copy)]
enum OperandOrder {
    CandidateFirst,
    AcceptedFirst,
}

#[derive(Default)]
struct Work {
    parses: usize,
    probes: usize,
    physical_resolutions: usize,
    legacy_comparisons: usize,
    physical_resets: usize,
    primary_comparisons: usize,
}

/// One synchronous merge/filter owns this index. No lease crosses an await.
pub(super) struct UriMembership {
    accepted: Vec<Accepted>,
    first_path: FxHashMap<String, usize>,
    first_physical: FxHashMap<PathBuf, usize>,
    pending_physical: VecDeque<usize>,
    physical_epoch: Option<u64>,
    legacy_only: bool,
    order: OperandOrder,
    work: Work,
}

impl UriMembership {
    pub(super) fn candidate_first<'a>(uris: impl IntoIterator<Item = &'a str>) -> Self {
        Self::with_order(uris, OperandOrder::CandidateFirst)
    }

    pub(super) fn accepted_first<'a>(uris: impl IntoIterator<Item = &'a str>) -> Self {
        Self::with_order(uris, OperandOrder::AcceptedFirst)
    }

    fn with_order<'a>(uris: impl IntoIterator<Item = &'a str>, order: OperandOrder) -> Self {
        let mut result = Self {
            accepted: Vec::new(),
            first_path: FxHashMap::default(),
            first_physical: FxHashMap::default(),
            pending_physical: VecDeque::new(),
            physical_epoch: None,
            legacy_only: false,
            order,
            work: Work::default(),
        };
        // Existing vectors already own their payloads. Seeding must not dedup
        // them, or resolve a physical path the legacy walk never inspected.
        for uri in uris {
            let file = FileIdentity::parse(uri);
            result.work.parses += 1;
            result.accept(uri, file);
        }
        result
    }

    fn accept(&mut self, raw: &str, file: Option<FileIdentity>) {
        let index = self.accepted.len();
        if let Some(file) = &file {
            self.first_path
                .entry(file.virtual_path.clone())
                .or_insert(index);
            self.pending_physical.push_back(index);
        } else {
            self.legacy_only = true;
        }
        self.accepted.push(Accepted {
            raw: raw.into(),
            file,
        });
    }

    pub(super) fn insert_if_absent(&mut self, state: &ServerState, raw: &str) -> bool {
        let file = FileIdentity::parse(raw);
        self.work.parses += 1;
        if self.contains_parsed(state, raw, file.as_ref()) {
            return false;
        }
        // The OR relation is not transitive. A rejected alias must never add
        // keys which could suppress a later, distinct accepted owner.
        self.accept(raw, file);
        true
    }

    pub(super) fn insert_unless_primary(
        &mut self,
        state: &ServerState,
        raw: &str,
        primary: &str,
    ) -> bool {
        self.work.primary_comparisons += 1;
        !location_matches_uri_cached(state, raw, primary) && self.insert_if_absent(state, raw)
    }

    pub(super) fn has_equivalent_owner(&mut self, state: &ServerState, raw: &str) -> bool {
        let file = FileIdentity::parse(raw);
        self.work.parses += 1;
        self.contains_parsed(state, raw, file.as_ref())
    }

    fn contains_parsed(
        &mut self,
        state: &ServerState,
        raw: &str,
        file: Option<&FileIdentity>,
    ) -> bool {
        self.work.probes += 1;
        let Some(file) = file.filter(|_| !self.legacy_only) else {
            // In particular, preserve None == None for unparseable virtual
            // paths. No URI policy is introduced by this performance change.
            return self.legacy_contains(state, raw);
        };
        let limit = self.accepted.len();
        // For admitted file URIs, raw equality implies equal serialized paths.
        // The first path hit is therefore the first raw-or-virtual hit too.
        let cutoff = self
            .first_path
            .get(&file.virtual_path)
            .copied()
            .unwrap_or(limit);
        if cutoff == 0 || limit == 0 {
            return cutoff < limit;
        }

        // A short typed lease excludes concurrent cache clearing, including
        // notification workers which do not retire the native request scope.
        let indexed = {
            let world = state.canonical_path_read();
            self.contains_file(file, cutoff, world.epoch(), |path| {
                let physical = world.resolve(path);
                world
                    .matches_observation(path, &physical)
                    .then_some(physical)
            })
        };
        if let Some(found) = indexed {
            return found;
        }
        // A conflicting concurrent miss is not a reusable physical fact.
        // Retry the exact legacy predicate after releasing the short lease.
        self.physical_epoch = None;
        self.legacy_contains(state, raw)
    }

    fn legacy_contains(&mut self, state: &ServerState, raw: &str) -> bool {
        for accepted in &self.accepted {
            self.work.legacy_comparisons += 1;
            let matches = match self.order {
                OperandOrder::CandidateFirst => {
                    location_matches_uri_cached(state, raw, &accepted.raw)
                }
                OperandOrder::AcceptedFirst => {
                    location_matches_uri_cached(state, &accepted.raw, raw)
                }
            };
            if matches {
                return true;
            }
        }
        false
    }

    fn contains_file(
        &mut self,
        file: &FileIdentity,
        mut cutoff: usize,
        epoch: u64,
        mut resolve: impl FnMut(&Path) -> Option<PathBuf>,
    ) -> Option<bool> {
        let limit = self.accepted.len();
        if self.physical_epoch != Some(epoch) {
            self.first_physical.clear();
            self.pending_physical = (0..limit).collect();
            self.physical_epoch = Some(epoch);
            self.work.physical_resets += 1;
        }
        // Materialized merges compare the accepted operand first. Preserve
        // that first physical observation before looking up a new candidate.
        if matches!(self.order, OperandOrder::AcceptedFirst)
            && self.pending_physical.front() == Some(&0)
        {
            self.resolve_next(&mut resolve)?;
        }
        self.work.physical_resolutions += 1;
        let physical = resolve(&file.path)?;
        cutoff = cutoff.min(self.first_physical.get(&physical).copied().unwrap_or(limit));
        while self
            .pending_physical
            .front()
            .is_some_and(|index| *index < cutoff)
        {
            let (index, resolved) = self.resolve_next(&mut resolve)?;
            if resolved == physical {
                cutoff = index;
                break;
            }
        }
        Some(cutoff < limit)
    }

    fn resolve_next(
        &mut self,
        resolve: &mut impl FnMut(&Path) -> Option<PathBuf>,
    ) -> Option<(usize, PathBuf)> {
        let index = self.pending_physical.pop_front()?;
        let file = self.accepted.get(index)?.file.as_ref()?;
        self.work.physical_resolutions += 1;
        let resolved = resolve(&file.path)?;
        self.first_physical.entry(resolved.clone()).or_insert(index);
        Some((index, resolved))
    }

    pub(super) fn record_work(&self, stage: &'static str) {
        let profiler = vize_l0::profiler::global_profiler();
        profiler.record_counter("maestro.canonical_uri.parses", self.work.parses as u64);
        profiler.record_counter("maestro.canonical_uri.probes", self.work.probes as u64);
        profiler.record_counter(
            "maestro.canonical_uri.primary_comparisons",
            self.work.primary_comparisons as u64,
        );
        profiler.record_counter(
            "maestro.canonical_uri.physical_resolutions",
            self.work.physical_resolutions as u64,
        );
        profiler.record_counter(
            "maestro.canonical_uri.legacy_comparisons",
            self.work.legacy_comparisons as u64,
        );
        static TRACE: OnceLock<bool> = OnceLock::new();
        if *TRACE.get_or_init(|| std::env::var_os("VIZE_TRACE_EDITOR_PREPARATION").is_some()) {
            tracing::info!(target: "vize_editor_preparation", phase = "canonical_uri_membership",
                stage, entries = self.accepted.len(), parses = self.work.parses,
                probes = self.work.probes, physical_resolutions = self.work.physical_resolutions,
                legacy_comparisons = self.work.legacy_comparisons,
                physical_resets = self.work.physical_resets,
                primary_comparisons = self.work.primary_comparisons);
        }
    }
}

#[cfg(test)]
mod tests;
