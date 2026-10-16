//! Full-token, retained-owner and refusal regressions for the demanded path.

#![expect(
    clippy::string_slice,
    reason = "tests assert complete original identifier token spans"
)]

use super::*;
use oxc_allocator::Allocator;
use oxc_ast::ast::{Expression, Statement};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType, Span};

fn refs(source: &str) -> Option<Vec<RetainedIdentifierRef>> {
    let allocator = Allocator::default();
    let ast = Parser::new(&allocator, source, SourceType::ts())
        .parse_expression()
        .unwrap();
    let retained = JsExpression {
        ast: &ast,
        raw: source,
    };
    let identity = std::ptr::from_ref(retained.ast);
    let before = retained.ast.span();
    let result = retained_expression_refs(&retained);
    assert_eq!(std::ptr::from_ref(retained.ast), identity);
    assert_eq!(retained.ast.span(), before);
    result
}

fn params(source: &str) -> Option<RetainedParameterRefs> {
    let allocator = Allocator::default();
    let ast = Parser::new(&allocator, source, SourceType::ts())
        .parse_slot_parameters()
        .unwrap();
    retained_parameter_refs(&ast)
}

fn declarations(source: &str) -> Option<Vec<RetainedIdentifierRef>> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    assert!(
        !parsed.panicked && parsed.diagnostics.is_empty(),
        "{source}"
    );
    let Statement::VariableDeclaration(declaration) = &parsed.program.body[0] else {
        panic!("original variable declaration")
    };
    let pattern = &declaration.declarations[0].id;
    let original = std::ptr::from_ref(pattern);
    let span = pattern.span();
    let declarations = retained_binding_declarations(pattern);
    assert_eq!(std::ptr::from_ref(pattern), original);
    assert_eq!(pattern.span(), span);
    declarations
}

#[test]
fn retained_binding_declarations_keep_only_actual_full_binding_tokens() {
    let source = r"const { [key()]: \u0024listeners = eval('code'), nested: [first, , ...rest], ...others } = input;";
    let found = declarations(source).unwrap();
    assert_eq!(names(&found), ["$listeners", "first", "rest", "others"]);
    for (reference, spelling) in found
        .iter()
        .zip([r"\u0024listeners", "first", "rest", "others"])
    {
        assert_eq!(
            &source[reference.span.start as usize..reference.span.end as usize],
            spelling
        );
    }
    assert_eq!(
        names(&declarations("let plain = external;").unwrap()),
        ["plain"]
    );
    assert_eq!(
        names(&declarations("var [first = outside, ...rest] = input;").unwrap()),
        ["first", "rest"]
    );
    assert!(declarations("const {} = input;").unwrap().is_empty());
    // An ignored initializer may exceed the identifier-read depth budget or
    // contain eval/unsupported value syntax. It declares no pattern names.
    let source = format!("const [local = {}outside] = input;", "!".repeat(100));
    assert_eq!(names(&declarations(&source).unwrap()), ["local"]);
    assert_eq!(
        names(&declarations("const { value = class {}, [eval('key')]: alias } = input;").unwrap()),
        ["value", "alias"]
    );
}

#[test]
fn retained_binding_declarations_refuse_whole_deep_or_wide_patterns() {
    let source = format!(
        "const [first, {}nested{}] = input;",
        "[".repeat(70),
        "]".repeat(70)
    );
    assert!(
        declarations(&source).is_none(),
        "never return the first partial binding"
    );
    let wide = (0..2200)
        .map(|index| format!("local{index}"))
        .collect::<Vec<_>>()
        .join(",");
    assert!(declarations(&format!("const [first, {wide}] = input;")).is_none());
    // Refusal is local to this demand; it cannot poison another original owner.
    assert_eq!(
        names(&declarations("const next = input;").unwrap()),
        ["next"]
    );
}

fn handler(source: &str) -> Option<Vec<RetainedIdentifierRef>> {
    // This entire function is the original authored test source. Its actual
    // body is borrowed; no synthetic handler wrapper or reparsing is involved.
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    assert!(
        !parsed.panicked && parsed.diagnostics.is_empty(),
        "{source}"
    );
    let Statement::FunctionDeclaration(function) = &parsed.program.body[0] else {
        panic!("expected an authored function");
    };
    retained_handler_refs(function.body.as_ref().unwrap())
}

fn names(references: &[RetainedIdentifierRef]) -> Vec<&str> {
    references
        .iter()
        .map(|reference| reference.name.as_str())
        .collect()
}

#[test]
fn full_spans_belong_to_original_identifier_nodes_including_escapes() {
    let source = r"[\u0024listeners, { \u0024listeners }, target[\u0024listeners]]";
    let found = refs(source).unwrap();
    assert_eq!(
        names(&found),
        ["$listeners", "$listeners", "target", "$listeners"]
    );
    for reference in found
        .iter()
        .filter(|reference| reference.name == "$listeners")
    {
        assert_eq!(
            &source[reference.span.start as usize..reference.span.end as usize],
            r"\u0024listeners"
        );
        assert_eq!(reference.span.end - reference.span.start, 15);
    }

    let source = r"({ [key]: \u0024listeners = fallback } = input)";
    let found = refs(source).unwrap();
    assert_eq!(names(&found), ["key", "$listeners", "fallback", "input"]);
    let token = &found[1];
    assert_eq!(
        &source[token.span.start as usize..token.span.end as usize],
        r"\u0024listeners"
    );

    let source = r"{ [key]: \u0024listeners = fallback }, ...rest";
    let found = params(source).unwrap();
    assert_eq!(names(&found.bindings), ["$listeners", "rest"]);
    assert_eq!(names(&found.references), ["key", "fallback"]);
    assert_eq!(
        &source[found.bindings[0].span.start as usize..found.bindings[0].span.end as usize],
        r"\u0024listeners"
    );
}

#[test]
fn comments_regex_types_and_this_keep_value_references_only() {
    for (source, expected) in [
        ("this.$listeners", vec![]),
        ("slot/* $listeners */.name", vec!["slot"]),
        ("/[/*]/.test(slot)", vec!["slot"]),
        ("($listeners as typeof TypeOnly)", vec!["$listeners"]),
        (
            "($listeners satisfies Record<TypeOnly, OtherType>)",
            vec!["$listeners"],
        ),
        ("use<TypeOnly>($listeners)", vec!["use", "$listeners"]),
        (
            "(value: TypeOnly = fallback): ReturnType => value + outer",
            vec!["fallback", "outer"],
        ),
        (
            "({ [key]: $listeners, static: other, ...spread })",
            vec!["key", "$listeners", "other", "spread"],
        ),
        (
            "receiver?.[key]?.($listeners, ...other)",
            vec!["receiver", "key", "$listeners", "other"],
        ),
    ] {
        assert_eq!(names(&refs(source).unwrap()), expected, "{source}");
    }
}

#[test]
fn callback_and_parameter_scopes_keep_defaults_keys_and_outer_reads() {
    for (source, expected) in [
        ("(function(slot) { return slot })(name)", vec!["name"]),
        ("(function() { return slot })()", vec!["slot"]),
        ("function named() { return named }", vec![]),
        (
            "function () { return arguments[0] + outside }",
            vec!["outside"],
        ),
        ("() => arguments[0] + outside", vec!["arguments", "outside"]),
        (
            "(a, b = a, { c = d } = e) => b + c + f",
            vec!["d", "e", "f"],
        ),
        (
            "({ [key]: local = fallback } = options) => local + outside",
            vec!["key", "fallback", "options", "outside"],
        ),
        ("() => { { const local = 1 } return local }", vec!["local"]),
        (
            "() => { var a = 1; if (flag) { var b = 2 } return a + b }",
            vec!["flag"],
        ),
        (
            "() => { if (flag) { const inner = make(); use(inner) } else { use(inner) } }",
            vec!["flag", "make", "use", "use", "inner"],
        ),
        (
            "(value = outside) => { const outside = 1; return value + outside }",
            vec!["outside"],
        ),
    ] {
        assert_eq!(names(&refs(source).unwrap()), expected, "{source}");
    }
    let source = "first, second = first, { [key]: local = fallback } = options, ...rest";
    let found = params(source).unwrap();
    assert_eq!(names(&found.bindings), ["first", "second", "local", "rest"]);
    assert_eq!(names(&found.references), ["key", "fallback", "options"]);
    let typed = params("value: TypeOnly = fallback, other: typeof RuntimeName = value").unwrap();
    assert_eq!(names(&typed.bindings), ["value", "other"]);
    assert_eq!(names(&typed.references), ["fallback"]);
}

#[test]
fn handler_blocks_track_locals_without_losing_original_spans() {
    let source = "function handler() { const { [key]: local = fallback } = options; if (flag) { const inner = local; consume(inner) } else { consume(inner) } return local + outside }";
    let found = handler(source).unwrap();
    assert_eq!(
        names(&found),
        [
            "key", "fallback", "options", "flag", "consume", "consume", "inner", "outside"
        ]
    );
    for reference in &found {
        assert_eq!(
            &source[reference.span.start as usize..reference.span.end as usize],
            reference.name.as_str()
        );
    }
    assert_eq!(
        names(
            &handler("function handler() { if (flag) { var local = source } return local }")
                .unwrap()
        ),
        ["flag", "source"]
    );
    assert_eq!(names(&handler("function handler() { let local = source; local++; target[key] = local; throw error }").unwrap()), ["source", "target", "key", "error"]);
}

#[test]
fn refusal_never_returns_partial_expression_parameter_or_handler_facts() {
    for source in [
        "[first, eval(code), last]",
        "[first, (eval)(code)]",
        "[first, (eval as any)(code)]",
        "[first, class { field = last }]",
        "() => { first; for (const item of items) consume(item) }",
        "() => { first; try { consume() } catch (error) { use(error) } }",
        "() => { first; function helper() { return last } }",
        "() => { first; class Local {} }",
    ] {
        assert!(refs(source).is_none(), "{source}");
    }
    assert!(params("first = outside, second = eval(code)").is_none());
    for source in [
        "function handler() { first; while (flag) consume() }",
        "function handler() { first; switch (flag) { default: consume() } }",
        "function handler() { first; eval(code); last }",
    ] {
        assert!(handler(source).is_none(), "{source}");
    }
}

#[test]
fn bounded_depth_and_width_apply_to_every_nested_phase() {
    assert!(refs(&format!("{}outer", "!".repeat(70))).is_none());
    assert!(refs(&format!("[first,{}]", "0,".repeat(4096))).is_none());
    // Many shallow child functions must not reset a per-child node budget.
    assert!(
        refs(&format!(
            "[first,{}]",
            "(local = outside) => local,".repeat(600)
        ))
        .is_none()
    );
    let nested_pattern = format!("{}value{}", "[".repeat(70), "]".repeat(70));
    assert!(params(&nested_pattern).is_none());
    // The binding collector is bounded even when the pattern reads nothing.
    let wide_bindings = (0..2200)
        .map(|index| format!("local{index}"))
        .collect::<Vec<_>>()
        .join(",");
    assert!(params(&format!("[{wide_bindings}]")).is_none());
    let nested_blocks = format!(
        "function handler() {{ {}var local = first;{} return local }}",
        "{".repeat(70),
        "}".repeat(70)
    );
    assert!(handler(&nested_blocks).is_none());
    let many_declarations = format!(
        "function handler() {{ first; {} }}",
        "{ var local = 0; }".repeat(800)
    );
    assert!(handler(&many_declarations).is_none());
    // A refusal leaves no global/thread-local state that could poison a later
    // bounded read, including successful empty results.
    assert_eq!(names(&refs("next").unwrap()), ["next"]);
    assert!(refs("this").unwrap().is_empty());
}

#[test]
fn legacy_public_references_and_outputs_remain_unchanged() {
    for (source, expected) in [
        ("item.name + other", vec![("item", 0), ("other", 12)]),
        ("({ active: value })", vec![("value", 11)]),
        ("(local) => local + outside", vec![("outside", 19)]),
        (r"\u0024listeners", vec![("$listeners", 0)]),
    ] {
        let expected: Vec<_> = expected
            .into_iter()
            .map(|(name, offset)| IdentifierRef {
                name: name.into(),
                offset,
            })
            .collect();
        assert_eq!(extract_identifier_refs_oxc(source), expected, "{source}");
    }
    // Legacy compiler behavior is intentionally unbounded and best-effort.
    let deep = format!("{}outer", "!".repeat(70));
    assert_eq!(
        extract_identifier_refs_oxc(&deep).last().unwrap().name,
        "outer"
    );
    assert_eq!(
        extract_identifier_refs_oxc("() => { first; while (flag) last }"),
        [IdentifierRef {
            name: "first".into(),
            offset: 8
        }]
    );
}

#[test]
fn full_span_is_the_actual_ast_span_without_a_name_length_inference() {
    let allocator = Allocator::default();
    let source = r"\u{24}listeners";
    let ast = Parser::new(&allocator, source, SourceType::ts())
        .parse_expression()
        .unwrap();
    let Expression::Identifier(id) = &ast else {
        panic!("expected identifier")
    };
    let found = retained_expression_refs(&JsExpression {
        ast: &ast,
        raw: source,
    })
    .unwrap();
    assert_eq!(found[0].span, id.span);
    assert_eq!(found[0].span, Span::new(0, source.len() as u32));
    assert_ne!(found[0].span.end as usize, found[0].name.len());
}
