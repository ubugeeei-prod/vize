use super::LinkedEditingService;

const SFC: &str = "<script setup lang=\"ts\">\nconst count = 1\n</script>\n\n<template>\n  <div class=\"wrap\">{{ count }}</div>\n</template>\n";

/// Flatten to `(start_line, start_char, end_line, end_char)` pairs so tests can
/// assert the FULL response in one `assert_eq!`.
fn ranges(content: &str, filename: &str, line: u32, character: u32) -> Vec<(u32, u32, u32, u32)> {
    let offset = crate::ide::position_to_offset(content, line, character)
        .expect("test position must resolve to a byte offset");
    let Some(result) = LinkedEditingService::ranges(content, filename, offset) else {
        return Vec::new();
    };
    assert_eq!(
        result.word_pattern, None,
        "the reference server sends no wordPattern; the client supplies its own"
    );
    result
        .ranges
        .into_iter()
        .map(|range| {
            (
                range.start.line,
                range.start.character,
                range.end.line,
                range.end.character,
            )
        })
        .collect()
}

#[test]
fn open_tag_name_links_to_its_close_tag_name() {
    // Recorded from @vue/language-server 3.3.8 for the same fixture/position:
    // {"ranges":[{"start":{"line":5,"character":3},"end":{"line":5,"character":6}},
    //            {"start":{"line":5,"character":33},"end":{"line":5,"character":36}}]}
    assert_eq!(
        ranges(SFC, "/App.vue", 5, 4),
        vec![(5, 3, 5, 6), (5, 33, 5, 36)]
    );
}

#[test]
fn close_tag_name_links_back_to_the_open_tag_name() {
    assert_eq!(
        ranges(SFC, "/App.vue", 5, 34),
        vec![(5, 3, 5, 6), (5, 33, 5, 36)]
    );
}

#[test]
fn the_caret_just_after_a_tag_name_still_links() {
    // Where the editor leaves the caret while the name is being typed.
    assert_eq!(
        ranges(SFC, "/App.vue", 5, 6),
        vec![(5, 3, 5, 6), (5, 33, 5, 36)]
    );
}

#[test]
fn positions_that_are_not_tag_names_link_nothing() {
    for (line, character, what) in [
        (5, 2, "the `<` itself"),
        (5, 9, "an attribute name"),
        (5, 15, "an attribute value"),
        (5, 24, "an interpolation identifier"),
        (1, 8, "a script identifier"),
        (0, 4, "the `<script>` block tag name"),
    ] {
        assert_eq!(
            ranges(SFC, "/App.vue", line, character),
            Vec::<(u32, u32, u32, u32)>::new(),
            "{what} must not report linked ranges"
        );
    }
}

#[test]
fn nested_same_name_elements_link_the_innermost_pair() {
    let source = "<template>\n  <div><div>x</div></div>\n</template>\n";
    // Cursor on the inner `<div>` name at character 8.
    assert_eq!(
        ranges(source, "/App.vue", 1, 8),
        vec![(1, 8, 1, 11), (1, 15, 1, 18)]
    );
    // Cursor on the outer `<div>` name at character 3.
    assert_eq!(
        ranges(source, "/App.vue", 1, 3),
        vec![(1, 3, 1, 6), (1, 21, 1, 24)]
    );
}

#[test]
fn the_template_block_tag_is_itself_a_linkable_pair() {
    // `<template>`/`</template>` are markup; renaming one without the other
    // would break the SFC, so they link like any other element.
    assert_eq!(
        ranges(SFC, "/App.vue", 4, 3),
        vec![(4, 1, 4, 9), (6, 2, 6, 10)]
    );
}

#[test]
fn self_closing_and_void_elements_have_nothing_to_link() {
    let source = "<template>\n  <Child />\n  <br>\n  <img src=\"a.png\">\n</template>\n";
    for (line, character) in [(1, 4), (2, 3), (3, 4)] {
        assert_eq!(
            ranges(source, "/App.vue", line, character),
            Vec::<(u32, u32, u32, u32)>::new(),
            "line {line} has no close tag to link"
        );
    }
}

#[test]
fn a_component_tag_links_across_lines() {
    let source =
        "<template>\n  <MyCard\n    title=\"a\"\n  >\n    body\n  </MyCard>\n</template>\n";
    assert_eq!(
        ranges(source, "/App.vue", 1, 5),
        vec![(1, 3, 1, 9), (5, 4, 5, 10)]
    );
}

#[test]
fn a_script_block_is_never_scanned_as_markup() {
    // `a < b` in a script body must not look like a tag.
    let source = "<script setup lang=\"ts\">\nconst ok = 1 < 2\n</script>\n<template><div>x</div></template>\n";
    assert_eq!(
        ranges(source, "/App.vue", 1, 14),
        Vec::<(u32, u32, u32, u32)>::new()
    );
    assert_eq!(
        ranges(source, "/App.vue", 3, 12),
        vec![(3, 11, 3, 14), (3, 18, 3, 21)]
    );
}

#[test]
fn standalone_petite_vue_html_links_tag_pairs_across_the_whole_file() {
    let source = "<div v-scope=\"{ n: 0 }\"><span>{{ n }}</span></div>\n";
    assert_eq!(
        ranges(source, "/index.html", 0, 26),
        vec![(0, 25, 0, 29), (0, 39, 0, 43)]
    );
}
