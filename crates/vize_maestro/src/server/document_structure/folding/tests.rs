use tower_lsp::lsp_types::{
    FoldingRangeKind, FoldingRangeParams, PartialResultParams, TextDocumentIdentifier, Url,
    WorkDoneProgressParams,
};

use super::folding_ranges;
use crate::server::ServerState;

/// The fixture from #3455, with the `<template>` closing on line 9.
const FOUR_DIVS: &str = "<script setup lang=\"ts\">\nconst count = 1\n</script>\n\n<template>\n  <div class=\"a\">\n    <div class=\"b\">{{ count }}</div>\n  </div>\n  <div class=\"c\" />\n</template>\n";

/// One region as `(start_line, end_line, kind, collapsed_text)`, so a test can
/// assert the FULL response in one `assert_eq!`.
type FlatRange = (u32, u32, Option<&'static str>, Option<&'static str>);

fn ranges(source: &str) -> Vec<FlatRange> {
    let uri = Url::parse("file:///App.vue").unwrap();
    let state = ServerState::new();
    state
        .documents
        .open(uri.clone(), source.to_string(), 1, "vue".to_string());

    let params = FoldingRangeParams {
        text_document: TextDocumentIdentifier { uri },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };
    let Some(ranges) = folding_ranges(&state, &params) else {
        return Vec::new();
    };

    ranges
        .into_iter()
        .map(|range| {
            assert_eq!(range.start_character, None, "block folds span lines only");
            assert_eq!(range.end_character, None, "block folds span lines only");
            let kind = match range.kind {
                Some(FoldingRangeKind::Region) => Some("region"),
                Some(FoldingRangeKind::Comment) => Some("comment"),
                Some(FoldingRangeKind::Imports) => Some("imports"),
                None => None,
            };
            let collapsed = match range.collapsed_text.as_deref() {
                Some("template") => Some("template"),
                Some("script setup") => Some("script setup"),
                Some("script") => Some("script"),
                Some("style") => Some("style"),
                Some(other) => panic!("unexpected collapsedText {other:?}"),
                None => None,
            };
            (range.start_line, range.end_line, kind, collapsed)
        })
        .collect()
}

#[test]
fn blocks_and_elements_stop_one_line_before_their_closing_tag() {
    // `@vue/language-server` 3.3.8 reports exactly these three regions for this
    // fixture: {5,6} (the multi-line `<div class="a">`), {0,1} (script setup)
    // and {4,8} (template). Before #3455 Maestro reported only the two block
    // regions, and both ran one line too far: 0..2 and 4..9, folding away
    // `</script>` and `</template>` themselves.
    assert_eq!(
        ranges(FOUR_DIVS),
        vec![
            (4, 8, Some("region"), Some("template")),
            (0, 1, Some("region"), Some("script setup")),
            // The inner `<div class="b">` opens and closes on line 6 and the
            // `<div class="c" />` is self-closing: neither hides a line.
            (5, 6, None, None),
        ]
    );
}

#[test]
fn a_block_with_nothing_between_its_tags_produces_no_region() {
    // `endLine` is the last hidden line, so a region would have to hide the
    // closing tag to be non-empty here.
    assert_eq!(
        ranges("<template>\n</template>\n<style>\n</style>\n"),
        Vec::new()
    );
}

#[test]
fn a_multi_line_comment_folds_as_a_comment_region() {
    let source = "<template>\n  <!--\n    note\n  -->\n  <div>\n    x\n  </div>\n</template>\n";
    assert_eq!(
        ranges(source),
        vec![
            (0, 6, Some("region"), Some("template")),
            (1, 2, Some("comment"), None),
            (4, 5, None, None),
        ]
    );
}

#[test]
fn nested_elements_fold_independently_in_document_order() {
    let source = "<template>\n  <section>\n    <ul>\n      <li>a</li>\n    </ul>\n  </section>\n</template>\n";
    assert_eq!(
        ranges(source),
        vec![
            (0, 5, Some("region"), Some("template")),
            (1, 4, None, None),
            (2, 3, None, None),
        ]
    );
}

#[test]
fn an_unclosed_element_folds_nothing_and_does_not_steal_the_outer_close_tag() {
    // `<span>` never closes. The nearest matching open tag wins for `</div>`,
    // so the `<div>` still folds and the `<span>` contributes no region.
    let source = "<template>\n  <div>\n    <span>a\n  </div>\n</template>\n";
    assert_eq!(
        ranges(source),
        vec![(0, 3, Some("region"), Some("template")), (1, 2, None, None),]
    );
}

#[test]
fn a_script_body_that_looks_like_markup_is_never_scanned() {
    // Only the template block is scanned for elements, so `a < b` in a script
    // body cannot open a phantom element.
    let source = "<script setup lang=\"ts\">\nconst ok = 1 < 2\nconst n = 2\n</script>\n\n<template>\n  <div>\n    x\n  </div>\n</template>\n";
    assert_eq!(
        ranges(source),
        vec![
            (5, 8, Some("region"), Some("template")),
            (0, 2, Some("region"), Some("script setup")),
            (6, 7, None, None),
        ]
    );
}

#[test]
fn an_unopened_document_yields_no_folding_ranges() {
    let state = ServerState::new();
    let params = FoldingRangeParams {
        text_document: TextDocumentIdentifier {
            uri: Url::parse("file:///Missing.vue").unwrap(),
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };
    assert!(folding_ranges(&state, &params).is_none());
}
