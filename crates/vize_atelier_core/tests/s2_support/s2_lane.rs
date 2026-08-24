//! The S2-lane projection: the DOM-output-determining facts of every
//! `ui.if` (post v-if pass), every `ui.for`, and — since series 5 —
//! every owner's binding surface, in document order.
//!
//! Facts are keyed by page-order ids, so the walk re-derives them with
//! the same numbering rule the folio's `ops=` header states: op line,
//! attached bindings, then children.

use vize_carton::String;
use vize_davinci::id::NodeId;
use vize_davinci::side_table::SideTable;
use vize_disegno::folio::{DisegnoFolio, FolioExpr, FolioOp};
use vize_ricalco::pass::{BranchKeyKind, IfFacts, ModelFacts, SlotFacts, TextFacts};

use super::slots::{POutlet, PUnit, has_slot_content, s2_outlet, s2_slot_active, s2_unit};
use super::surface::PSurface;
use super::surface_s2::{opens_pattern_scope, surface_of};
use super::text::{TPart, TUnit};
use super::text_old::is_rawtext_tag;

/// One branch's extracted key, as the fact records it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum S2Key {
    /// A static `key` attribute (`None` = bare).
    Static(Option<String>),
    /// A `:key` binding's trimmed value text.
    Dynamic(String),
}

/// One branch's projected facts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct S2Branch {
    /// The trimmed condition source text; `None` = unconditional.
    pub condition: Option<String>,
    /// The extracted key fact, when the pass lifted one.
    pub key: Option<S2Key>,
}

/// One `ui.if`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct S2Chain {
    /// The branches, in authored order.
    pub branches: Vec<S2Branch>,
}

/// One `ui.for`'s projected facts — the binding surface. An alias
/// position is `None` when unauthored (for the value position: the
/// zero-width escape of an absent alias; an *undecomposable* value
/// never reaches the projection, because its lowering error skips the
/// template pre-pass).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct S2For {
    /// The iterated source's trimmed text.
    pub source: String,
    /// The value alias's trimmed text.
    pub value: Option<String>,
    /// The second position (object key).
    pub key: Option<String>,
    /// The third position (index).
    pub index: Option<String>,
}

/// Everything the S2 lane projects out of one artifact.
pub struct S2Projection {
    pub chains: Vec<S2Chain>,
    pub fors: Vec<S2For>,
    pub units: Vec<PUnit>,
    pub outlets: Vec<POutlet>,
    /// The text units (series 4): one per remaining text-family op —
    /// the lowering already merged every unit (`lower::text`).
    pub text_units: Vec<TUnit>,
    /// Text-family ops inside rawtext content, excluded from the text
    /// projection (the lane-neutral rule, counted).
    pub text_rawtext_excluded: u64,
    /// The owner surfaces (series 5), in document order.
    pub surfaces: Vec<PSurface>,
    /// Models excluded because the v-model pass faulted them (the
    /// legacy lane removed the same models).
    pub models_invalid: u64,
    /// Branch-key bindings excluded from the surface (the chain
    /// projection owns them).
    pub keys_excluded: u64,
    /// The template authors a `<table>` (the legacy in-table tree
    /// construction class — the surface half skips, counted).
    pub has_table: bool,
}

/// The fact tables the walk reads.
pub struct Tables<'t> {
    pub if_facts: &'t SideTable<IfFacts>,
    pub slot_facts: &'t SideTable<SlotFacts>,
    pub text_facts: &'t SideTable<TextFacts>,
    pub model_faults: &'t SideTable<ModelFacts>,
}

/// Collect every chain, for, slot unit, outlet, text unit and owner
/// surface in `folio`, outer before nested, document order.
pub fn collect(folio: &DisegnoFolio, tables: &Tables<'_>) -> S2Projection {
    let mut out = S2Projection {
        chains: Vec::new(),
        fors: Vec::new(),
        units: Vec::new(),
        outlets: Vec::new(),
        text_units: Vec::new(),
        text_rawtext_excluded: 0,
        surfaces: Vec::new(),
        models_invalid: 0,
        keys_excluded: 0,
        has_table: false,
    };
    let mut next = 0u32;
    let state = WalkState {
        rawtext_depth: 0,
        pattern_scoped: false,
        excluded_bind: None,
        skip_slot_wrapper: false,
    };
    walk(&folio.ops, tables, state, &mut next, &mut out);
    out
}

/// The inherited walk state.
#[derive(Clone, Copy)]
struct WalkState {
    rawtext_depth: usize,
    pattern_scoped: bool,
    /// Branch-key bind to exclude on the next owner; consumed immediately.
    excluded_bind: Option<usize>,
    /// Skip a kept `<template v-slot>` under `ui.if` / `ui.for`.
    skip_slot_wrapper: bool,
}

fn walk(
    ops: &[FolioOp],
    tables: &Tables<'_>,
    state: WalkState,
    next: &mut u32,
    out: &mut S2Projection,
) {
    for (position, op) in ops.iter().enumerate() {
        let excluded_bind = if position == 0 {
            state.excluded_bind
        } else {
            None
        };
        match op {
            FolioOp::Element(element) => {
                if element.tag.as_str().eq_ignore_ascii_case("table") {
                    out.has_table = true;
                }
                let owner_index = *next;
                *next += 1 + u32::try_from(element.bindings.len()).expect("binding count fits");
                let skip_wrapper = state.skip_slot_wrapper
                    && element.tag.as_str() == "template"
                    && has_slot_content(&element.bindings);
                if !skip_wrapper {
                    let surface = surface_of(
                        &element.attributes,
                        &element.bindings,
                        owner_index,
                        tables,
                        state.pattern_scoped,
                        excluded_bind,
                        false,
                        out,
                    );
                    out.surfaces.push(surface);
                }
                let child_state = WalkState {
                    rawtext_depth: state.rawtext_depth
                        + usize::from(is_rawtext_tag(element.tag.as_str())),
                    pattern_scoped: state.pattern_scoped
                        || (element.tag == "template"
                            && opens_pattern_scope(&element.bindings, &element.children)),
                    excluded_bind: None,
                    skip_slot_wrapper: false,
                };
                walk(&element.children, tables, child_state, next, out);
            }
            FolioOp::Component(component) => {
                let owner_index = *next;
                let id = NodeId::from_index(owner_index);
                *next += 1 + u32::try_from(component.bindings.len()).expect("binding count fits");
                if s2_slot_active(&component.bindings, &component.children) {
                    out.units.push(s2_unit(id, tables.slot_facts));
                }
                let surface = surface_of(
                    &component.attributes,
                    &component.bindings,
                    owner_index,
                    tables,
                    state.pattern_scoped,
                    excluded_bind,
                    true,
                    out,
                );
                out.surfaces.push(surface);
                let child_state = WalkState {
                    rawtext_depth: state.rawtext_depth,
                    pattern_scoped: state.pattern_scoped
                        || opens_pattern_scope(&component.bindings, &component.children),
                    excluded_bind: None,
                    skip_slot_wrapper: false,
                };
                walk(&component.children, tables, child_state, next, out);
            }
            FolioOp::Text(text) => {
                *next += 1;
                if state.rawtext_depth > 0 {
                    out.text_rawtext_excluded += 1;
                } else {
                    out.text_units.push(TUnit {
                        parts: vec![TPart {
                            dynamic: false,
                            text: Some(text.content.clone()),
                        }],
                        compound: false,
                    });
                }
            }
            FolioOp::Interpolation(interpolation) => {
                let id = NodeId::from_index(*next);
                *next += 1;
                if state.rawtext_depth > 0 {
                    out.text_rawtext_excluded += 1;
                    continue;
                }
                let compound = id.and_then(|id| tables.text_facts.get(id));
                out.text_units.push(match compound {
                    Some(fact) => TUnit {
                        parts: fact
                            .parts
                            .iter()
                            .map(|part| TPart {
                                dynamic: part.dynamic,
                                text: Some(part.text.clone()),
                            })
                            .collect(),
                        compound: true,
                    },
                    None => TUnit {
                        parts: vec![TPart {
                            dynamic: true,
                            text: Some(expr_text(&interpolation.expression)),
                        }],
                        compound: false,
                    },
                });
            }
            FolioOp::If(if_op) => {
                let id = NodeId::from_index(*next).expect("page-order ids fit");
                *next += 1;
                let fact = tables.if_facts.get(id);
                out.chains.push(S2Chain {
                    branches: if_op
                        .branches
                        .iter()
                        .enumerate()
                        .map(|(index, branch)| S2Branch {
                            condition: branch.condition.as_ref().map(expr_text),
                            key: fact
                                .and_then(|fact| fact.branches.get(index))
                                .and_then(|key| key.as_ref())
                                .map(|key| match &key.kind {
                                    BranchKeyKind::Static(value) => S2Key::Static(
                                        value.as_ref().map(|value| String::from(value.as_str())),
                                    ),
                                    BranchKeyKind::Dynamic { source, .. } => {
                                        S2Key::Dynamic(String::from(source.as_str()))
                                    }
                                }),
                        })
                        .collect(),
                });
                for (index, branch) in if_op.branches.iter().enumerate() {
                    // The dynamic branch key's binding is excluded from
                    // its carrier's surface — the chain projection owns
                    // it, exactly as the legacy `extract_key_prop`
                    // removes the prop.
                    let excluded = fact
                        .and_then(|fact| fact.branches.get(index))
                        .and_then(|key| key.as_ref())
                        .and_then(|key| match &key.kind {
                            BranchKeyKind::Dynamic { bind_index, .. } => *bind_index,
                            BranchKeyKind::Static(_) => None,
                        });
                    let branch_state = WalkState {
                        excluded_bind: excluded,
                        skip_slot_wrapper: true,
                        ..state
                    };
                    walk(&branch.ops, tables, branch_state, next, out);
                }
            }
            FolioOp::For(for_op) => {
                *next += 1;
                out.fors.push(S2For {
                    source: expr_text(&for_op.binding.source),
                    value: alias_text(Some(&for_op.binding.value)),
                    key: alias_text(for_op.binding.key.as_ref()),
                    index: alias_text(for_op.binding.index.as_ref()),
                });
                let region_state = WalkState {
                    excluded_bind: None,
                    skip_slot_wrapper: true,
                    ..state
                };
                walk(&for_op.ops, tables, region_state, next, out);
            }
            FolioOp::Slot(slot) => {
                let owner_index = *next;
                *next += 1 + u32::try_from(slot.bindings.len()).expect("binding count fits");
                out.outlets.push(s2_outlet(&slot.name));
                let surface = surface_of(
                    &slot.attributes,
                    &slot.bindings,
                    owner_index,
                    tables,
                    state.pattern_scoped,
                    excluded_bind,
                    false,
                    out,
                );
                out.surfaces.push(surface);
                let fallback_state = WalkState {
                    excluded_bind: None,
                    ..state
                };
                walk(&slot.fallback, tables, fallback_state, next, out);
            }
        }
    }
}

/// An alias position's text: `None` when unauthored (absent position or
/// the zero-width hole).
fn alias_text(expr: Option<&FolioExpr>) -> Option<String> {
    let text = expr_text(expr?);
    if text.is_empty() { None } else { Some(text) }
}

fn expr_text(expr: &FolioExpr) -> String {
    match expr {
        FolioExpr::Js { source, .. }
        | FolioExpr::Foreign { source, .. }
        | FolioExpr::Opaque { source, .. }
        | FolioExpr::Filter { source, .. } => String::from(source.trim()),
    }
}
