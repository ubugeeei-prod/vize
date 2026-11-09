use vize_l0::{Allocator, Span};
use vize_l2::expr::JsExpr;
use vize_l2::resolution::{
    BindingId, BindingLookup, ResolutionErrorKind, Usage, resolve_expression,
};

struct Bindings<'a>(&'a [&'a str]);

impl BindingLookup for Bindings<'_> {
    fn lookup(&self, name: &str) -> Option<BindingId> {
        self.0
            .iter()
            .position(|candidate| *candidate == name)
            .and_then(|index| u32::try_from(index).ok())
            .map(BindingId::new)
    }
}

#[test]
fn retained_ast_resolves_values_not_static_keys_or_literal_text() {
    let arena = Allocator::default();
    let source = "({ a, key: obj.name, [key]: fn(a, ...xs), str: 'a' })";
    let expression = JsExpr::parse_in(&arena, source, Span::new(10, 10 + source.len() as u32))
        .expect("retained expression");
    let table = resolve_expression(expression, &Bindings(&["a", "obj", "key", "fn", "xs"]))
        .expect("complete runtime references");
    assert!(core::ptr::eq(expression.ast, table.expression().ast));
    let references: Vec<_> = table
        .occurrences()
        .iter()
        .map(|entry| (entry.name, entry.span, entry.shorthand))
        .collect();
    assert_eq!(
        references,
        vec![
            ("a", Span::new(3, 4), true),
            ("obj", Span::new(11, 14), false),
            ("key", Span::new(22, 25), false),
            ("fn", Span::new(28, 30), false),
            ("a", Span::new(31, 32), false),
            ("xs", Span::new(37, 39), false),
        ]
    );
    assert_eq!(
        table.occurrences()[0].binding,
        table.occurrences()[4].binding
    );
}

#[test]
fn assignments_distinguish_target_writes_from_member_object_reads() {
    let arena = Allocator::default();
    let source = "(a = b, a += b, ++a, obj[key] = a)";
    let expression = JsExpr::parse_in(&arena, source, Span::new(0, source.len() as u32))
        .expect("retained assignment expression");
    let table = resolve_expression(expression, &Bindings(&["a", "b", "obj", "key"]))
        .expect("all target roles");
    let references: Vec<_> = table
        .occurrences()
        .iter()
        .map(|entry| (entry.name, entry.usage))
        .collect();
    assert_eq!(
        references,
        vec![
            ("a", Usage::Write),
            ("b", Usage::Read),
            ("a", Usage::ReadWrite),
            ("b", Usage::Read),
            ("a", Usage::ReadWrite),
            ("obj", Usage::Read),
            ("key", Usage::Read),
            ("a", Usage::Read),
        ]
    );
}

#[test]
fn escaped_and_unicode_names_keep_semantics_and_byte_spans() {
    let arena = Allocator::default();
    let source = "\\u0061 + 日本語";
    let expression = JsExpr::parse_in(&arena, source, Span::new(7, 7 + source.len() as u32))
        .expect("retained unicode expression");
    let table =
        resolve_expression(expression, &Bindings(&["a", "日本語"])).expect("semantic names");
    assert_eq!(table.occurrences()[0].name, "a");
    assert_eq!(table.occurrences()[0].span, Span::new(0, 6));
    assert_eq!(table.occurrences()[1].span, Span::new(9, 18));
}

#[test]
fn empty_facts_only_follow_complete_analysis() {
    let arena = Allocator::default();
    let expression =
        JsExpr::parse_in(&arena, "42 /* name */", Span::new(0, 13)).expect("retained literal");
    let table = resolve_expression(expression, &Bindings(&[])).expect("analyzed literal");
    assert!(table.occurrences().is_empty());
    let expression =
        JsExpr::parse_in(&arena, "a + missing", Span::new(0, 11)).expect("retained identifiers");
    let error = resolve_expression(expression, &Bindings(&["a"]))
        .expect_err("missing facts reject the complete result");
    assert_eq!(error.kind, ResolutionErrorKind::MissingBinding);
    assert_eq!(error.span, Span::new(4, 11));
}

#[test]
fn unsupported_families_do_not_return_a_partial_reference_list() {
    let arena = Allocator::default();
    for source in [
        "a + (() => b)",
        "a as number",
        "({ a } = obj)",
        "({ get x() { return a } })",
        "delete a",
        "eval(a)",
        "(eval)(a)",
    ] {
        let expression = JsExpr::parse_in(&arena, source, Span::new(0, source.len() as u32))
            .expect("valid retained syntax");
        let error = resolve_expression(expression, &Bindings(&["a", "b", "obj", "eval"]))
            .expect_err("unsupported syntax must reject");
        assert_eq!(
            error.kind,
            ResolutionErrorKind::UnsupportedSyntax,
            "{source}"
        );
    }
}

#[test]
fn chains_templates_new_and_import_keep_all_runtime_uses_in_source_order() {
    let arena = Allocator::default();
    for (source, names, expected) in [
        (
            "obj?.[key]?.(value)",
            vec!["obj", "key", "value"],
            vec!["obj", "key", "value"],
        ),
        (
            "tag`a ${value} ${other}`",
            vec!["tag", "value", "other"],
            vec!["tag", "value", "other"],
        ),
        (
            "new ctor(value)",
            vec!["ctor", "value"],
            vec!["ctor", "value"],
        ),
        (
            "import(path, options)",
            vec!["path", "options"],
            vec!["path", "options"],
        ),
    ] {
        let expression = JsExpr::parse_in(&arena, source, Span::new(0, source.len() as u32))
            .expect("retained expression");
        let table =
            resolve_expression(expression, &Bindings(&names)).expect("all runtime references");
        assert_eq!(
            table
                .occurrences()
                .iter()
                .map(|entry| entry.name)
                .collect::<Vec<_>>(),
            expected,
            "{source}"
        );
        if source.starts_with("new ") {
            assert!(table.occurrences()[0].constructor);
            assert!(!table.occurrences()[1].constructor);
        }
    }
}

#[test]
fn traversal_limits_cover_empty_elements_and_expression_depth() {
    let arena = Allocator::default();
    let mut source = vize_l0::String::new("[");
    for _ in 0..4096 {
        source.push(',');
    }
    source.push(']');
    let expression = JsExpr::parse_in(&arena, &source, Span::new(0, source.len() as u32))
        .expect("retained array with elisions");
    assert_eq!(
        resolve_expression(expression, &Bindings(&[]))
            .expect_err("empty elements still consume traversal work")
            .kind,
        ResolutionErrorKind::TraversalLimit
    );

    let mut source = vize_l0::String::new("");
    for _ in 0..66 {
        source.push('!');
    }
    source.push('a');
    // Retained AST admission has its own guard even if a producer's parser
    // admits more depth than the legacy load-path text guard.
    let ast = oxc_parser::Parser::new(arena.as_oxc(), &source, oxc_span::SourceType::ts())
        .parse_expression()
        .expect("actual deeply nested AST");
    let span = Span::new(0, source.len() as u32);
    let coordinates = vize_l2::expr::js::JsCoordinates::checked(&source, &source, span, 0, &[])
        .expect("identity retained source");
    let expression = JsExpr::from_retained_in(&arena, arena.alloc(ast), &source, span, coordinates)
        .expect("retained nested expression");
    assert_eq!(
        resolve_expression(expression, &Bindings(&["a"]))
            .expect_err("nesting is bounded independently of parser admission")
            .kind,
        ResolutionErrorKind::TraversalLimit
    );
}
