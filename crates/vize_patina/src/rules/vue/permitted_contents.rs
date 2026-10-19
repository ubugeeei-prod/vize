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
//!    differently): `<div>` closes an open `<p>`, a `<div>` in a `<table>` is
//!    foster-parented, `<span>` breaks out of `<svg>`, a nested `<a>` or
//!    `<form>` is adopted or dropped. `<tr>` directly inside `<table>` is not
//!    reported: an SFC template never goes through the HTML parser, so Vue
//!    renders that pair as written and no `<tbody>` is implied.
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
//!   <table><tr><td>row</td></tr></table>
//!   <ul><li>list item</li><MyItem /></ul>
//! </template>
//! ```

use crate::context::LintContext;
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::html_content_model::{
    Context, Family, NodeKind, Skeleton, ViolationClass, authored_document_skeleton, check,
    template_skeleton,
};
use crate::markup::{MarkupContext, MarkupDocument, MarkupRule};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_l0::{CompactString, cstr};
use vize_relief::RootNode;

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
        // `<tr>` in `<table>` is legal content. The HTML parser would insert
        // `<tbody>`, but an SFC template is not parsed that way (#7211).
        if class == ViolationClass::TableWrapperInserted
            && child.eq_ignore_ascii_case("<tr>")
            && parent_name.eq_ignore_ascii_case("<table>")
        {
            continue;
        }
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

    /// SFC reads the authored L1 projection retained by its L2 document.
    /// JSX is lowered without HTML repair and already is authored.
    fn enter_document(&self, ctx: &mut MarkupContext<'_, '_>, document: &MarkupDocument) {
        let skeleton = authored_document_skeleton(document);
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

    /// The template lane: the checker reads the template as authored —
    /// the linter's own parse when it repaired nothing, a re-read with the
    /// non-repairing syntax otherwise.
    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, root: &RootNode<'a>) {
        let skeleton = template_skeleton(ctx.allocator(), ctx.source, root);
        report_skeleton(ctx, &skeleton);
    }
}

#[cfg(test)]
mod tests {
    use super::PermittedContents;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(PermittedContents));
        Linter::with_registry(registry)
    }

    /// `start..end message [label start..end label]` per diagnostic.
    fn template(source: &str) -> Vec<String> {
        let result = create_linter().lint_template_rules_only(source, "test.vue");
        render(&result.diagnostics)
    }

    fn render(diagnostics: &[crate::LintDiagnostic]) -> Vec<String> {
        diagnostics
            .iter()
            .map(|diagnostic| {
                let labels: Vec<String> = diagnostic
                    .labels
                    .iter()
                    .map(|label| format!(" [{}..{} {}]", label.start, label.end, label.message))
                    .collect();
                format!(
                    "{}..{} {}{}",
                    diagnostic.start,
                    diagnostic.end,
                    diagnostic.message,
                    labels.concat()
                )
            })
            .collect()
    }

    const NONE: [&str; 0] = [];

    #[test]
    fn conforming_templates_stay_silent() {
        for source in [
            r#"<p><span>text</span></p>"#,
            r#"<div><p>text</p></div>"#,
            r#"<ul><li>item</li><MyItem /><template v-for="i in l"><li>{{ i }}</li></template></ul>"#,
            r#"<table><thead><tr><th>Head</th></tr></thead><tbody><tr><td>Cell</td></tr></tbody></table>"#,
            r#"<table><MyRow /></table>"#,
            r#"<p><MyComponent /></p>"#,
            r#"<ul><motion.li>item</motion.li></ul>"#,
            r##"<main><a href="#"><h2>Documentation</h2><div>Read the guide</div></a></main>"##,
            r#"<select><option>A</option><optgroup label="G"><option>B</option></optgroup></select>"#,
            r#"<details><summary>Options</summary><fieldset><label><input type="checkbox" />Enabled</label><select><option>A</option></select><button>Apply</button></fieldset></details>"#,
        ] {
            assert_eq!(template(source), NONE, "{source}");
        }
    }

    /// Former false positives (TS-38 triage): each is conforming under the
    /// pinned WHATWG snapshot.
    #[test]
    fn former_false_positives_are_fixed() {
        for source in [
            // `select` permits `div` wrappers since the customizable-select change.
            r#"<select><div>option group</div></select>"#,
            // A label's first labelable descendant is its labeled control.
            r#"<label>Pick <select><option>A</option></select></label>"#,
            r#"<label><textarea></textarea></label>"#,
            // `hr` separators are select content.
            r#"<select><option>A</option><hr><option>B</option></select>"#,
        ] {
            assert_eq!(template(source), NONE, "{source}");
        }
    }

    #[test]
    fn parser_family_reports_name_the_closed_ancestor() {
        assert_eq!(
            template(r#"<p><div>block</div></p>"#),
            [
                "3..7 <div> closes the open <p>: the browser ends the paragraph before it, so the rendered DOM will not match this template [0..2 <p> is open here]"
            ]
        );
        assert_eq!(
            template(r#"<p><h1>heading</h1></p>"#),
            [
                "3..6 <h1> closes the open <p>: the browser ends the paragraph before it, so the rendered DOM will not match this template [0..2 <p> is open here]"
            ]
        );
        assert_eq!(
            template(r##"<p><a href="#"><div>block</div></a></p>"##),
            [
                "15..19 <div> closes the open <p>: the browser ends the paragraph before it, so the rendered DOM will not match this template [0..2 <p> is open here]"
            ]
        );
        assert_eq!(
            template(r#"<table><div>not valid</div></table>"#),
            [
                "7..11 <div> is moved out of <table>: the HTML parser places content that is not table structure before the table [0..6 <table> is open here]"
            ]
        );
        assert_eq!(
            template(r#"<table><tr><span>not td/th</span></tr></table>"#),
            NONE
        );
        assert_eq!(
            template(r#"<svg><div>html in svg</div></svg>"#),
            [
                "5..9 <div> breaks out of <svg>: the HTML parser ends SVG/MathML content at this tag [0..4 <svg> is open here]"
            ]
        );
    }

    #[test]
    fn tr_directly_inside_table_is_legal_in_an_sfc_template() {
        assert_eq!(
            template(
                r#"<table class="score-table"><tr><th>Subject</th><th>Score</th></tr><tr><td>Math</td><td>80</td></tr></table>"#
            ),
            NONE
        );
    }

    #[test]
    fn content_model_family_reports() {
        assert_eq!(
            template(r#"<span><div>block</div></span>"#),
            [
                "6..10 <div> is not phrasing content, but <span> only permits phrasing content [0..5 <span> is open here]"
            ]
        );
        assert_eq!(
            template(r#"<span><ul><li>item</li></ul></span>"#),
            [
                "6..9 <ul> is not phrasing content, but <span> only permits phrasing content [0..5 <span> is open here]"
            ]
        );
        assert_eq!(
            template(r##"<a href="#"><button>click</button></a>"##),
            [
                "12..19 <button> is interactive content and cannot be nested in <a> [0..2 <a> is open here]"
            ]
        );
        assert_eq!(
            template(r#"<ul><div>not li</div></ul>"#),
            ["4..8 <div> is not permitted as a child of <ul> [0..3 <ul> is open here]"]
        );
        assert_eq!(
            template(r#"<ol><span>not li</span></ol>"#),
            ["4..9 <span> is not permitted as a child of <ol> [0..3 <ol> is open here]"]
        );
        assert_eq!(
            template(r#"<ul><motion.div>item</motion.div></ul>"#),
            ["4..15 <motion.div> is not permitted as a child of <ul> [0..3 <ul> is open here]"]
        );
        assert_eq!(
            template(
                r##"<details><summary>Options</summary><a href="#"><button>Apply</button></a></details>"##
            ),
            [
                "47..54 <button> is interactive content and cannot be nested in <a> [35..37 <a> is open here]"
            ]
        );
    }

    /// A template root is mounted where its compiled namespace holds; beyond
    /// that, whatever depends on the mount point stays unknown and silent.
    #[test]
    fn mount_point_dependent_verdicts_stay_silent() {
        for source in [
            // The mount point may be a `<p>` (closing it) or not: unknown.
            r#"<div>block at the root</div>"#,
            // `<tr>` is valid inside a `<tbody>` the parent provides.
            r#"<tr><td>cell</td></tr>"#,
            // An unknown `foo.li` tag is outside the table's domain.
            r#"<ul><foo.li>item</foo.li></ul>"#,
        ] {
            assert_eq!(template(source), NONE, "{source}");
        }
    }

    #[test]
    fn jsx_lowered_documents_use_the_same_checker() {
        let linter = create_linter();
        for (source, expected) in [
            (r#"const view = <span><div>block</div></span>"#, 1),
            (
                r#"const view = <details><summary>Options</summary><label><input />Enabled</label><select><option>A</option></select></details>"#,
                0,
            ),
            (
                r##"const view = <details><summary>Options</summary><a href="#"><button>Apply</button></a></details>"##,
                1,
            ),
            (r#"const view = <p><div>block</div></p>"#, 1),
        ] {
            let result = linter.lint_jsx(source, "test.tsx", vize_atelier_jsx::JsxLang::Tsx);
            assert_eq!(result.error_count, expected, "{source}");
        }
    }
}
