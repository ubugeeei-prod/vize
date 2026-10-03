use super::{Allocator, Outer, ResolutionErrorKind, input, resolve_handler};

#[test]
fn unsupported_statement_expression_and_binding_forms_keep_precise_original_spans() {
    let allocator = Allocator::default();
    for (text, expected) in [
        ("while(x)y++", "while(x)y++"),
        ("for(;;)x++", "for(;;)x++"),
        ("throw x", "throw x"),
        ("try{}finally{}", "try{}finally{}"),
        ("function f(){}", "function f(){}"),
        ("class X{}", "class X{}"),
        ("let [x]=outer", "[x]"),
        ("first;()=>missing", "()=>missing"),
        ("eval('x')", "eval('x')"),
    ] {
        let original = input(&allocator, text);
        let body = original.body();
        let decoded = original.operand().syntax().source().text();
        let rejected =
            resolve_handler(original, &Outer(&[("first", 1), ("outer", 2)])).unwrap_err();
        assert_eq!(
            rejected.error.kind,
            ResolutionErrorKind::UnsupportedSyntax,
            "{text}"
        );
        assert_eq!(rejected.error.span.slice(decoded), expected, "{text}");
        assert!(core::ptr::eq(rejected.input().body(), body));
        assert_eq!(rejected.into_input().operand().raw_value(), text);
    }
}

#[test]
fn real_var_lexical_and_implicit_parameter_conflicts_refuse_the_collision_site() {
    let allocator = Allocator::default();
    for (text, name) in [
        ("let x;var x", "x"),
        ("var x;let x", "x"),
        ("{let x;{var x}}", "x"),
        ("{var x;let x}", "x"),
        ("let x;let x", "x"),
        ("let $event=1", "$event"),
    ] {
        let original = input(&allocator, text);
        let decoded = original.operand().syntax().source().text();
        let rejected = resolve_handler(original, &Outer(&[])).unwrap_err();
        assert_eq!(
            rejected.error.kind,
            ResolutionErrorKind::UnsupportedSyntax,
            "{text}"
        );
        assert_eq!(rejected.error.span.slice(decoded), name, "{text}");
    }
}

#[test]
fn disjoint_block_lexicals_do_not_conflict_with_an_outside_var() {
    let allocator = Allocator::default();
    for text in ["{let x}var x", "var x;{let x}", "{let x}{var x}"] {
        let resolution = resolve_handler(input(&allocator, text), &Outer(&[])).unwrap();
        assert_eq!(resolution.bindings().len(), 3);
        assert_eq!(resolution.declarations().len(), 2);
        assert_ne!(
            resolution.declarations()[0].binding,
            resolution.declarations()[1].binding
        );
    }
}
