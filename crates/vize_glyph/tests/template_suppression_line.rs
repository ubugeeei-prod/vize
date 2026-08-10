//! A line covered by a line-scoped lint suppression must survive formatting on
//! one line. `eslint-disable-next-line` is physical-line scoped, so breaking the
//! line it covers silently disables the user's suppression and a finding that
//! was suppressed in the source reappears after `vize fmt`. (#3343)

use vize_glyph::{FormatOptions, format_template};

/// Assert `source` formats to `expected`, and that a second pass is a no-op.
fn assert_formatted(source: &str, expected: &str) {
    let options = FormatOptions::default();
    let first = format_template(source, &options).unwrap();
    assert_eq!(first.as_str(), expected);
    let second = format_template(&first, &options).unwrap();
    assert_eq!(
        second.as_str(),
        first.as_str(),
        "joining a suppressed line must be idempotent"
    );
}

#[test]
fn eslint_disable_next_line_keeps_text_and_element_together() {
    // The gogocode fixture from the report: every text run used to be emitted as
    // its own line, so the `<input>` moved off the line the comment covers and
    // `a11y/form-control-has-label` went from 2 to 3 findings after formatting.
    assert_formatted(
        concat!(
            "<div class=\"mt20\">\n",
            "  <!--eslint-disable-next-line-->\n",
            "  custom space: <input v-on:keyup.custom=\"keys('custom keycode space')\" />\n",
            "</div>",
        ),
        concat!(
            "<div class=\"mt20\">\n",
            "  <!--eslint-disable-next-line-->\n",
            "  custom space: <input @keyup.custom='keys(\"custom keycode space\")' />\n",
            "</div>",
        ),
    );
}

#[test]
fn suppression_pragma_variants_all_pin_the_next_line() {
    // Inner spaces, a rule-scoped list, a trailing `--` reason, and this repo's
    // own `vize-`/`@vize:` next-line directives must all pin the following line.
    // Missing one leaves the defect in place for that spelling.
    for source in [
        "<div>\n  <!--eslint-disable-next-line-->\n  label: <input />\n</div>",
        "<div>\n  <!-- eslint-disable-next-line -->\n  label: <input />\n</div>",
        "<div>\n  <!--  eslint-disable-next-line  -->\n  label: <input />\n</div>",
        "<div>\n  <!-- eslint-disable-next-line a11y/form-control-has-label -->\n  label: <input />\n</div>",
        "<div>\n  <!-- eslint-disable-next-line a11y/form-control-has-label -- why -->\n  label: <input />\n</div>",
        "<div>\n  <!-- vize-disable-next-line a11y/form-control-has-label -->\n  label: <input />\n</div>",
        "<div>\n  <!-- @vize:expected -->\n  label: <input />\n</div>",
        "<div>\n  <!-- @vize:level(off) -->\n  label: <input />\n</div>",
    ] {
        assert_formatted(source, source);
    }
}

#[test]
fn joined_chunks_reuse_the_source_spacing() {
    // Whitespace between the joined chunks is replayed as a single space, and its
    // absence is preserved, so re-joining cannot change what Vue renders.
    assert_formatted(
        "<div>\n  <!-- eslint-disable-next-line -->\n  <b>a</b><i>b</i>\n</div>",
        "<div>\n  <!-- eslint-disable-next-line -->\n  <b>a</b><i>b</i>\n</div>",
    );
    assert_formatted(
        "<div>\n  <!-- eslint-disable-next-line -->\n  <b>a</b>   <i>b</i>\n</div>",
        "<div>\n  <!-- eslint-disable-next-line -->\n  <b>a</b> <i>b</i>\n</div>",
    );
    // Interpolations, closing tags and trailing text all stay on the line too.
    assert_formatted(
        "<p>\n  <!-- eslint-disable-next-line -->\n  a {{x}} <b>c</b> d\n</p>",
        "<p>\n  <!-- eslint-disable-next-line -->\n  a {{ x }} <b>c</b> d\n</p>",
    );
}

#[test]
fn ordinary_same_line_spacing_is_preserved_without_a_suppression() {
    // Same-line whitespace is a Vue text node even without a suppression.
    // Formatting may canonicalize its width but must not turn it into an
    // inter-tag newline that the compiler drops.
    assert_formatted(
        "<div>\n  space: <input type=\"text\" />\n</div>",
        "<div>\n  space: <input type=\"text\" />\n</div>",
    );
    // An unrelated comment pins nothing.
    assert_formatted(
        "<div>\n  <!-- a note -->\n  space: <input />\n</div>",
        "<div>\n  <!-- a note -->\n  space: <input />\n</div>",
    );
    // Neither does a block `eslint-disable`: it is not line scoped, so it
    // survives reflow on its own and needs no pinning.
    assert_formatted(
        "<div>\n  <!-- eslint-disable -->\n  space: <input />\n</div>",
        "<div>\n  <!-- eslint-disable -->\n  space: <input />\n</div>",
    );
    // A pragma two lines up covers neither the blank line nor the code below it.
    assert_formatted(
        "<div>\n  <!-- eslint-disable-next-line -->\n\n  space: <input />\n</div>",
        "<div>\n  <!-- eslint-disable-next-line -->\n  space: <input />\n</div>",
    );
}

#[test]
fn same_line_pragma_keeps_its_own_line_intact() {
    // `eslint-disable-line` covers the line the comment sits on, so the comment
    // must not be pushed onto a line of its own either.
    let source = "<div>\n  label: <input /> <!-- eslint-disable-line -->\n</div>";
    assert_formatted(source, source);
}

#[test]
fn pinned_line_does_not_leak_into_neighbours() {
    // Only the covered line is pinned; ordinary same-line text spacing remains
    // intact independently, and nesting still indents its children.
    assert_formatted(
        concat!(
            "<div>\n",
            "  before: <input />\n",
            "  <!-- eslint-disable-next-line -->\n",
            "  pinned: <input /><span>\n",
            "  inner: <input />\n",
            "  </span>\n",
            "  after: <input />\n",
            "</div>",
        ),
        concat!(
            "<div>\n",
            "  before: <input />\n",
            "  <!-- eslint-disable-next-line -->\n",
            "  pinned: <input /><span>\n",
            "    inner: <input />\n",
            "  </span>\n",
            "  after: <input />\n",
            "</div>",
        ),
    );
}

#[test]
fn multiline_directive_value_on_a_pinned_line_stays_idempotent() {
    // A directive value that is multiline in the source still spans lines, and
    // its continuation indentation is anchored the same way on every pass even
    // though the tag now starts mid-line (#3368 interaction).
    let source = concat!(
        "<div>\n",
        "  <!-- eslint-disable-next-line -->\n",
        "  label: <span :style=\"{\n",
        "    color: 'red',\n",
        "  }\">x</span>\n",
        "</div>",
    );
    let options = FormatOptions::default();
    let first = format_template(source, &options).unwrap();
    let second = format_template(&first, &options).unwrap();
    assert_eq!(
        second.as_str(),
        first.as_str(),
        "multiline directive value on a pinned line must be idempotent"
    );
    assert!(
        first.contains("label: <span"),
        "the element must still join the suppressed line:\n{first}"
    );
}

#[test]
fn whitespace_significant_element_on_a_pinned_line_stays_verbatim() {
    // `<pre>` content is still copied byte for byte (#963) while the element
    // itself joins the suppressed line.
    let source = "<div>\n  <!-- eslint-disable-next-line -->\n  a <pre>  x  </pre> b\n</div>";
    assert_formatted(source, source);
}
