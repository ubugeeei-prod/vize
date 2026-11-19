//! Attribute values that pug escapes or cooks keep their authored columns
//! (#13295). The compiled rendering keeps pug's exact entities; the
//! authored-order view keeps the authored bytes Vue reads the same way.

use vize_l0::Span;

use super::{PugRendering, PugTemplate, derive_template_source_with};

fn derive(source: &str, rendering: PugRendering) -> PugTemplate {
    let template = derive_template_source_with(source, rendering);
    assert!(!template.has_errors(), "{:?}", template.diagnostics);
    template
}

fn at(text: &str, needle: &str) -> u32 {
    text.find(needle)
        .unwrap_or_else(|| panic!("{needle:?} in {text:?}")) as u32
}

/// `needle` in the derived HTML maps back to `needle` in the pug.
fn assert_exact(template: &PugTemplate, source: &str, needle: &str) {
    let html = at(&template.html, needle);
    let span = Span::new(html, html + needle.len() as u32);
    let pug = at(source, needle);
    assert_eq!(
        template.map.to_pug(span),
        Span::new(pug, pug + needle.len() as u32),
        "{needle:?} in {:?}",
        template.html
    );
}

/// The derived `html_needle` maps to exactly the authored `pug_needle`.
fn assert_spelled_by(template: &PugTemplate, source: &str, html_needle: &str, pug_needle: &str) {
    let html = at(&template.html, html_needle);
    let pug = at(source, pug_needle);
    assert_eq!(
        template
            .map
            .to_pug(Span::new(html, html + html_needle.len() as u32)),
        Span::new(pug, pug + pug_needle.len() as u32),
        "{html_needle:?} in {:?}",
        template.html
    );
}

const COMPARISONS: &str =
    "Child(:title=\"a < 1 && b > 2 ? c.missing : ''\" @click=\"() => go(d.e)\")";

#[test]
fn the_view_keeps_comparison_logical_and_arrow_bytes() {
    let template = derive(COMPARISONS, PugRendering::AuthoredOrder);
    assert_eq!(
        template.html,
        "<Child :title=\"a < 1 && b > 2 ? c.missing : ''\" @click=\"() => go(d.e)\"></Child>"
    );
    for needle in ["a < 1 && b > 2 ? c.missing : ''", "() => go(d.e)"] {
        assert_exact(&template, COMPARISONS, needle);
    }
}

#[test]
fn the_compiled_rendering_keeps_pug_entities_with_exact_columns() {
    let template = derive(COMPARISONS, PugRendering::Pug);
    assert_eq!(
        template.html,
        "<Child :title=\"a &lt; 1 &amp;&amp; b &gt; 2 ? c.missing : ''\" @click=\"() =&gt; go(d.e)\"></Child>"
    );
    for needle in ["c.missing : ''", "go(d.e)", "a ", " 1 ", " b "] {
        assert_exact(&template, COMPARISONS, needle);
    }
    assert_spelled_by(&template, COMPARISONS, "&lt;", "<");
    assert_spelled_by(&template, COMPARISONS, "&gt; 2", "> 2");
    assert_spelled_by(&template, COMPARISONS, "&amp;&amp;", "&&");
}

#[test]
fn escapes_vue_reads_differently_stay_in_the_view() {
    let source = "p(title=\"x&amp y && z\" :a='\"q\" + w.v')";
    let template = derive(source, PugRendering::AuthoredOrder);
    assert_eq!(
        template.html,
        "<p title=\"x&amp;amp y && z\" :a=\"&quot;q&quot; + w.v\"></p>"
    );
    assert_spelled_by(&template, source, "&amp;amp", "&amp");
    assert_spelled_by(&template, source, "&quot;q&quot;", "\"q\"");
    for needle in [" y && z", " + w.v"] {
        assert_exact(&template, source, needle);
    }
}

#[test]
fn javascript_escapes_map_to_their_authored_spelling() {
    let source = "p(:a='go(\\'x\\') && y.z')";
    for rendering in [PugRendering::Pug, PugRendering::AuthoredOrder] {
        let template = derive(source, rendering);
        assert_spelled_by(&template, source, "'x'", "\\'x\\'");
        assert_exact(&template, source, "y.z");
    }
}

#[test]
fn unescaped_and_computed_values_render_as_before() {
    let unescaped = "p(:a!=\"b < c\")";
    for rendering in [PugRendering::Pug, PugRendering::AuthoredOrder] {
        let template = derive(unescaped, rendering);
        assert_eq!(template.html, "<p :a=\"b < c\"></p>");
        assert_exact(&template, unescaped, "b < c");
    }
    // A computed value has no authored spelling: it maps to the attribute.
    let computed = "p(title=\"a\" + \"<b\")";
    let template = derive(computed, PugRendering::Pug);
    assert_eq!(template.html, "<p title=\"a&lt;b\"></p>");
    assert_spelled_by(&template, computed, "a&lt;b", "title=\"a\" + \"<b\"");
}
