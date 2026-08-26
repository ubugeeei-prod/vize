use tower_lsp::lsp_types::{
    FoldingRangeKind, FoldingRangeParams, PartialResultParams, TextDocumentIdentifier, Url,
    WorkDoneProgressParams,
};

use super::{AuthoredLineMap, folding_ranges};
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
fn script_and_style_bodies_fold_alongside_blocks_and_template_elements() {
    let source = "<script setup lang=\"ts\">\nconst config = {\n  nested: true,\n}\n\nfunction greet() {\n  return config\n}\n</script>\n\n<template>\n  <section>\n    <div>hello</div>\n  </section>\n</template>\n\n<style scoped>\n.card {\n  color: red;\n}\n</style>\n";

    assert_eq!(
        ranges(source),
        vec![
            (10, 13, Some("region"), Some("template")),
            (0, 7, Some("region"), Some("script setup")),
            (16, 19, Some("region"), Some("style")),
            (11, 12, None, None),
            (1, 2, None, None),
            (5, 6, None, None),
            (17, 18, None, None),
        ]
    );
}

#[test]
fn script_ast_ignores_braces_in_literals_and_folds_nested_bodies() {
    let source = "<script lang=\"ts\">\nconst marker = \"}\"\nconst regex = /}/\nconst config = {\n  nested: {\n    enabled: true,\n  },\n  run() {\n    return marker\n  }\n}\n</script>\n";

    assert_eq!(
        ranges(source),
        vec![
            (0, 10, Some("region"), Some("script")),
            (3, 9, None, None),
            (4, 5, None, None),
            (7, 8, None, None),
        ]
    );
}

#[test]
fn style_scanner_ignores_braces_in_strings_and_comments() {
    let source = "<style lang=\"scss\">\n.outer {\n  content: \"}\";\n  /*\n    } {\n  */\n  // }\n  .inner {\n    color: red;\n  }\n}\n</style>\n";

    assert_eq!(
        ranges(source),
        vec![
            (0, 10, Some("region"), Some("style")),
            (1, 9, None, None),
            (7, 8, None, None),
        ]
    );
}

#[test]
fn multiline_opening_tags_keep_script_and_style_ranges_on_authored_lines() {
    let source = "<script\n  setup\n  lang=\"ts\"\n>\nconst config = {\n  enabled: true,\n}\n</script>\n\n<style\n  lang=\"scss\"\n>\n.card {\n  color: red;\n}\n</style>\n";

    assert_eq!(
        ranges(source),
        vec![
            (0, 6, Some("region"), Some("script setup")),
            (9, 14, Some("region"), Some("style")),
            (4, 5, None, None),
            (12, 13, None, None),
        ]
    );
}

#[test]
fn style_line_comments_do_not_start_at_url_schemes() {
    let source = "<style lang=\"scss\">\n.remote {\n  color: red;\n  background: url(https://example.test/a.png); }\n\n.local {\n  // } is not structural\n  color: blue;\n  padding: 0;\n}\n</style>\n";

    assert_eq!(
        ranges(source),
        vec![
            (0, 9, Some("region"), Some("style")),
            (1, 2, None, None),
            (5, 8, None, None),
        ]
    );
}

#[test]
fn script_and_style_internal_work_stays_linear() {
    let mut previous_script_work = None;
    let mut previous_style_work = None;

    for count in [128, 256, 512, 1024] {
        let mut source = vize_s0::String::from("<script setup lang=\"ts\">\nconst records = [\n");
        for _ in 0..count {
            source.push_str("  {\n    enabled: true,\n  },\n");
        }
        source.push_str("]\n</script>\n<style lang=\"scss\">\n");
        for _ in 0..count {
            source.push_str(".rule {\n  color: red;\n}\n");
        }
        source.push_str("</style>\n");

        let descriptor =
            vize_atelier_sfc::parse_sfc(&source, vize_atelier_sfc::SfcParseOptions::default())
                .unwrap();
        let authored_lines = AuthoredLineMap::new(&source);
        let script = descriptor.script_setup.as_ref().unwrap();
        let style = descriptor.styles.first().unwrap();
        let (script_ranges, script_work) = super::script::script_regions_with_metrics(
            script,
            authored_lines.line_at(script.loc.start),
        );
        let (style_ranges, style_work) = super::style::style_regions_with_metrics(
            style,
            authored_lines.line_at(style.loc.start),
        );

        assert_eq!(script_ranges.len(), count);
        assert_eq!(style_ranges.len(), count);
        assert!(script_work <= script.content.len() + count);
        assert!(style_work <= style.content.len() + count);
        if let Some(previous) = previous_script_work {
            assert!(script_work <= previous * 2 + 32);
        }
        if let Some(previous) = previous_style_work {
            assert!(style_work <= previous * 2 + 32);
        }
        previous_script_work = Some(script_work);
        previous_style_work = Some(style_work);
    }
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
