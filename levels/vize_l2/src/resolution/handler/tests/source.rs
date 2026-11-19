use super::{Allocator, Outer, Span, Usage, input, resolve_handler};

#[test]
fn once_decoded_entities_unicode_and_original_source_spans_stay_exact() {
    let allocator = Allocator::default();
    let original = input(&allocator, "return α &amp;&amp; β");
    let source = original.operand().syntax().source();
    let map = source.decode_map().unwrap().segments().as_ptr();
    let body = original.body();
    let resolution = resolve_handler(original, &Outer(&[("α", 1), ("β", 2)])).unwrap();
    assert!(core::ptr::eq(resolution.input().body(), body));
    assert_eq!(
        resolution
            .input()
            .operand()
            .syntax()
            .source()
            .decode_map()
            .unwrap()
            .segments()
            .as_ptr(),
        map
    );
    let [left, right] = resolution.references() else {
        panic!("two original references")
    };
    assert_eq!(left.span, Span::new(7, 9));
    assert_eq!(right.span, Span::new(13, 15));
    let left = resolution.authored_span(left.span).unwrap();
    let right = resolution.authored_span(right.span).unwrap();
    assert_eq!(left, Span::new(28, 30));
    assert_eq!(right, Span::new(42, 44));
    assert_eq!(left.slice(source.authored_root()), "α");
    assert_eq!(right.slice(source.authored_root()), "β");
}

#[test]
fn escaped_shorthand_and_constructor_facts_come_from_original_descendants() {
    let allocator = Allocator::default();
    let resolution =
        resolve_handler(input(&allocator, r"({\u0061})"), &Outer(&[("a", 7)])).unwrap();
    let [reference] = resolution.references() else {
        panic!("one original shorthand")
    };
    assert_eq!(reference.name, "a");
    assert!(reference.shorthand);
    assert_eq!(reference.usage, Usage::Read);
    assert_eq!(reference.span, Span::new(2, 8));
    let source = resolution.input().operand().syntax().source();
    assert_eq!(
        resolution
            .authored_span(reference.span)
            .unwrap()
            .slice(source.authored_root()),
        r"\u0061"
    );
    let resolution = resolve_handler(
        input(&allocator, "new Box(arg)"),
        &Outer(&[("Box", 2), ("arg", 3)]),
    )
    .unwrap();
    assert!(resolution.references()[0].constructor);
    assert!(!resolution.references()[1].constructor);
}

#[test]
fn complete_statement_usage_distinguishes_member_reads_from_direct_target_writes() {
    let allocator = Allocator::default();
    for (text, expected) in [
        (
            "count=value",
            [("count", Usage::Write), ("value", Usage::Read)],
        ),
        (
            "state.key=count",
            [("state", Usage::Read), ("count", Usage::Read)],
        ),
        (
            "count+=value",
            [("count", Usage::ReadWrite), ("value", Usage::Read)],
        ),
    ] {
        let resolution = resolve_handler(
            input(&allocator, text),
            &Outer(&[("count", 1), ("value", 2), ("state", 3)]),
        )
        .unwrap();
        assert_eq!(
            resolution
                .references()
                .iter()
                .map(|row| (row.name, row.usage))
                .collect::<alloc::vec::Vec<_>>(),
            expected
        );
    }
}
