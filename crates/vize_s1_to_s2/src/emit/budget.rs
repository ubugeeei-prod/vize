use vize_davinci::pass::{BudgetObserver, PassObserver};
use vize_s0::{Allocator, ensure_sufficient_stack};
use vize_s1::{SurfaceParseOptions, parse_with_options};

use crate::lower::{LegacyCaps, lower_with_caps_and_comment_policy};
use crate::pass::{TransformProfile, run_dom_transform_with_profile};

use super::run::{DomEmitObservation, emit_dom_observed};
use super::{DomEmit, DomEmitOptions, EmitError};

/// Observer-facing counts for the S2 DOM emitter.
///
/// `transform` reports only pass-manager walks still needed before DOM
/// emission. On the Vue 3 build path, preserving fact products are folded into
/// the emitter setup, so the single code-producing walk is the whole
/// observer-facing budget. Helper/name/static probes are subtree queries and
/// stay outside the P2-12a traversal baseline by design.
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

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ObservedPatchFactsEmit {
    pub(crate) emit: DomEmit,
    pub(crate) materialized_entries: usize,
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
    let mut transform = BudgetObserver::new();
    let observed = emit_dom_source_with_options_and_observer(
        allocator,
        source,
        caps,
        options,
        &mut transform,
    )?;
    Ok(ObservedDomEmit {
        emit: observed.emit,
        budget: DomEmitBudget {
            transform,
            emit_walks: 1,
            emit_visits: observed.emit_visits,
        },
    })
}

#[cfg(test)]
pub(crate) fn emit_dom_source_patch_facts_observed<'a>(
    allocator: &'a Allocator,
    source: &'a str,
) -> Result<ObservedPatchFactsEmit, EmitError> {
    let mut observer = vize_davinci::pass::NoObserver;
    let observed = emit_dom_source_with_options_and_observer(
        allocator,
        source,
        LegacyCaps::VUE3,
        &DomEmitOptions::DEFAULT,
        &mut observer,
    )?;
    Ok(ObservedPatchFactsEmit {
        emit: observed.emit,
        materialized_entries: observed.patch_fact_entries,
    })
}

pub(super) fn emit_dom_source_with_options_and_observer<'a, O: PassObserver>(
    allocator: &'a Allocator,
    source: &'a str,
    caps: LegacyCaps,
    options: &DomEmitOptions<'_>,
    observer: &mut O,
) -> Result<DomEmitObservation, EmitError> {
    ensure_sufficient_stack(|| {
        let (tree, errors) = parse_with_options(
            allocator,
            source,
            SurfaceParseOptions {
                experimental_in_tag_comments: options.experimental_in_tag_comments,
            },
        );
        let mut lowered = lower_with_caps_and_comment_policy(
            allocator,
            &tree,
            &errors,
            caps,
            options.comments,
            options.custom_element_patterns,
            options.custom_element_predicate,
        );
        let mut profile = TransformProfile::DEFAULT;
        if !options.hoist_static {
            profile = profile.without_static_analysis();
        }
        let facts = run_dom_transform_with_profile(&mut lowered, observer, profile);
        emit_dom_observed(&lowered, &facts, options)
    })
}
