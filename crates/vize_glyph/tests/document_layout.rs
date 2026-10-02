//! Complete document layout laws independent of any parser or semantic IR.

use vize_glyph::native_doc::{Doc, Line, LineEnding, PrintOptions, print};
use vize_l0::{Allocator, Vec};

fn joined<'a>(allocator: &'a Allocator, parts: impl IntoIterator<Item = Doc<'a>>) -> Doc<'a> {
    let mut docs = Vec::new_in(&allocator);
    docs.extend(parts);
    Doc::concat(docs)
}

#[test]
fn complete_group_width_includes_closing_syntax_and_utf8_scalars() {
    let allocator = Allocator::default();
    let doc = joined(
        &allocator,
        [Doc::text("é("), Doc::line(Line::Space), Doc::text("x)")],
    )
    .group(&allocator);
    assert_eq!(
        print(
            &doc,
            &PrintOptions {
                width: 5,
                ..PrintOptions::default()
            }
        ),
        "é( x)"
    );
    assert_eq!(
        print(
            &doc,
            &PrintOptions {
                width: 4,
                ..PrintOptions::default()
            }
        ),
        "é(\nx)"
    );
}

#[test]
fn indent_scope_ends_before_closing_line() {
    let allocator = Allocator::default();
    let body =
        joined(&allocator, [Doc::line(Line::Empty), Doc::text("value")]).indent(1, &allocator);
    let doc = joined(
        &allocator,
        [Doc::text("["), body, Doc::line(Line::Empty), Doc::text("]")],
    )
    .group(&allocator);
    assert_eq!(
        print(
            &doc,
            &PrintOptions {
                width: 5,
                ..PrintOptions::default()
            }
        ),
        "[\n  value\n]"
    );
    assert_eq!(print(&doc, &PrintOptions::default()), "[value]");
}

#[test]
fn inner_group_can_stay_flat_when_outer_group_breaks() {
    let allocator = Allocator::default();
    let inner = joined(
        &allocator,
        [Doc::text("x"), Doc::line(Line::Space), Doc::text("y")],
    )
    .group(&allocator);
    let doc = joined(
        &allocator,
        [Doc::text("prefix"), Doc::line(Line::Empty), inner],
    )
    .group(&allocator);
    assert_eq!(
        print(
            &doc,
            &PrintOptions {
                width: 6,
                ..PrintOptions::default()
            }
        ),
        "prefix\nx y"
    );
}

#[test]
fn group_reserves_width_for_its_real_pending_continuation() {
    let allocator = Allocator::default();
    let inner = joined(
        &allocator,
        [Doc::text("a"), Doc::line(Line::Space), Doc::text("b")],
    )
    .group(&allocator);
    let doc = joined(&allocator, [inner, Doc::text("!")]);
    assert_eq!(
        print(
            &doc,
            &PrintOptions {
                width: 3,
                ..PrintOptions::default()
            }
        ),
        "a\nb!"
    );
    assert_eq!(
        print(
            &doc,
            &PrintOptions {
                width: 4,
                ..PrintOptions::default()
            }
        ),
        "a b!"
    );
}

#[test]
fn hard_line_and_original_line_endings_are_distinct() {
    let allocator = Allocator::default();
    let doc = joined(
        &allocator,
        [
            Doc::text("before\nraw"),
            Doc::hard_line(),
            Doc::text("after"),
        ],
    )
    .group(&allocator);
    assert_eq!(
        print(
            &doc,
            &PrintOptions {
                line_ending: LineEnding::CrLf,
                ..PrintOptions::default()
            }
        ),
        "before\nraw\r\nafter"
    );
}

#[test]
fn printer_never_truncates_unbreakable_source_at_zero_width() {
    assert_eq!(
        print(
            &Doc::text("αβ&entity;"),
            &PrintOptions {
                width: 0,
                ..PrintOptions::default()
            }
        ),
        "αβ&entity;"
    );
}

#[test]
fn printer_does_not_recurse_through_deep_document_groups() {
    let allocator = Allocator::default();
    let mut doc = Doc::text("retained");
    for _ in 0..20_000 {
        doc = doc.group(&allocator);
    }
    assert_eq!(print(&doc, &PrintOptions::default()), "retained");
}
