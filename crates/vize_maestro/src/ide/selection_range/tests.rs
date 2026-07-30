use super::SelectionRangeService;

const SFC: &str = "<script setup lang=\"ts\">\nconst count = 1\n</script>\n\n<template>\n  <div class=\"wrap\">{{ count }}</div>\n</template>\n";

/// Flatten a selection-range chain into `(start_line, start_char, end_line, end_char)`
/// tuples, innermost first, so tests can assert the FULL chain in one
/// `assert_eq!` instead of walking `parent` pointers by hand.
fn chain(content: &str, filename: &str, line: u32, character: u32) -> Vec<(u32, u32, u32, u32)> {
    let offset = crate::ide::position_to_offset(content, line, character)
        .expect("test position must resolve to a byte offset");
    let mut node = SelectionRangeService::selection_range(content, filename, offset);
    let mut flattened = Vec::new();
    while let Some(current) = node {
        flattened.push((
            current.range.start.line,
            current.range.start.character,
            current.range.end.line,
            current.range.end.character,
        ));
        node = current.parent.map(|parent| *parent);
    }
    flattened
}

#[test]
fn interpolation_identifier_expands_through_template_and_block_levels() {
    // Cursor inside `count` of `{{ count }}` on line 5.
    assert_eq!(
        chain(SFC, "/App.vue", 5, 24),
        vec![
            (5, 23, 5, 28), // the `count` token / trimmed interpolation expression
            (5, 20, 5, 31), // `{{ count }}` (also the div's inner content)
            (5, 2, 5, 37),  // `<div class="wrap">{{ count }}</div>`
            (4, 10, 6, 0),  // template block content
            (4, 0, 6, 11),  // `<template>` … `</template>`
            (0, 0, 7, 0),   // whole document
        ]
    );
}

#[test]
fn attribute_value_expands_value_then_attribute_then_start_tag() {
    // Cursor inside `wrap` of `class="wrap"` on line 5.
    assert_eq!(
        chain(SFC, "/App.vue", 5, 15),
        vec![
            (5, 14, 5, 18), // `wrap`
            (5, 13, 5, 19), // `"wrap"` including the quotes
            (5, 7, 5, 19),  // `class="wrap"`
            (5, 2, 5, 20),  // `<div class="wrap">`
            (5, 2, 5, 37),  // the whole element
            (4, 10, 6, 0),
            (4, 0, 6, 11),
            (0, 0, 7, 0),
        ]
    );
}

#[test]
fn script_positions_get_the_documented_narrower_chain() {
    // `<script>` interiors are raw text: markup levels are deliberately skipped,
    // so the chain is token -> block content -> block -> document.
    assert_eq!(
        chain(SFC, "/App.vue", 1, 8),
        vec![
            (1, 6, 1, 11), // `count`
            (0, 24, 2, 0), // script block content
            (0, 0, 2, 9),  // `<script …>` … `</script>`
            (0, 0, 7, 0),
        ]
    );
}

#[test]
fn nested_elements_are_emitted_innermost_first() {
    let source = "<template>\n  <ul>\n    <li><span>a</span></li>\n  </ul>\n</template>\n";
    // Cursor on the `a` text node inside `<span>`.
    assert_eq!(
        chain(source, "/App.vue", 2, 14),
        vec![
            (2, 14, 2, 15), // `a`
            (2, 8, 2, 22),  // `<span>a</span>`
            (2, 4, 2, 27),  // `<li>…</li>`
            (1, 6, 3, 2),   // `<ul>` inner content
            (1, 2, 3, 7),   // `<ul>…</ul>`
            (0, 10, 4, 0),  // template block content
            (0, 0, 4, 11),
            (0, 0, 5, 0),
        ]
    );
}

#[test]
fn void_elements_do_not_swallow_their_following_siblings() {
    // `<br>` has no close tag; a naive tag stack would keep it open and report
    // `<br>` as the parent of everything after it.
    let source = "<template>\n  <p><br><b>x</b></p>\n</template>\n";
    assert_eq!(
        chain(source, "/App.vue", 1, 12),
        vec![
            (1, 12, 1, 13), // `x`
            (1, 9, 1, 17),  // `<b>x</b>`
            (1, 5, 1, 17),  // `<p>` inner content: `<br><b>x</b>`
            (1, 2, 1, 21),  // `<p>…</p>`
            (0, 10, 2, 0),
            (0, 0, 2, 11),
            (0, 0, 3, 0),
        ],
        "`<br>` must never appear as a parent of the `<b>` element that follows it"
    );
}

#[test]
fn directive_expression_expands_to_the_attribute_then_the_start_tag() {
    let source = "<template>\n  <button @click=\"bump(1)\">go</button>\n</template>\n";
    // Cursor inside `bump` of `@click="bump(1)"`.
    assert_eq!(
        chain(source, "/App.vue", 1, 20),
        vec![
            (1, 18, 1, 22), // `bump`
            (1, 18, 1, 25), // `bump(1)`
            (1, 17, 1, 26), // `"bump(1)"` including the quotes
            (1, 10, 1, 26), // `@click="bump(1)"`
            (1, 2, 1, 27),  // `<button @click="bump(1)">`
            (1, 2, 1, 38),  // the whole element
            (0, 10, 2, 0),
            (0, 0, 2, 11),
            (0, 0, 3, 0),
        ]
    );
}

#[test]
fn html_comments_are_a_selection_level() {
    let source = "<template>\n  <!-- keep me -->\n</template>\n";
    assert_eq!(
        chain(source, "/App.vue", 1, 9),
        vec![
            (1, 7, 1, 11), // `keep`
            (1, 2, 1, 18), // `<!-- keep me -->`
            (0, 10, 2, 0),
            (0, 0, 2, 11),
            (0, 0, 3, 0),
        ]
    );
}

#[test]
fn every_chain_link_strictly_contains_the_previous_one() {
    // Structural invariant of the LSP contract: parents must strictly enclose
    // their child. Sweep every offset of the fixture rather than trusting the
    // hand-picked positions above.
    for offset in 0..=SFC.len() {
        let Some(root) = SelectionRangeService::selection_range(SFC, "/App.vue", offset) else {
            continue;
        };
        let mut node = Some(root);
        let mut previous: Option<(u32, u32, u32, u32)> = None;
        while let Some(current) = node {
            let span = (
                current.range.start.line,
                current.range.start.character,
                current.range.end.line,
                current.range.end.character,
            );
            if let Some(child) = previous {
                assert!(
                    (span.0, span.1) <= (child.0, child.1)
                        && (span.2, span.3) >= (child.2, child.3),
                    "offset {offset}: parent {span:?} must enclose child {child:?}"
                );
                assert_ne!(
                    span, child,
                    "offset {offset}: duplicate chain link {span:?}"
                );
            }
            previous = Some(span);
            node = current.parent.map(|parent| *parent);
        }
    }
}

#[test]
fn standalone_petite_vue_html_scans_the_whole_document_as_markup() {
    let source = "<div v-scope=\"{ count: 0 }\">{{ count }}</div>\n";
    assert_eq!(
        chain(source, "/index.html", 0, 32),
        vec![
            (0, 31, 0, 36), // `count`
            (0, 28, 0, 39), // `{{ count }}`
            (0, 0, 0, 45),  // the whole element
            (0, 0, 1, 0),   // whole document
        ]
    );
}
