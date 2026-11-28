use oxc_allocator::Allocator;
use oxc_span::SourceType;

use crate::Parser;

/// A thread stack far smaller than the nesting below needs without growth.
const SMALL_STACK: usize = 256 * 1024;
const DEPTH: usize = 20_000;

/// `prefix`, then `open` [`DEPTH`] times, `middle`, and `close` as often.
fn nested(prefix: &str, open: &str, middle: &str, close: &str) -> String {
    let mut source = String::from(prefix);
    source.push_str(&open.repeat(DEPTH));
    source.push_str(middle);
    source.push_str(&close.repeat(DEPTH));
    source
}

/// Parse `source` on a [`SMALL_STACK`] thread and return whether it parsed
/// without diagnostics.
fn parses_cleanly(source: String, source_type: SourceType) -> bool {
    std::thread::Builder::new()
        .stack_size(SMALL_STACK)
        .spawn(move || {
            let allocator = Allocator::default();
            let result = Parser::new(&allocator, &source, source_type).parse();
            !result.panicked && result.diagnostics.is_empty()
        })
        .unwrap()
        .join()
        .unwrap()
}

#[test]
fn nested_expressions_grow_past_a_small_stack() {
    let ts = SourceType::ts();
    let cases = [
        ("array", nested("x = ", "[", "1", "]")),
        ("paren", nested("x = ", "(", "1", ")")),
        ("object", nested("x = ", "{a:", "1", "}")),
        ("call", nested("x = ", "f(", "1", ")")),
        ("template", nested("x = ", "`${", "1", "}`")),
        ("conditional", nested("x = ", "a ? b : ", "c", "")),
        ("assignment", nested("", "x = ", "1", "")),
        ("arrow", nested("x = ", "() => ", "0", "")),
        ("unary", nested("x = ", "!", "a", "")),
        ("await", nested("async function f() { ", "await ", "a", "") + " }"),
        ("new", nested("x = ", "new ", "A", "")),
        ("assertion", nested("x = ", "<T>", "a", "")),
        ("destructuring assignment", nested("", "[", "a", "]") + " = b"),
        ("parenthesized target", nested("", "(", "a", ")") + " = b"),
    ];
    for (name, source) in cases {
        assert!(parses_cleanly(source, ts), "{name}");
    }
}

#[test]
fn nested_statements_and_patterns_grow_past_a_small_stack() {
    let ts = SourceType::ts();
    let cases = [
        ("block", nested("", "{", "", "}")),
        ("if", nested("", "if (a) ", ";", "")),
        ("function", nested("", "function f() {", "", "}")),
        ("binding", nested("const ", "[", "a", "]") + " = b"),
        ("object binding", nested("const ", "{a:", "a", "}") + " = b"),
        ("type arguments", nested("type T = ", "A<", "B", ">")),
        ("tuple type", nested("type T = ", "[", "B", "]")),
        ("type operator", nested("type T = ", "keyof ", "B", "")),
    ];
    for (name, source) in cases {
        assert!(parses_cleanly(source, ts), "{name}");
    }
    assert!(parses_cleanly(nested("x = ", "<a>", "", "</a>"), SourceType::tsx()), "jsx");
}

#[test]
fn shallow_sources_parse_identically_with_and_without_growth() {
    let source = "const x = [{ a: f(1, `${b}`) }, (c ? d : e)]; if (x) { y(); }";
    let parse = |stack: usize| {
        std::thread::Builder::new()
            .stack_size(stack)
            .spawn(move || {
                let allocator = Allocator::default();
                let result = Parser::new(&allocator, source, SourceType::ts()).parse();
                format!("{:?}", result.program)
            })
            .unwrap()
            .join()
            .unwrap()
    };
    // 64 KiB starts inside the red zone, so every step runs on a grown segment.
    assert_eq!(parse(64 * 1024), parse(8 * 1024 * 1024));
}
