//! The `v-model` pass: the P2-9 port of the element family's one
//! remaining transform-time behaviour — the live lane's model
//! validation in `crates/vize_atelier_core/src/lane/element.rs`
//! (`process_element_props`; the old step files of the family measured
//! **dead** in the shipped lane, the installment-4 pattern again).
//!
//! # What survived the port, and where the rest went
//!
//! - **Absorbed by lowering (P2-8):** the `ui.model` contract itself
//!   (read/write pair, argument name position, element kind and
//!   modifiers as attributes), and the `VModelNoExpression` diagnostic.
//! - **DOM realization (P2-11):** everything the live lane *generates* —
//!   the component `modelValue`/`onUpdate:` product props, the native
//!   update handler, `vModelText`-family helper selection, modifier
//!   objects. The S2 contract never expands in S2 (`op/model.rs`).
//! - **The pass body (here):** the two validations the live lane runs
//!   unconditionally before expanding, with relief's exact wording:
//!   `VModelOnScope` — the model value must not be an iteration or
//!   slot-scope binding — and `VModelArgOnElement` — a plain element's
//!   model takes no argument (the default dialect; the JSX-compat
//!   static-argument allowance is that lane's option, not this pass's).
//!
//! # The scope environment (hygiene consumption, third form)
//!
//! The legacy check is `ctx.is_in_scope(value_exp)`: whole trimmed text
//! against the alias **texts** of enclosing `v-for` scopes and the
//! enumerated prop names of enclosing `v-slot` scopes. The pass mirrors
//! it structurally: a `ui.for` region opens a frame holding its
//! value/key/index source texts (texts, not enumerated names — exactly
//! what `enter_v_for_scope` registers), and a slot carrier (component,
//! or `template` element, with children and a params-bearing
//! `ui.slot-content`) opens a frame over its children holding the
//! params' simple-identifier name. A destructuring params pattern
//! contributes no name — the one-scanner rule (#4365): the S2 lane
//! enumerates pattern bindings nowhere until the `enumerate_bindings`
//! seam lands, so the legacy lane's pattern-name enumeration is
//! deliberately not imitated; the differential lane counts the class
//! (`models_pattern_scope`) instead of comparing inside it.
//!
//! # Classification (the review point)
//!
//! **`MandatoryDiagnostic`, barrier** — the series' **first** diagnostic
//! kind, and the measured answer to "one pass or several" for the
//! element family: `v-bind`/`v-on` carry **zero** transform-time
//! behaviour in the live lane (their work is all codegen — realization),
//! so their port is entirely lowering; what is left of the family is
//! exactly these diagnostics.
//!
//! - *Why mandatory:* both errors are user-visible at every tier — the
//!   live lane validates unconditionally; skipping loses them.
//! - *Why `MandatoryDiagnostic` and not `MandatoryLowering`:* the pass
//!   canonicalizes nothing and mutates nothing — the artifact and every
//!   fact the lowering published are byte-identical across it; its
//!   whole product is the two diagnostics plus [`ModelFacts`], the
//!   record of which models failed (the legacy lane *removes* invalid
//!   models from its tree — a binding op cannot leave the S2 surface
//!   without shifting every page-order id, so the fault fact is the
//!   removal's preserving twin). The diagnostic kind's literal
//!   definition finally fits, after three installments of the recorded
//!   preserving-mandatory tension on the lowering side.
//! - *Why barrier:* law 1 (mandatory passes never fuse — enforced at
//!   construction), and independently the scope environment is
//!   ancestor-context a fused single visit does not carry.
//! - `Preserved::ALL`: nothing moves.
//!
//! # Facts and accounting
//!
//! The pass drives its own shaped recursion over the shared
//! [`super::walk::PageWalk`] (the environment is call-stack scoped), and
//! asserts the re-derived count against the lowering's minted accounting
//! on every run. [`ModelFacts`] is keyed by the `ui.model` binding op's
//! page-order id, sparse: entries exist only for faulted models. Every
//! diagnostic leaves provenance (`error.v-model-on-scope` /
//! `error.v-model-arg-on-element`).

use alloc::vec::Vec as StdVec;

use vize_davinci::diagnostic::Diagnostic;
use vize_davinci::pass::{Fusability, PassDesc, PassKind, Preserved};
use vize_davinci::side_table::SideTable;
use vize_s0::String;
use vize_s2::op::{BindingOp, Op};
use vize_s2::provenance::ProvenanceRecord;

use super::walk::{PageWalk, assert_accounting};
use crate::lower::Lowered;

mod check;
use check::check_model;

/// The pass name in pipeline strings and folio pages.
pub const NAME: &str = "v-model";

/// The on-scope message, byte-identical to relief's
/// `ErrorCode::VModelOnScope` (pinned in the `vize_atelier_core`
/// differential suite, which owns the relief edge).
pub const ON_SCOPE_MESSAGE: &str = "v-model cannot be used on v-for or v-slot scope variables.";

/// See [`ON_SCOPE_MESSAGE`]; relief's `ErrorCode::VModelArgOnElement`.
pub const ARG_ON_ELEMENT_MESSAGE: &str = "v-model argument is not supported on plain elements.";

/// The pass description — classification reasoning in the module docs.
pub const DESC: PassDesc = PassDesc::new(
    NAME,
    PassKind::MandatoryDiagnostic,
    Fusability::Barrier,
    // The pass touches nothing: diagnostics and facts beside the tree.
    Preserved::ALL,
);

/// Why one `ui.model` is unrealizable where it stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelFault {
    /// The model value is an iteration or slot-scope binding
    /// (`VModelOnScope`).
    OnScope,
    /// A plain element's model carries an argument
    /// (`VModelArgOnElement`).
    ArgOnElement,
}

/// Per-`ui.model` validation facts, keyed by the binding op's page-order
/// id; entries exist only for faulted models (sparse-table discipline).
/// The legacy lane removes these models from its tree — realization and
/// the differential projection read this fact as that removal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelFacts {
    /// The first failing check, in the legacy lane's check order.
    pub fault: ModelFault,
}

/// Facts cross compile boundaries with their artifact (P1-11).
const _: () = {
    const fn assert_owned<T: 'static>() {}
    assert_owned::<ModelFault>();
    assert_owned::<ModelFacts>();
};

/// 64-bit footprints, guarded like every fact-size assert.
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<ModelFault>() == 1);
    assert!(core::mem::size_of::<ModelFacts>() == 1);
};

/// Run the pass over `lowered`.
///
/// One shaped page-order recursion carrying the scope environment;
/// diagnostics and provenance append to `lowered`'s own channels, the
/// tree is untouched.
///
/// # Panics
///
/// Panics when the re-derived page-order count disagrees with the
/// lowering's minted accounting — a compiler bug by the id law, never an
/// input property.
#[must_use]
pub fn run(lowered: &mut Lowered<'_>) -> SideTable<ModelFacts> {
    let Lowered {
        root,
        op_count,
        diagnostics,
        provenance,
        ..
    } = lowered;
    let mut channels = Channels {
        diagnostics,
        provenance,
        facts: SideTable::new(),
    };
    let mut walk = PageWalk::new();
    let mut env: StdVec<String> = StdVec::new();
    region(&mut walk, &mut channels, &mut env, &root.ops);
    assert_accounting(&walk, *op_count, NAME);
    channels.facts
}

pub(super) struct Channels<'l> {
    pub(super) diagnostics: &'l mut StdVec<Diagnostic>,
    pub(super) provenance: &'l mut StdVec<ProvenanceRecord>,
    pub(super) facts: SideTable<ModelFacts>,
}

fn region<'a>(
    walk: &mut PageWalk,
    channels: &mut Channels<'_>,
    env: &mut StdVec<String>,
    ops: &[Op<'a>],
) {
    for op in ops {
        visit(walk, channels, env, op);
    }
}

fn visit<'a>(
    walk: &mut PageWalk,
    channels: &mut Channels<'_>,
    env: &mut StdVec<String>,
    op: &Op<'a>,
) {
    let id = walk.mint();
    let _ = id;
    match op {
        Op::Element(element) => {
            let slot_name = owner_bindings(walk, channels, env, &element.bindings);
            let scoped = element.tag == "template" && !element.children.ops.is_empty();
            with_slot_scope(env, scoped.then_some(slot_name).flatten(), |env| {
                region(walk, channels, env, &element.children.ops);
            });
        }
        Op::Component(component) => {
            let slot_name = owner_bindings(walk, channels, env, &component.bindings);
            let scoped = !component.children.ops.is_empty();
            with_slot_scope(env, scoped.then_some(slot_name).flatten(), |env| {
                region(walk, channels, env, &component.children.ops);
            });
        }
        Op::Text(_) | Op::Interpolation(_) => {}
        Op::If(if_op) => {
            for branch in if_op.branches.iter() {
                region(walk, channels, env, &branch.region.ops);
            }
        }
        Op::For(for_op) => {
            // The legacy `enter_v_for_scope` registers the alias texts
            // (value even when it is a pattern; key and index when
            // authored) — mirrored as texts, never enumerated names.
            let before = env.len();
            let value = for_op.binding.value.source();
            if !value.is_empty() {
                env.push(String::from(value));
            }
            for expr in [&for_op.binding.key, &for_op.binding.index]
                .into_iter()
                .flatten()
            {
                env.push(String::from(expr.source()));
            }
            region(walk, channels, env, &for_op.region.ops);
            env.truncate(before);
        }
        Op::Slot(slot) => {
            // Outlets never carry `ui.model` (the lowering rejects the
            // spelling), and the legacy lane opens no slot scope on
            // them; the bindings only mint their ids here.
            for _ in slot.bindings.iter() {
                let _ = walk.mint();
            }
            region(walk, channels, env, &slot.fallback.ops);
        }
    }
}

/// Mint an owner's binding ids in page order, validate its models, and
/// return the slot-scope name its first params-bearing `ui.slot-content`
/// would open over the owner's children (the legacy
/// `enter_v_slot_scope_if_needed` reads the first `v-slot` only).
fn owner_bindings<'a>(
    walk: &mut PageWalk,
    channels: &mut Channels<'_>,
    env: &StdVec<String>,
    bindings: &[BindingOp<'a>],
) -> Option<String> {
    let mut slot_name: Option<Option<String>> = None;
    for binding in bindings {
        let id = walk.mint();
        match binding {
            BindingOp::Model(model) => check_model(channels, env, id, model),
            BindingOp::SlotContent(content) => {
                if slot_name.is_none() {
                    slot_name = Some(
                        content
                            .params
                            .as_ref()
                            .and_then(crate::lower::simple_identifier)
                            .map(String::from),
                    );
                }
            }
            BindingOp::Bind(_)
            | BindingOp::On(_)
            | BindingOp::VueDirective(_)
            | BindingOp::VueCssBind(_)
            | BindingOp::VueSync(_)
            | BindingOp::VueSlotScope(_)
            | BindingOp::VueOnce(_)
            | BindingOp::VueMemo(_)
            | BindingOp::VueShow(_)
            | BindingOp::VueHtml(_)
            | BindingOp::VueText(_) => {}
        }
    }
    slot_name.flatten()
}

/// Push a slot-scope name around `body` when one applies.
fn with_slot_scope(
    env: &mut StdVec<String>,
    name: Option<String>,
    body: impl FnOnce(&mut StdVec<String>),
) {
    match name {
        Some(name) => {
            env.push(name);
            body(env);
            env.pop();
        }
        None => body(env),
    }
}
