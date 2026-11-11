use super::{NoIrregularWhitespace, lint_sfc};

/// `(line, character count)` of each reported run, in source order.
fn runs(source: &str) -> Vec<(usize, usize)> {
    let result = lint_sfc(Box::new(NoIrregularWhitespace), source);
    result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            let before = source.get(..diagnostic.start as usize).unwrap_or_default();
            let run = source
                .get(diagnostic.start as usize..diagnostic.end as usize)
                .unwrap_or_default();
            (before.matches('\n').count() + 1, run.chars().count())
        })
        .collect()
}

#[test]
fn irregular_whitespace_scans_script_and_template_outside_strings() {
    let source = concat!(
        "<script setup>\nconst a =\u{a0}1; //\u{a0}note\nconst s = 'x\u{a0}y';\n</script>\n",
        "\u{a0}\n",
        "<template>\n<p title=\"x\u{a0}y\">{{\u{a0}a +\u{a0}\u{a0}1 }}{{ 'x\u{a0}y' }}x\u{a0}y</p>",
        "<!--\u{a0}--><b :title=\"'x\u{a0}y' + `x\u{a0}y`\" @click=\"() => run('x\u{a0}y')\" />\n</template>\n",
        "<style>.a { color:\u{a0}red }</style>\n",
    );
    // Line 7 holds the element, the HTML comment, and the bound attribute.
    // The comment and the template literal are reported; string literals are not.
    assert_eq!(
        runs(source),
        [
            (2, 1),
            (2, 1),
            (7, 1),
            (7, 1),
            (7, 2),
            (7, 1),
            (7, 1),
            (7, 1)
        ],
        "{source:?}"
    );
}

#[test]
fn irregular_whitespace_does_not_scan_between_script_blocks() {
    let source = concat!(
        "<script>const a =\u{a0}1</script>\n",
        "<template><p>x\u{a0}y</p></template>\n",
        "<script setup>const b =\u{a0}2</script>\n",
    );
    assert_eq!(runs(source), [(1, 1), (2, 1), (3, 1)]);
}
