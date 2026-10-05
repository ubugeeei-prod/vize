//! Demand-only authored binding occurrences, separate from serialized Croquis facts.

use crate::scope::ScopeId;
use vize_carton::{CompactString, FxHashMap, FxHashSet};

/// Identity of one authored declaration in the analyzed script view.
/// Names never establish equality between declarations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BindingIdentity {
    pub scope: ScopeId,
    pub block: OccurrenceBlock,
    pub start: u32,
    pub end: u32,
}

/// The authored block whose byte offsets an occurrence addresses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OccurrenceBlock {
    Script,
    Template,
    Style(u32),
}

/// A reference witnessed by an AST identifier and lexical binding ownership.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BindingOccurrence {
    pub binding: BindingIdentity,
    pub start: u32,
    pub end: u32,
    pub block: OccurrenceBlock,
}

/// An authored declaration. The name is a source witness, not a lookup proxy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoredBinding {
    pub identity: BindingIdentity,
    pub name: CompactString,
    /// Top-level const/let/function declaration witnessed by its original AST.
    pub lens: bool,
}

/// Owned editor facts. Ordinary Croquis dumps and snapshots do not include these.
#[derive(Debug, Clone, Default)]
pub struct BindingOccurrences {
    bindings: FxHashMap<BindingIdentity, AuthoredBinding>,
    occurrences: Vec<BindingOccurrence>,
    seen: FxHashSet<BindingOccurrence>,
    deferred_setup_reads: Vec<(CompactString, u32, u32)>,
}

impl BindingOccurrences {
    pub fn bindings(&self) -> impl Iterator<Item = &AuthoredBinding> {
        self.bindings.values()
    }

    pub fn occurrences(&self) -> &[BindingOccurrence] {
        &self.occurrences
    }

    pub(crate) fn note_binding(
        &mut self,
        scope: ScopeId,
        name: &str,
        start: u32,
        end: u32,
    ) -> BindingIdentity {
        self.note_declaration(scope, name, start, end, OccurrenceBlock::Script)
    }

    pub(crate) fn note_declaration(
        &mut self,
        scope: ScopeId,
        name: &str,
        start: u32,
        end: u32,
        block: OccurrenceBlock,
    ) -> BindingIdentity {
        let identity = BindingIdentity {
            scope,
            block,
            start,
            end,
        };
        self.bindings
            .entry(identity)
            .or_insert_with(|| AuthoredBinding {
                identity,
                name: CompactString::new(name),
                lens: false,
            });
        identity
    }

    pub(crate) fn note_lens_declaration(&mut self, identity: BindingIdentity) {
        if let Some(binding) = self.bindings.get_mut(&identity) {
            binding.lens = true;
        }
    }

    pub(crate) fn declaration(&self, name: &str, start: u32, end: u32) -> Option<BindingIdentity> {
        self.bindings
            .values()
            .find(|binding| {
                binding.name == name
                    && binding.identity.block == OccurrenceBlock::Script
                    && binding.identity.start == start
                    && binding.identity.end == end
            })
            .map(|binding| binding.identity)
    }

    /// Normalize setup-relative script facts into the existing joined script view.
    #[doc(hidden)]
    pub fn shift_script_offsets(&mut self, delta: u32) -> Option<()> {
        let mut shifted = FxHashMap::default();
        for (_, mut binding) in std::mem::take(&mut self.bindings) {
            if binding.identity.block == OccurrenceBlock::Script {
                binding.identity.start = binding.identity.start.checked_add(delta)?;
                binding.identity.end = binding.identity.end.checked_add(delta)?;
            }
            shifted.insert(binding.identity, binding);
        }
        for occurrence in &mut self.occurrences {
            if occurrence.binding.block == OccurrenceBlock::Script {
                occurrence.binding.start = occurrence.binding.start.checked_add(delta)?;
                occurrence.binding.end = occurrence.binding.end.checked_add(delta)?;
            }
            if occurrence.block == OccurrenceBlock::Script {
                occurrence.start = occurrence.start.checked_add(delta)?;
                occurrence.end = occurrence.end.checked_add(delta)?;
            }
        }
        for (_, start, end) in &mut self.deferred_setup_reads {
            *start = start.checked_add(delta)?;
            *end = end.checked_add(delta)?;
        }
        self.bindings = shifted;
        self.seen = self.occurrences.iter().cloned().collect();
        Some(())
    }

    /// Retain independent original script blocks after their existing analysis.
    #[doc(hidden)]
    pub fn merge(&mut self, other: Self) {
        self.bindings.extend(other.bindings);
        self.deferred_setup_reads.extend(other.deferred_setup_reads);
        for occurrence in other.occurrences {
            if self.seen.insert(occurrence.clone()) {
                self.occurrences.push(occurrence);
            }
        }
    }

    pub(crate) fn defer_setup_read(&mut self, name: CompactString, start: u32, end: u32) {
        self.deferred_setup_reads.push((name, start, end));
    }

    /// Join exact setup reads to the existing merged-script declaration relation.
    /// Unknown globals stay unowned; a known declaration without a packet refuses.
    #[doc(hidden)]
    pub fn resolve_script_globals(
        &mut self,
        globals: &FxHashMap<CompactString, (u32, u32)>,
    ) -> Option<()> {
        for (name, start, end) in std::mem::take(&mut self.deferred_setup_reads) {
            if let Some(&(declaration_start, declaration_end)) = globals.get(&name) {
                let binding = self.declaration(&name, declaration_start, declaration_end)?;
                self.note_reference(binding, start, end, OccurrenceBlock::Script);
            }
        }
        Some(())
    }

    /// Called by the existing demanded CSS expression analysis, not an editor request.
    #[doc(hidden)]
    pub fn note_style_read(
        &mut self,
        name: &str,
        declaration: (u32, u32),
        style: u32,
        start: u32,
        end: u32,
    ) -> Option<()> {
        let binding = self.declaration(name, declaration.0, declaration.1)?;
        self.note_reference(binding, start, end, OccurrenceBlock::Style(style));
        Some(())
    }

    pub(crate) fn note_reference(
        &mut self,
        binding: BindingIdentity,
        start: u32,
        end: u32,
        block: OccurrenceBlock,
    ) {
        let occurrence = BindingOccurrence {
            binding,
            start,
            end,
            block,
        };
        if self.seen.insert(occurrence.clone()) {
            self.occurrences.push(occurrence);
        }
    }
}
