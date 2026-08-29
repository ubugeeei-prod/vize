//! The hoist-static pass's lattice walk — the post-order region
//! recursion computing [`StaticFacts`] per owner, split from
//! `pass/hoist.rs` under the source budget. The rules live in
//! `hoist.rs`'s module docs; this file is their single implementation.

use alloc::vec::Vec as StdVec;

use vize_davinci::side_table::SideTable;
use vize_s0::cstr;
use vize_s2::op::{
    Attribute, BindingOp, ComponentOp, DynamicName, ElementOp, Namespace, Op, SlotOp,
};
use vize_s2::provenance::ProvenanceRecord;

use super::consts::{constant_for_hoist, hoistable_binding};
use super::{StaticFacts, StaticLevel};
use crate::pass::walk::PageWalk;

/// What one child op contributes to its parent's three predicates.
pub(super) struct Contribution {
    /// The child's effect on the parent's level.
    level: ChildLevel,
    /// `is_static_nested_child` twin.
    nested: bool,
    /// The native-descendants predicate's per-child twin.
    native: bool,
}

pub(super) enum ChildLevel {
    /// Keeps the parent fully static.
    Static,
    /// Forces `HasDynamicText` (an interpolation).
    DynamicText,
    /// Forces `NotStatic`.
    Dynamic,
}

/// One region's aggregated children facts.
pub(super) struct RegionSummary {
    any_dynamic: bool,
    any_dynamic_text: bool,
    all_nested: bool,
    all_native: bool,
    child_count: usize,
}

impl RegionSummary {
    fn empty() -> Self {
        Self {
            any_dynamic: false,
            any_dynamic_text: false,
            all_nested: true,
            all_native: true,
            child_count: 0,
        }
    }

    fn absorb(&mut self, contribution: &Contribution) {
        match contribution.level {
            ChildLevel::Static => {}
            ChildLevel::DynamicText => self.any_dynamic_text = true,
            ChildLevel::Dynamic => self.any_dynamic = true,
        }
        self.all_nested &= contribution.nested;
        self.all_native &= contribution.native;
        self.child_count += 1;
    }
}

/// Visit one region, returning its aggregate; children before parents,
/// ids minted through the one shared arithmetic. `ns` is the region's
/// markup-namespace context (components inherit it — they carry none
/// of their own — and element children re-enter through the
/// integration points, the lowering's own rule mirrored).
pub(super) fn visit_region(
    walk: &mut PageWalk,
    ops: &[Op<'_>],
    ns: Namespace,
    provenance: &mut StdVec<ProvenanceRecord>,
    facts: &mut SideTable<StaticFacts>,
) -> RegionSummary {
    let mut summary = RegionSummary::empty();
    for op in ops {
        let id = walk.mint();
        let contribution = match op {
            Op::Element(element) => {
                for _ in element.bindings.iter() {
                    let _ = walk.mint();
                }
                let children = visit_region(
                    walk,
                    &element.children.ops,
                    children_ns(element.namespace, element.tag),
                    provenance,
                    facts,
                );
                let fact = element_facts(element, &children);
                publish(provenance, facts, id, "ui.element", element.span, fact);
                element_contribution(element, fact)
            }
            Op::Component(component) => {
                for _ in component.bindings.iter() {
                    let _ = walk.mint();
                }
                let children = visit_region(walk, &component.children.ops, ns, provenance, facts);
                let fact = component_facts(component, ns, &children);
                publish(provenance, facts, id, "ui.component", component.span, fact);
                // The legacy lattice: a component child is dynamic, is
                // never a static nested child, and breaks the
                // native-descendants predicate.
                Contribution {
                    level: ChildLevel::Dynamic,
                    nested: false,
                    native: false,
                }
            }
            Op::Text(_) => Contribution {
                level: ChildLevel::Static,
                nested: true,
                native: true,
            },
            Op::Interpolation(_) => Contribution {
                level: ChildLevel::DynamicText,
                nested: true,
                native: true,
            },
            Op::If(if_op) => {
                for branch in if_op.branches.iter() {
                    visit_region(walk, &branch.region.ops, ns, provenance, facts);
                }
                Contribution {
                    level: ChildLevel::Dynamic,
                    nested: false,
                    native: false,
                }
            }
            Op::For(for_op) => {
                visit_region(walk, &for_op.region.ops, ns, provenance, facts);
                Contribution {
                    level: ChildLevel::Dynamic,
                    nested: false,
                    native: false,
                }
            }
            Op::Slot(slot) => {
                for _ in slot.bindings.iter() {
                    let _ = walk.mint();
                }
                visit_region(walk, &slot.fallback.ops, ns, provenance, facts);
                // An outlet is dynamic and non-native, but counts as a
                // static nested child when its whole props surface is
                // static (the shipped `is_plain_static_nested_element`
                // slot arm).
                Contribution {
                    level: ChildLevel::Dynamic,
                    nested: slot_props_static(slot),
                    native: false,
                }
            }
        };
        summary.absorb(&contribution);
    }
    summary
}

/// The element's own facts from its surface plus its children summary.
fn element_facts(element: &ElementOp<'_>, children: &RegionSummary) -> StaticFacts {
    let props_static = props_all_hoistable(&element.attributes, &element.bindings);
    // The shipped svg quirk: any directive on a literal `<svg>` tag
    // blocks staticness outright.
    let svg_blocked = element.tag == "svg" && !element.bindings.is_empty();
    let level = if svg_blocked || !props_static || children.any_dynamic {
        StaticLevel::NotStatic
    } else if children.any_dynamic_text {
        StaticLevel::HasDynamicText
    } else {
        StaticLevel::FullyStatic
    };
    StaticFacts {
        level,
        props_hoistable: props_static
            && !(element.attributes.is_empty() && element.bindings.is_empty()),
        nested_static: children.child_count > 0 && children.all_nested,
        native_descendants: children.all_native,
        foreign: element.namespace != Namespace::Html,
    }
}

/// A component's facts: level fixed at `NotStatic` (the shipped
/// `tag_type != Element` gate), the props/children predicates real —
/// the `NotStatic` props-hoist arm reads them for components too.
fn component_facts(
    component: &ComponentOp<'_>,
    ns: Namespace,
    children: &RegionSummary,
) -> StaticFacts {
    let props_static = props_all_hoistable(&component.attributes, &component.bindings);
    StaticFacts {
        level: StaticLevel::NotStatic,
        props_hoistable: props_static
            && !(component.attributes.is_empty() && component.bindings.is_empty()),
        nested_static: children.child_count > 0 && children.all_nested,
        native_descendants: children.all_native,
        foreign: ns != Namespace::Html,
    }
}

/// The namespace an element's children live in — the lowering's
/// integration-point rule (`lower/element.rs`), mirrored: the analysis
/// reads the materialized per-element namespace and only re-derives the
/// context boundary.
fn children_ns(own: Namespace, tag: &str) -> Namespace {
    match own {
        Namespace::Svg if matches!(tag, "foreignObject" | "desc" | "title") => Namespace::Html,
        Namespace::MathMl if matches!(tag, "mi" | "mo" | "mn" | "ms" | "mtext") => Namespace::Html,
        other => other,
    }
}

/// The element's contribution to its parent: `FullyStatic` keeps the
/// parent static, anything else (dynamic text included) is dynamic —
/// the shipped nested-element rule.
fn element_contribution(element: &ElementOp<'_>, fact: StaticFacts) -> Contribution {
    // A `v-slot` carrier is the shipped `ElementType::Template`: never
    // a static nested child, never a native descendant. (Structural
    // templates never reach S2 as elements — the lowering unwraps
    // them.)
    let slot_carrier = element
        .bindings
        .iter()
        .any(|binding| matches!(binding, BindingOp::SlotContent(_)));
    Contribution {
        level: match fact.level {
            StaticLevel::FullyStatic => ChildLevel::Static,
            StaticLevel::HasDynamicText | StaticLevel::NotStatic => ChildLevel::Dynamic,
        },
        nested: !slot_carrier
            && props_all_hoistable(&element.attributes, &element.bindings)
            && (element.children.ops.is_empty() || fact.nested_static),
        // The shipped native predicate reads `tag_type == Element` only
        // — no namespace test — so a foreign-namespace element subtree
        // still counts native, mirrored.
        native: !slot_carrier && fact.native_descendants,
    }
}

/// `props_are_static_attrs` / the prop half of `has_static_props`:
/// every attribute except `ref`, every binding through the mirrored
/// `is_hoistable_static_prop` rule.
fn props_all_hoistable(attributes: &[Attribute<'_>], bindings: &[BindingOp<'_>]) -> bool {
    attributes.iter().all(|attribute| attribute.name != "ref")
        && bindings.iter().all(hoistable_binding)
}

/// The outlet's static-nested-child rule: its props surface (the
/// separated `name` position included) must be fully static.
fn slot_props_static(slot: &SlotOp<'_>) -> bool {
    let name_static = match &slot.name {
        DynamicName::Static(_) => true,
        DynamicName::Dynamic(expr) => constant_for_hoist(expr),
    };
    name_static && props_all_hoistable(&slot.attributes, &slot.bindings)
}

/// Publish one owner's fact with its provenance record.
fn publish(
    provenance: &mut StdVec<ProvenanceRecord>,
    facts: &mut SideTable<StaticFacts>,
    id: Option<vize_davinci::id::NodeId>,
    owner: &str,
    span: vize_s0::Span,
    fact: StaticFacts,
) {
    // Past id exhaustion there is no key to publish under; the analysis
    // stays total and only the keying stops (the lowering's own rule).
    let Some(id) = id else {
        return;
    };
    let level = match fact.level {
        StaticLevel::NotStatic => "not-static",
        StaticLevel::HasDynamicText => "dynamic-text",
        StaticLevel::FullyStatic => "fully-static",
    };
    provenance.push(ProvenanceRecord {
        rule: vize_s0::String::from("pass.hoist-static.fact"),
        node: Some(id),
        before: vize_s0::String::from(owner),
        after: cstr!(
            "level={level} props={} nested={} native={}",
            fact.props_hoistable,
            fact.nested_static,
            fact.native_descendants
        ),
        span,
    });
    facts.insert(id, fact);
}
