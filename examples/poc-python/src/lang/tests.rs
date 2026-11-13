use super::{expression, script};

#[test]
fn rustpython_ast_expressions_lower_to_javascript() {
    let cases = [
        ("count > 3 and not done", "count > 3 && !done"),
        ("'hot' if done else 'cold'", "done ? \"hot\" : \"cold\""),
        ("f'Hello, {name}!'", "`Hello, ${name}!`"),
        ("len(items) == 3", "items.length === 3"),
        ("item.upper()", "item.toUpperCase()"),
        ("ref(False)", "ref(false)"),
        ("items[0]", "items[0]"),
        ("(left + right) * scale", "(left + right) * scale"),
    ];
    for (python, javascript) in cases {
        assert_eq!(expression(python).as_deref(), Ok(javascript), "{python}");
    }
}

#[test]
fn valid_but_unsupported_python_is_refused() {
    assert!(expression("value in items").is_err());
    assert!(expression("value // 2").is_err());
    assert!(expression("{'name': name}").is_err());
    assert!(expression("fn(value=1)").is_err());
}

#[test]
fn parser_diagnostics_retain_the_source_offset() {
    let error = script("value = (\n", 40).expect_err("invalid Python must fail");
    assert_eq!(error.message, "Python parse error");
    assert!(error.span.start >= 48);
    assert_eq!(error.span.end, error.span.start + 1);
}

#[test]
fn unsupported_statement_uses_the_rustpython_range() {
    let error = script("value = 1\nwhile value:\n    value -= 1\n", 10)
        .expect_err("while is outside the setup lowering surface");
    assert_eq!(error.message, "unsupported Python syntax");
    assert_eq!(error.span.start, 20);
    assert_eq!(error.span.end, 47);
}
