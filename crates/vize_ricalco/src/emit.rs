//! S2 → DOM render-function emission (P2-11).
//!
//! The unpublished home for the new DOM backend: `vize_atelier_dom` is
//! published and cannot name this crate in its release graph (the
//! installment-1 publish-gate measurement). Dual-run lives in
//! atelier_dom **test space** as a stripped-on-publish dev-dep, the
//! P2-9 carve-out. This module writes the JS string **directly from
//! S2 ops** — it does not mint relief codegen-nodes (`NodeType` 13–20).
//!
//! This installment emits **static native HTML**, interpolations,
//! mixed text siblings, static-name `ui.bind`, static-name `ui.on`
//! (including event/key/option modifiers), native `ui.if`, **native
//! `ui.for`**, **object-spread `v-bind`** (`normalizeProps` /
//! `mergeProps`), **static-name components** (`resolveComponent` /
//! `createVNode` / `createBlock`), and **object `v-on`** (`toHandlers`).
//! `.native`, template fragments, filters, slots, and builtins stay
//! [`EmitError::Unsupported`]. The old lane stays the shipped compile
//! path; [`super::DOM_LANE_FLAG`] is named here and *read* in the
//! atelier_dom witness.

#[path = "emit/buf.rs"]
mod buf;
#[path = "emit/children.rs"]
mod children;
#[path = "emit/component.rs"]
mod component;
#[path = "emit/flag.rs"]
mod flag;
#[path = "emit/helper.rs"]
mod helper;
#[path = "emit/js.rs"]
mod js;
#[path = "emit/merge.rs"]
mod merge;
#[path = "emit/on.rs"]
mod on;
#[path = "emit/props.rs"]
mod props;
#[path = "emit/vfor.rs"]
mod vfor;
#[path = "emit/vif.rs"]
mod vif;
#[path = "emit/vnode.rs"]
mod vnode;

use vize_carton::{Allocator, String};
use vize_davinci::diagnostic::Severity;
use vize_davinci::id::NodeId;
use vize_davinci::pass::BudgetObserver;
use vize_disegno::op::{ElementOp, ForOp, IfOp};
use vize_sinopia::parse;

use crate::lower::{Lowered, lower};
use crate::pass::walk::PageWalk;
use crate::pass::{S2Facts, run_transform};

use self::buf::Buf;
use self::vnode::emit_root;

fn emit_if_op(cx: &mut EmitCx<'_>, if_op: &IfOp<'_>, id: Option<NodeId>) -> Result<(), EmitError> {
    vif::emit_if(cx, if_op, id)
}

fn emit_if_branch_call(
    cx: &mut EmitCx<'_>,
    element: &ElementOp<'_>,
    key: &str,
) -> Result<(), EmitError> {
    vnode::emit_if_branch_element(cx, element, key)
}

fn emit_for_op(cx: &mut EmitCx<'_>, for_op: &ForOp<'_>) -> Result<(), EmitError> {
    vfor::emit_for(cx, for_op)
}

fn emit_for_item_call(
    cx: &mut EmitCx<'_>,
    element: &ElementOp<'_>,
    stable: bool,
) -> Result<(), EmitError> {
    vnode::emit_for_item_element(cx, element, stable)
}

/// Per-emit numbering + helper buffer. Page-order ids re-derive the
/// same arithmetic the S2 passes use so compound text facts resolve.
struct EmitCx<'facts> {
    buf: Buf,
    facts: &'facts S2Facts,
    walk: PageWalk,
    /// Sibling `v-if` chains share one counter; nested chains reset.
    if_branch_key: u32,
}

/// One DOM render module, split the way the shipped codegen splits it
/// (`CodegenResult::{preamble, code}`) so a dual-run can compare each
/// half and the concatenated form the DOM snapshots use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomEmit {
    /// Helper destructure (`const { … } = Vue\n`).
    pub preamble: String,
    /// The `function render(…)` body, no trailing newline after `}`.
    pub code: String,
}

impl DomEmit {
    /// `preamble + "\\n" + code` — the same concatenation
    /// `vize_atelier_dom` snapshots pin.
    #[must_use]
    pub fn assembled(&self) -> String {
        let mut out = self.preamble.clone();
        out.push('\n');
        out.push_str(self.code.as_str());
        out
    }
}

/// Why emission refused. Never a panic: the lowering is total, emission
/// of an unhandled shape is a counted skip.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmitError {
    /// S2 carries an error diagnostic; refuse to guess a render function.
    Diagnostics,
    /// This installment does not emit this shape.
    Unsupported,
}

/// Emit a DOM render function from an already-lowered (and typically
/// transformed) S2 artifact. `facts` is the transform product compounds
/// compile from.
pub fn emit_dom(lowered: &Lowered<'_>, facts: &S2Facts) -> Result<DomEmit, EmitError> {
    if lowered
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == Severity::Error)
    {
        return Err(EmitError::Diagnostics);
    }
    let mut cx = EmitCx {
        buf: Buf::new(),
        facts,
        walk: PageWalk::new(),
        if_branch_key: 0,
    };
    cx.buf
        .push("function render(_ctx, _cache, $props, $setup, $data, $options) {");
    cx.buf.indent();
    cx.buf.newline();
    let names = component::collect_names(&lowered.root);
    if !names.is_empty() {
        component::emit_resolves(&mut cx, &names);
        cx.buf.newline();
    }
    cx.buf.push("return ");
    emit_root(&mut cx, &lowered.root)?;
    cx.buf.deindent();
    cx.buf.newline();
    cx.buf.push("}");
    Ok(DomEmit {
        preamble: cx.buf.preamble(),
        code: cx.buf.code,
    })
}

/// Parse → lower → S2 transform → emit. The comparator's one-shot entry
/// so atelier_dom tests do not re-derive the pipeline.
pub fn emit_dom_source<'a>(
    allocator: &'a Allocator,
    source: &'a str,
) -> Result<DomEmit, EmitError> {
    let (tree, errors) = parse(allocator, source);
    let mut lowered = lower(allocator, &tree, &errors);
    let mut budget = BudgetObserver::new();
    let facts = run_transform(&mut lowered, &mut budget);
    emit_dom(&lowered, &facts)
}
