use super::compute_raw_line_mask;

fn mask(source: &str) -> Vec<bool> {
    let lines: Vec<&[u8]> = source.lines().map(|l| l.as_bytes()).collect();
    compute_raw_line_mask(&lines)
}

#[test]
fn lines_inside_an_interpolation_template_literal_are_raw() {
    // Every byte between the backticks is part of the string's runtime
    // value, so the SFC layer must not indent these lines (#3334).
    let source = "<div>\n  {{ items.map((p) => `\n${p} {\n  --a: b;\n}\n`).join(\"\") }}\n</div>";
    // Lines 2..=5 all *begin* inside the literal — including the line
    // that closes it, whose leading bytes would otherwise be indented
    // into the string's value.
    assert_eq!(mask(source), [false, false, true, true, true, true, false]);
}

#[test]
fn ordinary_interpolation_lines_stay_indentable() {
    let source = "<div>\n  {{ a\n    + b }}\n</div>";
    assert_eq!(mask(source), [false, false, false, false]);
}

#[test]
fn a_closing_brace_pair_inside_a_string_does_not_end_the_interpolation() {
    // The `}}` lives in a JS string, so the literal that follows it is
    // still inside the interpolation and its lines stay raw.
    let source = "<div>\n  {{ \"}}\" + `\nfoo\n` }}\n</div>";
    assert_eq!(mask(source), [false, false, true, true, false]);
}

#[test]
fn a_template_literal_nested_in_a_substitution_is_tracked() {
    let source = "<div>\n  {{ `${ xs.map((x) => `\na\n`) }\nb\n` }}\n</div>";
    assert_eq!(mask(source), [false, false, true, true, true, true, false]);
}

#[test]
fn a_backtick_inside_a_comment_is_not_structural() {
    let source = "<div>\n  {{ /* ` */ a }}\n  <span>y</span>\n</div>";
    assert_eq!(mask(source), [false, false, false, false]);
}

#[test]
fn an_unterminated_marker_stays_text_and_keeps_markup_tracking() {
    // Vue treats a `{{` with no `}}` as text, so the scanner must not
    // enter expression mode and lose the `<pre>` region behind it.
    let source = "<div>\n  {{ mustache syntax, unclosed\n  <pre>\nx\n  </pre>\n</div>";
    assert_eq!(
        mask(source),
        [false, false, false, true, true, false],
        "an unmatched `{{{{` must not disable markup scanning"
    );
}

#[test]
fn an_unbalanced_quote_does_not_swallow_later_lines() {
    // `'a) }}` leaves a quote open at the line end; strings cannot span a
    // newline, so the literal on the following lines is still tracked.
    let source = "<div>\n  {{ t('a) }}\n  {{ `\nx\n` }}\n</div>";
    assert_eq!(mask(source), [false, false, false, true, true, false]);
}

#[test]
fn a_closed_literal_does_not_leak_into_later_lines() {
    let source = "<div>\n  {{ `x` }}\n  <span>y</span>\n</div>";
    assert_eq!(mask(source), [false, false, false, false]);
}
