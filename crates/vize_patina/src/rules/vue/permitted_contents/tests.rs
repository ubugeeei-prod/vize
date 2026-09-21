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
        [
            "7..10 <tr> directly inside <table> gets an implied wrapper (<tbody>, <tr> or <colgroup>) from the HTML parser [0..6 <table> is open here]"
        ]
    );
    assert_eq!(
        template(r#"<svg><div>html in svg</div></svg>"#),
        [
            "5..9 <div> breaks out of <svg>: the HTML parser ends SVG/MathML content at this tag [0..4 <svg> is open here]"
        ]
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
