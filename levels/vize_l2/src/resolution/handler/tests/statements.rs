use super::{Allocator, Outer, input, resolve_handler};

#[test]
fn declaration_and_return_facts_are_minted_from_the_same_original_body() {
    let arena = Allocator::default();
    for (text, leading) in [
        ("var unused=0; $event.count++;", true),
        ("let unused=$event;", true),
        ("/*kept*/ const unused=0; $event.count++;", true),
        ("$event.count++; var unused=0;", false),
        ("'use strict'; var unused=0; $event.count++;", false),
    ] {
        let original = input(&arena, text);
        let body = original.body();
        let resolution = resolve_handler(original, &Outer(&[])).unwrap();
        assert!(core::ptr::eq(resolution.input().body(), body));
        assert_eq!(resolution.syntax().leading_declaration(), leading);
        assert_eq!(resolution.syntax().first_return(), None);
    }
}

#[test]
fn the_original_first_nested_return_keeps_its_exact_source_window() {
    let arena = Allocator::default();
    let text = "var x=&quot;雪&quot;;{return $event &amp;&amp; 1;}return 0;";
    let resolution = resolve_handler(input(&arena, text), &Outer(&[])).unwrap();
    assert!(resolution.syntax().leading_declaration());
    let returned = resolution.syntax().first_return().unwrap();
    assert_eq!(
        returned.slice(resolution.input().operand().syntax().source().text()),
        "return $event && 1;"
    );
    let authored = resolution.authored_span(returned).unwrap();
    assert_eq!(
        authored.slice(
            resolution
                .input()
                .operand()
                .syntax()
                .source()
                .authored_root()
        ),
        "return $event &amp;&amp; 1;"
    );
}

#[test]
fn late_refusal_rolls_back_statement_facts_with_the_original_scope_tables() {
    let arena = Allocator::default();
    let original = input(&arena, "var unused=0; return $event; while(true){}");
    let mut pending = super::super::sink::Pending::new();
    let source = original.references();
    assert!(crate::resolution::walk::handler_body(&source, &mut pending).is_err());
    let tables = pending.finish(&Outer(&[])).unwrap();
    assert!(!tables.syntax.leading_declaration());
    assert_eq!(tables.syntax.first_return(), None);
    assert_eq!(tables.scopes.len(), 1);
    assert_eq!(tables.bindings.len(), 1);
    assert_eq!(tables.declarations, []);
    assert_eq!(tables.references, []);
    assert_eq!(
        original.operand().syntax().source().text(),
        "var unused=0; return $event; while(true){}"
    );
    assert_eq!(
        original
            .operand()
            .syntax()
            .source()
            .span()
            .slice(original.operand().syntax().source().authored_root()),
        "var unused=0; return $event; while(true){}"
    );
}
