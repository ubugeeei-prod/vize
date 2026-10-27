//! Per-generation diagnostic ownership of repeated component input expressions.
//!
//! Producers record their actual emitted rows. An inference row is suppressed
//! only after a mapped primary with the same Croquis identity and semantic role
//! has been emitted. No diagnostic messages, codes, or generated text are read.

use std::{cell::RefCell, ops::Range};

use vize_carton::FxHashSet;
use vize_croquis::croquis::{ComponentUsage, PassedProp};

mod contextual;
pub(crate) use contextual::unproven_prop_ranges;

pub(crate) type DiagnosticOwnershipCollector = RefCell<DiagnosticOwnership>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum InputRole {
    Value,
    Name,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct InputOwner {
    usage: u32,
    scope: u32,
    start: u32,
    end: u32,
    role: InputRole,
}

impl InputOwner {
    pub(crate) fn for_prop(usage: &ComponentUsage, prop: &PassedProp) -> Option<Self> {
        if prop.value.as_ref().is_some_and(|value| {
            crate::virtual_ts::scope::contains_inline_function_prop_value(value.as_str())
        }) {
            // A contextual callback can have a different contract in another
            // invocation. Its body is never classified as a diagnostic echo.
            return None;
        }
        Self::for_expected_projection(usage, prop)
    }

    /// Geometry can distinguish a proved same-root inference copy without
    /// claiming its contextual diagnostics are interchangeable with a primary.
    pub(crate) fn for_expected_projection(
        usage: &ComponentUsage,
        prop: &PassedProp,
    ) -> Option<Self> {
        if prop.name_is_dynamic || matches!(prop.name.as_str(), "key" | "ref") {
            return None;
        }
        Some(Self {
            usage: usage.start,
            scope: usage.scope_id.as_u32(),
            start: prop.start,
            end: prop.end,
            role: InputRole::Value,
        })
    }

    pub(crate) const fn with_role(mut self, role: InputRole) -> Self {
        self.role = role;
        self
    }
}

#[derive(Debug)]
pub(crate) struct DiagnosticEcho {
    pub(crate) generated: Range<usize>,
    pub(crate) authored: Range<usize>,
    pub(crate) suppress: bool,
}

#[derive(Debug)]
struct Projection {
    owner: InputOwner,
    generated: Range<usize>,
    authored: Range<usize>,
    inference: bool,
    suppress: bool,
    inert: bool,
    valid: bool,
}

#[derive(Default, Debug)]
pub(crate) struct DiagnosticOwnership {
    rows: Vec<Projection>,
    unproven: FxHashSet<InputOwner>,
    callback_primaries: Vec<Range<usize>>,
}

impl DiagnosticOwnership {
    /// This actual canonical statement may not be hidden by a literal-shaped
    /// inference copy whose contextual contract is not proved equivalent.
    pub(crate) fn protect_callback_primary(&mut self, generated: Range<usize>) {
        if !generated.is_empty() {
            self.callback_primaries.push(generated);
        }
    }

    pub(crate) fn take_callback_primaries(&mut self) -> Vec<Range<usize>> {
        std::mem::take(&mut self.callback_primaries)
    }
    pub(crate) fn refuse_owner(&mut self, owner: InputOwner) {
        self.unproven.insert(owner.with_role(InputRole::Value));
    }
    pub(crate) fn record(
        &mut self,
        owner: InputOwner,
        generated: Range<usize>,
        authored: Range<usize>,
        inference: bool,
    ) {
        self.record_projection(owner, generated, authored, inference, true);
    }

    /// An unmapped inference copy needs no primary: the native checker drops
    /// its diagnostics, which the host would otherwise report at generated
    /// offsets past the end of the SFC.
    pub(crate) fn record_inert(
        &mut self,
        owner: InputOwner,
        generated: Range<usize>,
        authored: Range<usize>,
    ) {
        if !generated.is_empty() && !authored.is_empty() {
            self.rows.push(Projection {
                owner,
                generated,
                authored,
                inference: true,
                suppress: true,
                inert: true,
                valid: true,
            });
        }
    }

    pub(crate) fn record_bare_navigation(
        &mut self,
        owner: InputOwner,
        generated: Range<usize>,
        authored: Range<usize>,
    ) {
        self.record_projection(owner, generated, authored, true, true);
    }

    pub(crate) fn record_expect_geometry(
        &mut self,
        owner: InputOwner,
        generated: Range<usize>,
        authored: Range<usize>,
        inference: bool,
    ) {
        self.record_projection(owner, generated, authored, inference, false);
    }

    fn record_projection(
        &mut self,
        owner: InputOwner,
        generated: Range<usize>,
        authored: Range<usize>,
        inference: bool,
        suppress: bool,
    ) {
        if suppress && self.unproven.contains(&owner.with_role(InputRole::Value)) {
            return;
        }
        if !generated.is_empty() && !authored.is_empty() {
            self.rows.push(Projection {
                owner,
                generated,
                authored,
                inference,
                suppress,
                inert: false,
                valid: true,
            });
        }
    }

    /// Consume the very same accepted insertion coordinates as the mapping
    /// authority. An edit inside a row removes its proof, never its diagnostic.
    pub(crate) fn note_generated_insertion(&mut self, at: usize, length: usize) {
        if length == 0 {
            return;
        }
        for range in &mut self.callback_primaries {
            if at <= range.start {
                range.start += length;
                range.end += length;
            } else if at < range.end {
                // An unproved edit can remove echo authority, but cannot
                // authorize hiding the canonical callback's diagnostics.
                range.end += length;
            }
        }
        for row in &mut self.rows {
            if at <= row.generated.start {
                row.generated.start += length;
                row.generated.end += length;
            } else if at < row.generated.end {
                row.generated.end += length;
                row.valid = false;
            }
        }
    }

    /// Deferred root buffers are rebased with their text, before publication.
    pub(crate) fn append_rebased(&mut self, other: Self, offset: usize) {
        self.unproven.extend(other.unproven);
        self.callback_primaries.extend(
            other
                .callback_primaries
                .into_iter()
                .map(|range| range.start + offset..range.end + offset),
        );
        self.rows.extend(other.rows.into_iter().map(|mut row| {
            row.generated.start += offset;
            row.generated.end += offset;
            row
        }));
    }

    pub(crate) fn into_echoes(mut self) -> Vec<DiagnosticEcho> {
        // Deferred root buffers can publish a primary before rows that were
        // recorded earlier. Generated coordinates remain the ordering authority.
        // Preserve the allocation-free fast path for ordinary monotonic emission.
        if !self.rows.is_sorted_by_key(|row| row.generated.start) {
            self.rows.sort_by_key(|row| row.generated.start);
        }
        let proven = |row: &Projection| {
            row.valid
                && (row.inert
                    || !row.suppress
                    || !self
                        .unproven
                        .contains(&row.owner.with_role(InputRole::Value)))
        };
        let owners: FxHashSet<_> = self
            .rows
            .iter()
            .filter(|row| proven(row) && !row.inference)
            .map(|row| row.owner)
            .collect();
        let mut primaries: Vec<Range<usize>> = Vec::new();
        for row in self.rows.iter().filter(|row| proven(row) && !row.inference) {
            if let Some(last) = primaries.last_mut() {
                if row.generated.start < last.start {
                    // Unproven ownership must remain diagnostically visible.
                    return Vec::new();
                }
                if row.generated.start <= last.end {
                    last.end = last.end.max(row.generated.end);
                    continue;
                }
            }
            primaries.push(row.generated.clone());
        }
        let mut cursor = 0;
        let mut last_echo = 0;
        let mut echoes = Vec::new();
        for row in self
            .rows
            .into_iter()
            .filter(|row| proven(row) && row.inference)
        {
            if row.generated.start < last_echo {
                return Vec::new();
            }
            last_echo = row.generated.start;
            while primaries
                .get(cursor)
                .is_some_and(|primary| primary.end <= row.generated.start)
            {
                cursor += 1;
            }
            let overlaps_primary = primaries
                .get(cursor)
                .is_some_and(|primary| primary.start < row.generated.end);
            if (row.inert || owners.contains(&row.owner)) && !overlaps_primary {
                echoes.push(DiagnosticEcho {
                    generated: row.generated,
                    authored: row.authored,
                    suppress: row.suppress,
                });
            }
        }
        echoes
    }
}

#[cfg(test)]
mod tests;
