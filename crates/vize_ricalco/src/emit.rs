//! S2 → DOM render-function emission (P2-11).
//!
//! The unpublished home for the new DOM backend: `vize_atelier_dom` is
//! published and cannot name this crate in its release graph (the
//! installment-1 publish-gate measurement). Dual-run lives in
//! atelier_dom **test space** as a stripped-on-publish dev-dep, the
//! P2-9 carve-out. This module writes the JS string **directly from
//! S2 ops** — it does not mint relief codegen-nodes (`NodeType` 13–20).
//!
//! Installment 5 emits **static native HTML**, interpolations,
//! mixed text siblings, and **static-name `ui.bind`** (`:class` /
//! `:style` / `:id`, patch flags). Object-spread `v-bind`, events,
//! filters, and components stay [`EmitError::Unsupported`]. The old lane
//! stays the shipped compile path; [`super::DOM_LANE_FLAG`] is named
//! here and *read* in the atelier_dom witness.

#[path = "emit/buf.rs"]
mod buf;
#[path = "emit/children.rs"]
mod children;
#[path = "emit/js.rs"]
mod js;
#[path = "emit/props.rs"]
mod props;
#[path = "emit/vnode.rs"]
mod vnode;

use vize_carton::{Allocator, String};
use vize_davinci::diagnostic::Severity;
use vize_davinci::pass::BudgetObserver;
use vize_sinopia::parse;

use crate::lower::{Lowered, lower};
use crate::pass::walk::PageWalk;
use crate::pass::{S2Facts, run_transform};

use self::buf::Buf;
use self::vnode::emit_root;

/// Per-emit numbering + helper buffer. Page-order ids re-derive the
/// same arithmetic the S2 passes use so compound text facts resolve.
struct EmitCx<'facts> {
    buf: Buf,
    facts: &'facts S2Facts,
    walk: PageWalk,
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
    };
    cx.buf
        .push("function render(_ctx, _cache, $props, $setup, $data, $options) {");
    cx.buf.indent();
    cx.buf.newline();
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
