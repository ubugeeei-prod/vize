//! Source-host controls for closure-stable callables. The union-property and
//! nullable-handler listener fixtures stay on the linear projection.
use super::value_binding_origins::ValueReference;
use oxc_span::Span;

fn parse(source: &str) -> super::ScriptParseResult {
    super::parse_script_setup_for_value_origins::<false>(
        source, None, false, false, false, false, None,
    )
}

fn at(result: &super::ScriptParseResult, source: &str, name: &str) -> bool {
    let start = source.rfind(name).expect(name) as u32;
    let scope = result.scopes.scope_at_offset(start).expect("scope");
    let reference = ValueReference::new(scope, name, Span::new(start, start + name.len() as u32));
    result
        .value_binding_origins
        .as_ref()
        .expect("origins")
        .closure_stable_callable(&reference)
}

fn expect_stable(source: &str, name: &str) {
    let result = parse(source);
    let origins = result.value_binding_origins.as_ref().expect("origins");
    assert!(origins.is_complete(), "{source}");
    assert!(at(&result, source, name), "{source}");
}

fn expect_unstable(source: &str, name: &str) {
    let result = parse(source);
    let origins = result.value_binding_origins.as_ref().expect("origins");
    assert!(origins.is_complete(), "{source}");
    assert!(!at(&result, source, name), "{source}");
}

#[test]
fn direct_functions_and_their_aliases_are_closure_stable() {
    for source in [
        "const handler = (value: string) => value; void handler;",
        "const handler = async (value: string) => value; void handler;",
        "const handler = function (value: string) { return value; }; void handler;",
        "function handler(value: string) { return value; } void handler;",
        "const handler = (value: string) => value; const alias = handler; void alias;",
        "const handler = (value: string) => value; const alias = (handler); void alias;",
        "const handler = (value: string) => value; const label = 'callable' as const; void handler;",
        "const outer = (value: string) => value; { const handler = 1; void handler; } void outer;",
    ] {
        let name = if source.contains("void alias") {
            "alias"
        } else if source.contains("void outer") {
            "outer"
        } else {
            "handler"
        };
        expect_stable(source, name);
    }
}

#[test]
fn type_wrappers_calls_members_and_shadows_are_not_callable_authority() {
    for (source, name) in [
        (
            "const handler = ((value: string) => value) as (value: string) => void; void handler;",
            "handler",
        ),
        (
            "const handler = ((value: string) => value) satisfies (value: string) => void; void handler;",
            "handler",
        ),
        (
            "const handler = (value: string) => value; const alias = handler!; void alias;",
            "alias",
        ),
        (
            "const handler = (value: string) => value; const alias = (handler as (value: string) => void); void alias;",
            "alias",
        ),
        (
            "const handler = Math.random() < 0.5 ? (value: string) => value : null; void handler;",
            "handler",
        ),
        (
            "const handler = Math.random() < 0.5 ? Object.assign((value: string) => value, { kind: 'callable' as const }) : { kind: 'other' as const }; void handler;",
            "handler",
        ),
        (
            "const handler = (value: string) => value; const kind = handler.kind; void kind;",
            "kind",
        ),
        (
            "let handler = (value: string) => value; void handler;",
            "handler",
        ),
        (
            "const handler = (value: string) => value; { const handler = 1; void handler; }",
            "handler",
        ),
        (
            "const outer = (value: string) => value; const callback = (handler: string) => handler;",
            "handler",
        ),
        ("const { handler } = opaque(); void handler;", "handler"),
    ] {
        expect_unstable(source, name);
    }
}

#[test]
fn var_and_named_function_expressions_refuse_the_domain() {
    for source in [
        "var handler = (value: string) => value; void handler;",
        "const handler = function handler(value: string) { return value; }; void handler;",
    ] {
        let result = parse(source);
        let origins = result.value_binding_origins.as_ref().expect("origins");
        assert!(!origins.is_complete(), "{source}");
        assert!(!at(&result, source, "handler"), "{source}");
    }
}

#[test]
fn a_value_import_is_not_a_closure_stable_callable() {
    let source = "import { onSelect } from './row'; void onSelect;";
    let result = parse(source);
    let origins = result.value_binding_origins.as_ref().expect("origins");
    assert!(origins.is_complete());
    assert!(origins.is_syntactic_value_import("onSelect"));
    assert!(!at(&result, source, "onSelect"));
}
