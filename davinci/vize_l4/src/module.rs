//! SFC module assembly: imports, hoists, script, render and exports.
//!
//! One writer produces the whole module, so one source map covers script and
//! template together. The order is fixed: the runtime import preamble (built
//! last from the used-helper set), hoisted constants and cache
//! declarations, the script body, the render function, then the default
//! export. `vize_atelier_sfc` keeps only lane selection.

#![expect(clippy::todo, reason = "skeleton: #6840")]

use vize_l0::Span;

use crate::runtime::Runtime;
use crate::write::{Emitted, LinkSink, Writer};

/// How the render function joins the component.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderPlacement {
    /// No template.
    None,
    /// A separate `render` / `ssrRender` function set on the component.
    Function,
    /// Returned from `setup()` (inline `<script setup>` templates).
    Inline,
}

/// The authored script of the SFC, already rewritten by the script lane.
#[derive(Debug, Clone, Copy)]
pub struct ScriptPart<'a> {
    /// Script text to write, in output order.
    pub text: &'a str,
    /// Its authored range, for the module source map.
    pub authored: Span,
}

/// The pieces one SFC module is assembled from.
#[derive(Debug)]
pub struct ModuleParts<'a, L: LinkSink> {
    /// The runtime whose helpers the preamble imports.
    pub runtime: Runtime,
    /// Script and `<script setup>` output, if any.
    pub script: Option<ScriptPart<'a>>,
    /// The target's render body, including its used helpers.
    pub render: Option<Writer<L>>,
    /// Where the render function goes.
    pub placement: RenderPlacement,
    /// Hoisted constant right-hand sides, numbered `_hoisted_1…` in order.
    pub hoists: &'a [&'a str],
    /// The component's export name (`_sfc_main`, or the default export).
    pub component: &'a str,
}

/// Assemble one SFC module into a single output with one link set.
#[must_use]
pub fn assemble<L: LinkSink>(_parts: ModuleParts<'_, L>) -> Emitted<L> {
    todo!("#6840: assemble preamble, hoists, script, render and exports")
}
