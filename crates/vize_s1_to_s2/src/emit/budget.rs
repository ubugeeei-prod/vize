use vize_davinci::pass::BudgetObserver;
use vize_s0::Allocator;
use vize_s1::parse;

use crate::lower::{LegacyCaps, lower_with_caps};
use crate::pass::run_transform;

use super::{DomEmit, DomEmitOptions, EmitError, emit_dom_with_emit_budget};

/// Observer-facing counts for the S2 DOM emitter.
///
/// `emit_walks` / `emit_visits` deliberately describe the single
/// code-producing walk. Helper/name/static probes are subtree queries
/// and stay outside the P2-12a traversal baseline by design.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DomEmitBudget {
    pub transform: BudgetObserver,
    pub emit_walks: u32,
    pub emit_visits: u32,
}

impl DomEmitBudget {
    #[must_use]
    pub const fn total_walks(&self) -> u32 {
        self.transform.walks + self.emit_walks
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedDomEmit {
    pub emit: DomEmit,
    pub budget: DomEmitBudget,
}

pub fn emit_dom_source_observed<'a>(
    allocator: &'a Allocator,
    source: &'a str,
) -> Result<ObservedDomEmit, EmitError> {
    emit_dom_source_with_caps_observed(allocator, source, LegacyCaps::VUE3)
}

pub fn emit_dom_source_with_caps_observed<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    caps: LegacyCaps,
) -> Result<ObservedDomEmit, EmitError> {
    emit_dom_source_observed_with_options(allocator, source, caps, &DomEmitOptions::DEFAULT)
}

/// [`emit_dom_source_with_caps_observed`] under explicit [`DomEmitOptions`].
pub fn emit_dom_source_observed_with_options<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    caps: LegacyCaps,
    options: &DomEmitOptions<'_>,
) -> Result<ObservedDomEmit, EmitError> {
    let (tree, errors) = parse(allocator, source);
    let mut lowered = lower_with_caps(allocator, &tree, &errors, caps);
    let mut transform = BudgetObserver::new();
    let facts = run_transform(&mut lowered, &mut transform);
    let (emit, emit_visits) = emit_dom_with_emit_budget(&lowered, &facts, options)?;
    Ok(ObservedDomEmit {
        emit,
        budget: DomEmitBudget {
            transform,
            emit_walks: 1,
            emit_visits,
        },
    })
}
