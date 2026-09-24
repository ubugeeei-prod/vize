//! The capability answers on hand-picked MoonBit expressions: exact where
//! the scan can prove it, pessimal everywhere else.

#![expect(clippy::unwrap_used, reason = "tests assert by panicking")]

use vize_dialect_moonbit::dialect::{MoonBitDialect, is_handler_path};
use vize_s0::{Allocator, Span};
use vize_s2::expr::capability::ExprDialect;
use vize_s2::expr::{ExprRef, ForeignExpr, OpaqueExpr, OpaqueReason};

fn answers(source: &str, dialect: &str) -> (Vec<vize_s0::String>, bool, bool) {
    let allocator = Allocator::new();
    let arena = &allocator;
    let expr = allocator.alloc(ForeignExpr {
        dialect,
        source,
        span: Span::new(10, 10 + u32::try_from(source.len()).unwrap()),
        facts: vize_s0::Vec::new_in(&arena),
    });
    let expr = ExprRef::Foreign(expr);
    let mut names = Vec::new();
    MoonBitDialect.enumerate_bindings(expr, &mut |name| names.push(name.into()));
    (
        names,
        MoonBitDialect.bindings_are_exact(expr),
        MoonBitDialect.is_constant(expr),
    )
}

#[test]
fn free_names_skip_members_packages_constructors_and_literals() {
    let (names, exact, constant) = answers(
        "@math.max(count.val, limit) + Some(total).unwrap() + \"n=\\{n}\" + 1.5",
        "moonbit",
    );
    assert_eq!(names, ["count", "limit", "total", "n"]);
    assert!(exact);
    assert!(!constant);
}

#[test]
fn binders_and_labels_make_the_enumeration_a_lower_bound() {
    for source in [
        "items.map(fn(item) { item.price })",
        "match x { Some(y) => y; None => 0 }",
        "if x is Some(y) { y } else { 0 }",
        "greet(name~)",
        "greet(name=first)",
    ] {
        let (_, exact, _) = answers(source, "moonbit");
        assert!(!exact, "{source}");
    }
}

#[test]
fn only_literal_arithmetic_is_constant() {
    assert!(answers("(1 + 2) * 3", "moonbit").2);
    assert!(answers("\"done\"", "moonbit").2);
    assert!(!answers("count + 1", "moonbit").2);
    assert!(!answers("()", "moonbit").2);
}

#[test]
fn another_dialect_and_unterminated_text_get_pessimal_answers() {
    assert_eq!(answers("count", "elixir"), (Vec::new(), false, false));
    assert_eq!(answers("\"open", "moonbit"), (Vec::new(), false, false));
}

#[test]
fn spans_map_offset_for_offset_and_emission_is_verbatim_or_refused() {
    let allocator = Allocator::new();
    let arena = &allocator;
    let foreign = allocator.alloc(ForeignExpr {
        dialect: "moonbit",
        source: "a.val",
        span: Span::new(40, 45),
        facts: vize_s0::Vec::new_in(&arena),
    });
    let expr = ExprRef::Foreign(foreign);
    assert_eq!(
        MoonBitDialect.map_span(expr, Span::new(2, 5)),
        Span::new(42, 45)
    );
    assert_eq!(
        MoonBitDialect.map_span(expr, Span::new(2, 9)),
        Span::new(40, 45)
    );
    let mut out = vize_s0::String::default();
    MoonBitDialect.emit(expr, &mut out).unwrap();
    assert_eq!(out, "a.val");
    let opaque = allocator.alloc(OpaqueExpr {
        reason: OpaqueReason::Compound,
        source: "{{ a }} b",
        span: Span::new(0, 9),
    });
    let mut out = vize_s0::String::default();
    MoonBitDialect
        .emit(ExprRef::Opaque(opaque), &mut out)
        .unwrap();
    assert_eq!(out, "{{ a }} b");
    let js = ExprRef::parse_js_in(&allocator, "a + 1", Span::new(0, 5));
    assert!(
        MoonBitDialect
            .emit(js, &mut vize_s0::String::default())
            .is_err()
    );
    assert!(!MoonBitDialect.bindings_are_exact(js));
}

#[test]
fn handler_references_are_lowercase_member_paths() {
    for path in ["save", "store.save", "a.b.c"] {
        assert!(is_handler_path(path), "{path}");
    }
    for statement in ["save()", "n.val += 1", "Save", "a.", "", "fn() { save() }"] {
        assert!(!is_handler_path(statement), "{statement}");
    }
}
