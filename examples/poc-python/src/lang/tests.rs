use poc_support::vue::Binding;

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
        ("(2 ** 3) ** 2", "(2 ** 3) ** 2"),
        ("2 ** 3 ** 2", "2 ** 3 ** 2"),
        ("-2 ** 2", "-(2 ** 2)"),
        ("-(-count)", "-(-count)"),
        ("3 < (2 < 1)", "3 < (2 < 1)"),
        (r#"f"\\{name}""#, r#"`\\${name}`"#),
        (r#""line\n\"quoted\"\\tail""#, r#""line\n\"quoted\"\\tail""#),
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

#[test]
fn aliased_vue_ref_retains_template_ref_semantics() {
    let (imports, body, bindings) =
        script("from vue import ref as make_ref\ncount = make_ref(0)\n", 0).unwrap();
    assert_eq!(imports, "import { ref as make_ref } from \"vue\";\n");
    assert_eq!(body, "const count = make_ref(0);");
    assert_eq!(bindings.get("count"), Some(Binding::Ref));
}

#[test]
fn javascript_binding_names_and_top_level_redeclarations_are_rejected() {
    assert!(script("default = 1\n", 0).is_err());
    assert!(script("def default():\n    return 1\n", 0).is_err());
    assert!(script("def valid(default):\n    return default\n", 0).is_err());
    assert!(script("from vue import ref as default\n", 0).is_err());
    assert!(script("count = 0\ncount = 1\n", 0).is_err());
}

#[test]
fn function_assignment_targets_are_local_and_declared_once() {
    let (_, body, _) = script(
        "def increment(value):\n    count = value\n    count = count + 1\n    return count\n",
        0,
    )
    .unwrap();
    assert_eq!(
        body,
        "function increment(value) {\n  let count = value;\n  count = count + 1;\n  return count;\n}"
    );
}
