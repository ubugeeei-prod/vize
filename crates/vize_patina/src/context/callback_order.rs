//! Internal editor publication order from actual template callback visits.

use super::LintContext;
use crate::diagnostic::LintDiagnostic;
use vize_l0::{FxHashMap, SmallVec};

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub(crate) enum CallbackSite {
    Prescan,
    Template,
    Element(u32),
    Attributes(u32),
    Directive(u32, u32),
    Interpolation(u32),
    Conditional(u32),
    List(u32),
    Text(u32),
    Comment(u32),
    ExitElement(u32),
}

#[derive(Default)]
struct Visits {
    ordinals: SmallVec<[usize; 1]>,
    replayed: usize,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct EmissionOrder {
    visit: usize,
    rule: usize,
    emission: usize,
}

pub(super) struct CallbackOrder {
    visits: FxHashMap<CallbackSite, Visits>,
    emissions: Vec<EmissionOrder>,
    next_visit: usize,
    current: Option<usize>,
    rule: usize,
}

impl CallbackOrder {
    fn new() -> Self {
        Self {
            visits: FxHashMap::default(),
            emissions: Vec::new(),
            next_visit: 0,
            current: None,
            rule: usize::MAX,
        }
    }

    fn record(&mut self, site: CallbackSite) {
        let ordinal = self.next_visit;
        self.next_visit += 1;
        self.visits.entry(site).or_default().ordinals.push(ordinal);
        self.current = Some(ordinal);
        self.rule = usize::MAX;
    }

    fn replay(&mut self, site: CallbackSite) {
        self.current = self.visits.get_mut(&site).and_then(|visits| {
            let ordinal = visits.ordinals.get(visits.replayed).copied();
            visits.replayed += 1;
            ordinal
        });
        self.rule = usize::MAX;
    }

    fn emission(&mut self) {
        let visit = self.current.unwrap_or(usize::MAX);
        self.emissions.push(EmissionOrder {
            visit,
            rule: if self.current.is_some() {
                self.rule
            } else {
                // A callback without an original visit retains actual arrival order.
                usize::MAX
            },
            emission: self.emissions.len(),
        });
    }

    pub(super) fn finish(self, diagnostics: Vec<LintDiagnostic>) -> Vec<LintDiagnostic> {
        debug_assert_eq!(diagnostics.len(), self.emissions.len());
        let mut rows: Vec<_> = diagnostics
            .into_iter()
            .enumerate()
            .map(|(index, diagnostic)| {
                let order = self.emissions.get(index).copied().unwrap_or(EmissionOrder {
                    visit: usize::MAX,
                    rule: usize::MAX,
                    emission: index,
                });
                (order, diagnostic)
            })
            .collect();
        rows.sort_by_key(|(order, _)| *order);
        rows.into_iter().map(|(_, diagnostic)| diagnostic).collect()
    }
}

impl LintContext<'_> {
    pub(crate) fn enable_callback_order(&mut self) {
        self.callback_order = Some(Box::new(CallbackOrder::new()));
    }

    #[inline]
    pub(crate) fn record_callback_site(&mut self, site: CallbackSite) {
        if let Some(order) = &mut self.callback_order {
            order.record(site);
        }
    }

    #[inline]
    pub(crate) fn replay_callback_site(&mut self, site: CallbackSite) {
        if let Some(order) = &mut self.callback_order {
            order.replay(site);
        }
    }

    pub(crate) fn clear_callback_site(&mut self) {
        if let Some(order) = &mut self.callback_order {
            order.current = None;
            order.rule = usize::MAX;
        }
    }

    #[inline]
    pub(crate) fn set_callback_rule(&mut self, index: usize) {
        if let Some(order) = &mut self.callback_order {
            order.rule = index;
        }
    }

    #[inline]
    pub(super) fn record_emission(&mut self) {
        if let Some(order) = &mut self.callback_order {
            order.emission();
        }
    }
}
