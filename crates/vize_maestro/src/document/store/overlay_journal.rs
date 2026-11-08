//! Bounded source-mutation journal for revision-scoped overlay consumers.

use std::collections::VecDeque;

use parking_lot::Mutex;
use tower_lsp::lsp_types::Url;

use super::DocumentStore;

/// Covers the complete 50,000-SFC scaling axis plus a large edit burst. A
/// lagging consumer falls back to one complete scan instead of trusting a gap.
const JOURNAL_CAPACITY: usize = 131_072;

#[derive(Default)]
pub(super) struct OverlayMutationJournal {
    state: Mutex<JournalState>,
}

#[derive(Default)]
struct JournalState {
    sequence: u64,
    entries: VecDeque<(u64, Url)>,
}

pub(crate) struct OverlayMutationDelta {
    pub(crate) cursor: u64,
    pub(crate) uris: Vec<Url>,
}

impl OverlayMutationJournal {
    pub(super) fn record(&self, uri: Url) {
        self.record_many([uri]);
    }

    pub(super) fn record_many(&self, uris: impl IntoIterator<Item = Url>) {
        let mut state = self.state.lock();
        state.sequence = state.sequence.wrapping_add(1);
        let sequence = state.sequence;
        for uri in uris {
            state.entries.push_back((sequence, uri));
        }
        while state.entries.len() > JOURNAL_CAPACITY {
            state.entries.pop_front();
        }
    }

    pub(super) fn cursor(&self) -> u64 {
        self.state.lock().sequence
    }

    /// Return every URI changed after `cursor`, or `None` when bounded history
    /// no longer proves the delta complete.
    pub(super) fn changes_since(&self, cursor: u64) -> Option<OverlayMutationDelta> {
        let state = self.state.lock();
        if cursor == state.sequence {
            return Some(OverlayMutationDelta {
                cursor,
                uris: Vec::new(),
            });
        }
        if cursor > state.sequence
            || state
                .entries
                .front()
                .is_none_or(|(oldest, _)| *oldest > cursor.wrapping_add(1))
        {
            return None;
        }
        Some(OverlayMutationDelta {
            cursor: state.sequence,
            uris: state
                .entries
                .iter()
                .filter(|(sequence, _)| *sequence > cursor)
                .map(|(_, uri)| uri.clone())
                .collect(),
        })
    }

    pub(super) fn acknowledge(&self, cursor: u64) {
        let mut state = self.state.lock();
        while state
            .entries
            .front()
            .is_some_and(|(sequence, _)| *sequence <= cursor)
        {
            state.entries.pop_front();
        }
    }
}

impl DocumentStore {
    pub(crate) fn overlay_change_cursor(&self) -> u64 {
        self.overlay_journal.cursor()
    }

    pub(crate) fn overlay_changes_since(&self, cursor: u64) -> Option<OverlayMutationDelta> {
        self.overlay_journal.changes_since(cursor)
    }

    pub(crate) fn acknowledge_overlay_changes(&self, cursor: u64) {
        self.overlay_journal.acknowledge(cursor);
    }
}

#[cfg(test)]
mod tests {
    use tower_lsp::lsp_types::Url;

    use super::OverlayMutationJournal;

    fn uri(name: &str) -> Url {
        Url::parse(&format!("file:///workspace/{name}")).unwrap()
    }

    #[test]
    fn acknowledged_entries_are_released_without_losing_newer_mutations() {
        let journal = OverlayMutationJournal::default();
        journal.record(uri("A.vue"));
        let first = journal.cursor();
        journal.record_many([uri("B.vue"), uri("C.vue")]);

        let complete = journal.changes_since(0).unwrap();
        assert_eq!(complete.cursor, 2);
        assert_eq!(complete.uris, [uri("A.vue"), uri("B.vue"), uri("C.vue")]);

        journal.acknowledge(first);
        let remaining = journal.changes_since(first).unwrap();
        assert_eq!(remaining.cursor, 2);
        assert_eq!(remaining.uris, [uri("B.vue"), uri("C.vue")]);

        journal.acknowledge(remaining.cursor);
        assert!(
            journal
                .changes_since(remaining.cursor)
                .unwrap()
                .uris
                .is_empty()
        );
        assert!(journal.changes_since(remaining.cursor + 1).is_none());
    }
}
