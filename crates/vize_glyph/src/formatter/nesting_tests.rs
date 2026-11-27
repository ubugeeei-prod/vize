//! What an SFC keeps when one block nests past a formatter limit: that block
//! stays as authored and the other blocks are still formatted.

use crate::template::MAX_ELEMENT_NESTING_DEPTH;
use crate::{Allocator, FormatOptions, GlyphFormatter, format_sfc};
use vize_l1::css::nesting::MAX_CSS_NESTING_DEPTH;

const UNFORMATTED_SCRIPT: &str = "<script setup>\nconst a=1\n</script>\n";
const FORMATTED_SCRIPT: &str = "<script setup>\nconst a = 1;\n</script>\n";

/// `depth` open elements around one text node, one tag per line.
fn nested_elements(depth: usize) -> String {
    "<a>\n".repeat(depth) + "x\n" + &"</a>\n".repeat(depth)
}

/// Format twice: the first result must be `expected`, and stable.
#[track_caller]
fn assert_formats_to(source: &str, expected: &str) {
    let options = FormatOptions::default();
    let first = format_sfc(source, &options).unwrap();
    assert_eq!(first.code.as_str(), expected);
    assert_eq!(first.changed, source != expected);
    let second = format_sfc(first.code.as_str(), &options).unwrap();
    assert_eq!(second.code.as_str(), expected);
    assert!(!second.changed);
}

#[test]
fn template_nested_past_the_limit_stays_as_authored() {
    let template = format!(
        "<template>\n{}</template>\n",
        nested_elements(MAX_ELEMENT_NESTING_DEPTH + 1)
    );
    assert_formats_to(
        &format!("{UNFORMATTED_SCRIPT}\n{template}"),
        &format!("{FORMATTED_SCRIPT}\n{template}"),
    );
}

#[test]
fn unclosed_template_nesting_far_past_the_limit_stays_as_authored() {
    // One line, as a truncated or generated file may be.
    let source = format!("<template>{}</template>\n", "<a>".repeat(30_000));
    assert_formats_to(&source, &source);
}

#[test]
fn whitespace_preserving_template_nested_past_the_limit_stays_as_authored() {
    let source = format!(
        "<template>\n{}</template>\n",
        nested_elements(MAX_ELEMENT_NESTING_DEPTH + 1)
    );
    let options = FormatOptions::default();
    let allocator = Allocator::default();
    let result = GlyphFormatter::new(&options, &allocator)
        .with_template_whitespace_preserved(true)
        .format(&source)
        .unwrap();
    assert_eq!(result.code.as_str(), source);
    assert!(!result.changed);
}

#[test]
fn art_block_nested_past_the_limit_stays_as_authored() {
    let source = format!(
        "<art title=\"deep\">\n{}</art>\n",
        nested_elements(MAX_ELEMENT_NESTING_DEPTH + 1)
    );
    assert_formats_to(&source, &source);
}

#[test]
fn style_nested_past_the_limit_stays_as_authored() {
    // The rule's own block and 32 functions: one bracket past the limit.
    let style = format!(
        "<style>\n.a{{width:{}1px{}}}\n</style>\n",
        "calc(".repeat(MAX_CSS_NESTING_DEPTH),
        ")".repeat(MAX_CSS_NESTING_DEPTH)
    );
    assert_formats_to(
        &format!("{UNFORMATTED_SCRIPT}\n{style}"),
        &format!("{FORMATTED_SCRIPT}\n{style}"),
    );
}

#[test]
fn interpolation_nested_past_the_parser_guard_stays_as_authored() {
    let source = format!(
        "<template>\n  <div>\n    {{{{\n      {}a{}\n    }}}}\n  </div>\n</template>\n",
        "(".repeat(20_000),
        ")".repeat(20_000)
    );
    assert_formats_to(&source, &source);
}
