//! Diagnostic-only execution facts, owned by the existing AST walks.

use super::SetupContextTracker;
use crate::binding_occurrences::BindingOccurrences;

mod resolve;

type Range = (u32, u32);

#[derive(Default)]
pub(super) struct SsrFacts {
    client_regions: Vec<Range>,
    global_client_regions: Vec<(Range, Range)>,
    calls: Vec<Range>,
    functions: Vec<(Range, Option<Range>)>,
    vue_callbacks: Vec<(Range, Range, bool)>,
    opaque_functions: bool,
    occurrences: Option<BindingOccurrences>,
}

impl std::fmt::Debug for SetupContextTracker {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Preserve the public dump and every original snapshot. These private
        // diagnostic facts never enter the serialized semantic contracts.
        formatter
            .debug_struct("SetupContextTracker")
            .field("violations", &self.violations)
            .field("browser_globals", &self.browser_globals)
            .finish()
    }
}

impl SetupContextTracker {
    fn ssr_mut(&mut self) -> &mut SsrFacts {
        self.ssr.get_or_insert_with(Default::default)
    }

    pub(crate) fn note_client_region(&mut self, start: u32, end: u32) {
        self.ssr_mut().client_regions.push((start, end));
    }

    pub(crate) fn note_vue_callback(&mut self, cs: u32, ce: u32, bs: u32, be: u32, watch: bool) {
        self.ssr_mut()
            .vue_callbacks
            .push(((cs, ce), (bs, be), watch));
    }

    pub(crate) fn refuse_ssr_functions(&mut self) {
        self.ssr_mut().opaque_functions = true;
    }

    pub(crate) fn note_global_client_region(
        &mut self,
        start: u32,
        end: u32,
        probe_start: u32,
        probe_end: u32,
    ) {
        self.ssr_mut()
            .global_client_regions
            .push(((start, end), (probe_start, probe_end)));
    }

    pub(crate) fn clear_ssr_facts(&mut self) {
        self.ssr = None;
    }

    pub(crate) fn note_ssr_call(&mut self, start: u32, end: u32) {
        self.ssr_mut().calls.push((start, end));
    }

    pub(crate) fn note_ssr_function(&mut self, range: Range, declaration: Option<Range>) {
        let facts = self.ssr_mut();
        if let Some((_, old)) = facts.functions.iter_mut().find(|(body, _)| *body == range) {
            if old.is_none() && declaration.is_some() {
                *old = declaration;
            }
        } else {
            facts.functions.push((range, declaration));
        }
    }

    pub(crate) fn set_ssr_occurrences(&mut self, packet: Option<BindingOccurrences>) {
        self.ssr_mut().occurrences = packet;
    }

    pub(crate) fn take_ssr_occurrences(&mut self) -> Option<BindingOccurrences> {
        self.ssr.as_mut()?.occurrences.take()
    }

    pub(super) fn shift_ssr_offsets(&mut self, delta: u32) {
        let Some(facts) = self.ssr.as_mut() else {
            return;
        };
        for range in facts.client_regions.iter_mut().chain(&mut facts.calls) {
            range.0 = range.0.saturating_add(delta);
            range.1 = range.1.saturating_add(delta);
        }
        for (range, declaration) in &mut facts.functions {
            range.0 = range.0.saturating_add(delta);
            range.1 = range.1.saturating_add(delta);
            if let Some(declaration) = declaration {
                declaration.0 = declaration.0.saturating_add(delta);
                declaration.1 = declaration.1.saturating_add(delta);
            }
        }
        for (range, probe) in &mut facts.global_client_regions {
            for value in [range, probe] {
                value.0 = value.0.saturating_add(delta);
                value.1 = value.1.saturating_add(delta);
            }
        }
        for (callee, callback, _) in &mut facts.vue_callbacks {
            for value in [callee, callback] {
                value.0 = value.0.saturating_add(delta);
                value.1 = value.1.saturating_add(delta);
            }
        }
        if facts
            .occurrences
            .as_mut()
            .is_some_and(|packet| packet.shift_script_offsets(delta).is_none())
        {
            facts.occurrences = None;
        }
    }

    pub(super) fn extend_ssr(&mut self, other: Option<Box<SsrFacts>>) {
        let Some(other) = other else { return };
        let other = *other;
        let facts = self.ssr_mut();
        facts.client_regions.extend(other.client_regions);
        facts
            .global_client_regions
            .extend(other.global_client_regions);
        facts.calls.extend(other.calls);
        facts.functions.extend(other.functions);
        facts.vue_callbacks.extend(other.vue_callbacks);
        facts.opaque_functions |= other.opaque_functions;
        facts.occurrences = facts.occurrences.take().and_then(|mut packet| {
            packet.merge(other.occurrences?);
            Some(packet)
        });
    }
}
