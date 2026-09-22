//! vue/permitted-contents
//!
//! Report nesting the HTML standard forbids, as proven by the exact checker in
//! [`crate::html_content_model`] (Davinci P4-11a, precision tier `exact`).
//!
//! Two families of violations:
//!
//! 1. **Parser** — the HTML parser would not build the element where the
//!    template puts it, so the browser's DOM differs from the virtual DOM
//!    (SSR hydration mismatches, `innerHTML`-built static content renders
//!    differently): `<div>` closes an open `<p>`, `<tr>` gets an implied
//!    `<tbody>`, a `<div>` in a `<table>` is foster-parented, `<span>` breaks
//!    out of `<svg>`, a nested `<a>` or `<form>` is adopted or dropped.
//! 2. **Content model** — the DOM is built as written but a content model
//!    forbids it: `<div>` in `<span>`, `<button>` in `<a>`, `<div>` in `<ul>`.
//!
//! Every report is a proven fact: an unresolved component, a slot, a dynamic
//! binding or the unknown place a component is mounted makes a verdict
//! unknown, and unknown is silence. The composed cross-component check
//! (P4-11b) resolves what a single file cannot see.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <template>
//!   <p><div>block in a paragraph</div></p>
//!   <table><tr><td>row without tbody</td></tr></table>
//!   <a href="#"><button>nested control</button></a>
//!   <ul><div>not a list item</div></ul>
//! </template>
//! ```
//!
//! ### Valid
//! ```vue
//! <template>
//!   <p><span>inline in a paragraph</span></p>
//!   <table><tbody><tr><td>cell</td></tr></tbody></table>
//!   <ul><li>list item</li><MyItem /></ul>
//! </template>
//! ```

use crate::context::LintContext;
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::html_content_model::{
    Context, Family, NodeKind, Skeleton, ViolationClass, check, skeleton, template_skeleton,
};
use crate::markup::{MarkupContext, MarkupDocument, MarkupRule};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::RootNode;
use vize_s0::{CompactString, cstr};

static META: RuleMeta = RuleMeta {
    name: "vue/permitted-contents",
    description: "Enforce HTML content model rules",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

#[derive(Default)]
pub struct PermittedContents;

/// `<tag>` for an element, `text` for text, with the span a diagnostic
/// points at (an element's start tag up to the end of its name).
fn describe(skeleton: &Skeleton, index: u32) -> (CompactString, u32, u32) {
    let node = skeleton.node(index);
    match &node.kind {
        NodeKind::Element(element) => (
            cstr!("<{}>", element.tag),
            node.span.start,
            element.name_span.end,
        ),
        _ => (CompactString::new("text"), node.span.start, node.span.end),
    }
}

/// Report every proven violation of a skeleton checked with an unknown
/// mount point.
fn report_skeleton(ctx: &mut LintContext<'_>, skeleton: &Skeleton) {
    let report = check(skeleton, 0, &Context::Truncated);
    for (node, class, evidence) in report.findings() {
        let (child, start, end) = describe(skeleton, node);
        let parent = evidence.map(|(_, index)| describe(skeleton, index));
        let parent_name = parent
            .as_ref()
            .map_or(CompactString::new(""), |(name, ..)| name.clone());
        let message = ctx.t_fmt(
            &cstr!("vue/permitted-contents.{}", class.id()),
            &[("child", child.as_str()), ("parent", parent_name.as_str())],
        );
        let mut diagnostic = LintDiagnostic::error(ctx.current_rule, message, start, end);
        if let Some((name, label_start, label_end)) = parent {
            let label = ctx.t_fmt(
                "vue/permitted-contents.evidence",
                &[("parent", name.as_str())],
            );
            diagnostic = diagnostic.with_label(label, label_start, label_end);
        }
        let help = ctx.t(help_key(class));
        if let Some(help) = ctx.help_level().process(help.as_ref()) {
            diagnostic = diagnostic.with_help(help);
        }
        ctx.report(diagnostic);
    }
}

fn help_key(class: ViolationClass) -> &'static str {
    match class.family() {
        Family::Parser => "vue/permitted-contents.help.parser",
        Family::ContentModel => "vue/permitted-contents.help.content-model",
    }
}

impl MarkupRule for PermittedContents {
    fn name(&self) -> &'static str {
        META.name
    }

    /// The markup lane (lowered JSX): JSX is lowered without tree
    /// construction repair, so the document already is the authored tree.
    fn enter_document(&self, ctx: &mut MarkupContext<'_, '_>, document: &MarkupDocument) {
        let skeleton = skeleton(document);
        report_skeleton(ctx.lint(), &skeleton);
    }
}

impl Rule for PermittedContents {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn jsx_needs_lowering(&self) -> bool {
        true
    }

    /// Templates keep `run_on_template`: it re-reads the template as
    /// authored, which the facade over the repaired lint parse is not.
    fn markup_on_templates(&self) -> bool {
        false
    }

    /// The template lane: the checker reads the template as authored —
    /// the linter's own parse when it repaired nothing, a re-read with the
    /// non-repairing syntax otherwise.
    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, root: &RootNode<'a>) {
        let skeleton = template_skeleton(ctx.allocator(), ctx.source, root);
        report_skeleton(ctx, &skeleton);
    }
}

#[cfg(test)]
mod tests;
