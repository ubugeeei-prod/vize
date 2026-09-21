//! The hoist-static pass's optimization remarks (P3-13): why an owner's
//! subtree or props surface is, or is not, a hoist candidate.
//!
//! The pass is an analysis, so `applied` here means "established": the
//! fact that licenses the hoist holds. Whether DOM realization then hoists
//! is position- and option-dependent (`hoist.rs`'s module docs) and is not
//! claimed. The vocabulary is registered in
//! `davinci-road/plan/remarks-format.md`:
//!
//! - `static-subtree` — per `ui.element`: `applied` when the subtree is
//!   fully static (whole-hoist eligible), else `missed` with the first
//!   blocker in the lattice's own evaluation order.
//! - `static-props` — per owner whose subtree is not whole-hoistable and
//!   whose props surface is non-empty: `applied` when the surface is
//!   hoistable on its own, else `missed` with its blocker.
//!
//! Arguments: `tag` (element) or `component` (component name), then
//! `blocker` (`svg-directive`, `ref-attribute`, `binding`, `child`), then
//! `op` (the blocking binding's or child's mnemonic), then `rule` (the
//! failed `ui.bind` gate). Every blocker comes from the same functions the
//! facts do (`consts::props_blocker`, the region summary), so a remark
//! cannot explain a decision the analysis did not make.

use vize_davinci::pass::{Remark, RemarkArg, RemarkSink};
use vize_s0::Span;
use vize_s2::op::{Attribute, BindingOp, ComponentOp, ElementOp};

use super::consts::{PropsBlocker, props_blocker};
use super::lattice::RegionSummary;
use super::{StaticFacts, StaticLevel};

/// Whole-subtree hoist eligibility.
pub const STATIC_SUBTREE: &str = "static-subtree";
/// Props-surface hoist eligibility.
pub const STATIC_PROPS: &str = "static-props";

/// Up to four arguments, built on the stack.
struct Args<'a> {
    slots: [RemarkArg<'a>; 4],
    len: usize,
}

impl<'a> Args<'a> {
    fn new(first: RemarkArg<'a>) -> Self {
        Self {
            slots: [first; 4],
            len: 1,
        }
    }

    fn push(&mut self, arg: RemarkArg<'a>) {
        self.slots[self.len] = arg;
        self.len += 1;
    }

    fn blocker(&mut self, blocker: &'a str, op: Option<&'a str>, rule: Option<&'a str>) {
        self.push(RemarkArg::str("blocker", blocker));
        if let Some(op) = op {
            self.push(RemarkArg::str("op", op));
        }
        if let Some(rule) = rule {
            self.push(RemarkArg::str("rule", rule));
        }
    }

    fn props(&mut self, blocker: PropsBlocker) {
        match blocker {
            PropsBlocker::RefAttribute => self.blocker("ref-attribute", None, None),
            PropsBlocker::Binding { op, rule } => self.blocker("binding", Some(op), rule),
        }
    }

    fn as_slice(&self) -> &[RemarkArg<'a>] {
        &self.slots[..self.len]
    }
}

/// An element's remarks, emitted after its fact is published.
pub(super) fn element<R: RemarkSink>(
    remarks: &mut R,
    element: &ElementOp<'_>,
    fact: StaticFacts,
    children: &RegionSummary,
) {
    let owner = RemarkArg::str("tag", element.tag);
    let mut args = Args::new(owner);
    if fact.level == StaticLevel::FullyStatic {
        remarks.emit(&Remark::applied(
            STATIC_SUBTREE,
            element.span,
            args.as_slice(),
        ));
        return;
    }
    // The lattice's own order (`lattice::element_facts`): the svg quirk,
    // then the props surface, then dynamic children, then dynamic text.
    if element.tag == "svg" && !element.bindings.is_empty() {
        args.blocker("svg-directive", None, None);
    } else if let Some(blocker) = props_blocker(&element.attributes, &element.bindings) {
        args.props(blocker);
    } else if let Some(child) = children.first_dynamic.or(children.first_dynamic_text) {
        args.blocker("child", Some(child), None);
    }
    debug_assert!(args.len > 1, "a non-static subtree always has a blocker");
    remarks.emit(&Remark::missed(
        STATIC_SUBTREE,
        element.span,
        args.as_slice(),
    ));
    props(
        remarks,
        owner,
        element.span,
        (&element.attributes, &element.bindings),
        fact,
    );
}

/// A component's remark: components are never whole-hoisted (the
/// shipped `tag_type != Element` gate), so only the props surface speaks.
pub(super) fn component<R: RemarkSink>(
    remarks: &mut R,
    component: &ComponentOp<'_>,
    fact: StaticFacts,
) {
    props(
        remarks,
        RemarkArg::str("component", component.name),
        component.span,
        (&component.attributes, &component.bindings),
        fact,
    );
}

/// The `static-props` remark for a non-empty props surface.
fn props<R: RemarkSink>(
    remarks: &mut R,
    owner: RemarkArg<'_>,
    span: Span,
    (attributes, bindings): (&[Attribute<'_>], &[BindingOp<'_>]),
    fact: StaticFacts,
) {
    if attributes.is_empty() && bindings.is_empty() {
        return;
    }
    let mut args = Args::new(owner);
    let blocker = props_blocker(attributes, bindings);
    debug_assert_eq!(blocker.is_none(), fact.props_hoistable);
    match blocker {
        None => remarks.emit(&Remark::applied(STATIC_PROPS, span, args.as_slice())),
        Some(blocker) => {
            args.props(blocker);
            remarks.emit(&Remark::missed(STATIC_PROPS, span, args.as_slice()));
        }
    }
}
