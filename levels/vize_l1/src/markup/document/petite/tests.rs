use super::*;
use vize_l0::{Allocator, SourceRoot};

#[test]
fn retains_original_scope_and_effect_frames() {
    let source = r#"<DIV v-scope="{ n: 1 }"><button v-effect='n++'></button></DIV>"#;
    let arena = Allocator::default();
    let owner = NativeDocument::lex_in(&arena, SourceRoot::new(source).unwrap());
    let mut directives = owner.petite_directives().unwrap();
    let scope = directives.next().unwrap().unwrap();
    assert_eq!(scope.kind(), DocumentPetiteDirectiveKind::Scope);
    assert_eq!(scope.element_name(), "DIV");
    assert_eq!(scope.authored(), r#"v-scope="{ n: 1 }""#);
    assert_eq!(scope.value().unwrap().authored(), "{ n: 1 }");
    let effect = directives.next().unwrap().unwrap();
    assert_eq!(effect.kind(), DocumentPetiteDirectiveKind::Effect);
    assert_eq!(effect.element_name(), "button");
    assert_eq!(effect.authored(), "v-effect='n++'");
    assert_eq!(effect.value().unwrap().authored(), "n++");
    assert!(directives.next().is_none());
}

#[test]
fn preserves_argument_modifiers_and_entity_events() {
    let arena = Allocator::default();
    let source = r#"<div v-scope:arg.one.two="{ label: '&amp;' }"></div>"#;
    let owner = NativeDocument::lex_in(&arena, SourceRoot::new(source).unwrap());
    let directive = owner.petite_directives().unwrap().next().unwrap().unwrap();
    assert_eq!(directive.argument().unwrap().source(), Some("arg"));
    assert!(
        directive
            .modifiers()
            .map(|token| token.source().unwrap())
            .eq(["one", "two"])
    );
    let value = directive.value().unwrap();
    assert_eq!(value.authored(), "{ label: '&amp;' }");
    assert!(value.tokens().any(|token| {
        token.kind() == Kind::AttributeEntity && token.decoded_entity().is_some()
    }));
}

#[test]
fn distinguishes_valueless_and_empty_values() {
    let source = "<div v-scope v-effect=\"\"></div>";
    let arena = Allocator::default();
    let owner = NativeDocument::lex_in(&arena, SourceRoot::new(source).unwrap());
    let mut directives = owner.petite_directives().unwrap();
    assert!(directives.next().unwrap().unwrap().value().is_none());
    assert_eq!(
        directives
            .next()
            .unwrap()
            .unwrap()
            .value()
            .unwrap()
            .authored(),
        ""
    );
    assert!(directives.next().is_none());
}

#[test]
fn refuses_recovered_lexical_input() {
    let arena = Allocator::default();
    let owner = NativeDocument::lex_in(
        &arena,
        SourceRoot::new("<div v-scope='unterminated").unwrap(),
    );
    assert!(matches!(
        owner.petite_directives(),
        Err(DocumentPetiteRefusal::Lexical(_))
    ));
}
