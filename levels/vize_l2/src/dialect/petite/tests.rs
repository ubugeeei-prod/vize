//! L2 petite-vue owner and coordinate laws.

use oxc_ast::ast::{Expression, PropertyKey};
use oxc_span::GetSpan;
use vize_l0::{Allocator, SourceRoot, Span};
use vize_l1::markup::document::NativeDocument;

use super::{PetiteDirectiveKind as Kind, PetiteDocument, PetiteScopeShape};
use crate::expr::{ExprRef, OpaqueReason};

#[test]
fn scope_and_effect_keep_distinct_l2_meanings() {
    let allocator = Allocator::default();
    let source = r#"<div v-scope="{ count: 0 }" v-effect="count++; count += 1"></div>"#;
    let owner = NativeDocument::lex_in(&allocator, SourceRoot::new(source).unwrap());
    let document = PetiteDocument::lower_in(&allocator, &owner).unwrap();
    let mut directives = document.directives();

    let scope = directives.next().unwrap();
    assert_eq!(scope.kind(), Kind::Scope);
    assert_eq!(scope.scope().unwrap().shape(), PetiteScopeShape::Object);
    assert!(matches!(scope.expression(), Some(ExprRef::Js(_))));
    assert!(core::ptr::eq(
        document.directive_at(Kind::Scope, scope.span()).unwrap(),
        scope
    ));
    assert!(document.directive_at(Kind::Effect, scope.span()).is_none());

    let effect = directives.next().unwrap();
    assert_eq!(effect.kind(), Kind::Effect);
    assert!(!effect.effect().unwrap().is_empty());
    assert!(matches!(
        effect.expression(),
        Some(ExprRef::Opaque(value)) if value.reason == OpaqueReason::ParseRejected
    ));
    assert!(directives.next().is_none());
    assert!(core::ptr::eq(document.owner(), &owner));
}

#[test]
fn decoded_entities_retain_exact_authored_js_coordinates() {
    let allocator = Allocator::default();
    let source = r#"<div v-scope="{ label: '&amp;' }"></div>"#;
    let owner = NativeDocument::lex_in(&allocator, SourceRoot::new(source).unwrap());
    let document = PetiteDocument::lower_in(&allocator, &owner).unwrap();
    let directive = document.directives().next().unwrap();
    assert_eq!(directive.value().unwrap().decoded(), "{ label: '&' }");

    let Some(ExprRef::Js(js)) = directive.expression() else {
        panic!("decoded JS expression");
    };
    let Expression::ObjectExpression(object) = js.ast else {
        panic!("object expression");
    };
    let oxc_ast::ast::ObjectPropertyKind::ObjectProperty(property) = &object.properties[0] else {
        panic!("object property");
    };
    let PropertyKey::StaticIdentifier(key) = &property.key else {
        panic!("static key");
    };
    let decoded = js.ast_span_to_source(key.span()).unwrap();
    let authored = js.authored_span(decoded).unwrap();
    let expected = source.find("label").unwrap() as u32;
    assert_eq!(authored, Span::new(expected, expected + 5));
    assert!(js.matches_authored_source(source));
}

#[test]
fn empty_and_non_object_scopes_are_distinct() {
    let allocator = Allocator::default();
    let source = "<main v-scope><i v-scope='count'></i><b v-effect=\"\"></b></main>";
    let owner = NativeDocument::lex_in(&allocator, SourceRoot::new(source).unwrap());
    let document = PetiteDocument::lower_in(&allocator, &owner).unwrap();
    let mut directives = document.directives();
    assert_eq!(
        directives.next().unwrap().scope().unwrap().shape(),
        PetiteScopeShape::Empty
    );
    assert_eq!(
        directives.next().unwrap().scope().unwrap().shape(),
        PetiteScopeShape::Other
    );
    assert!(directives.next().unwrap().effect().unwrap().is_empty());
    assert!(directives.next().is_none());
}

#[test]
fn lexical_failure_refuses_the_whole_semantic_view() {
    let allocator = Allocator::default();
    let owner = NativeDocument::lex_in(
        &allocator,
        SourceRoot::new("<div v-scope='unterminated").unwrap(),
    );
    assert!(PetiteDocument::lower_in(&allocator, &owner).is_err());
}
