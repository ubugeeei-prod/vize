use super::{finish, parse};
use oxc_parser::{ParseOptions, Parser};
use oxc_span::SourceType;
use vize_l0::{Allocator, SourceRoot, String};
use vize_l2::file::{FileIssueKind, ReferenceTarget};
use vize_l2::lang::js::ProgramInput;

#[test]
fn compatible_var_parameter_redeclarations_stay_explicitly_incomplete() {
    let arena = Allocator::default();
    let parsed = parse(&arena, "function f(value) { var value = 1; return value; }");
    let file = finish(&arena, &parsed).unwrap();
    assert!(!file.is_complete());
    assert_eq!(file.bindings().count(), 2);
    assert!(
        file.issues()
            .iter()
            .any(|issue| issue.kind == FileIssueKind::DuplicateDeclaration)
    );
}

#[test]
fn incomplete_function_fields_and_statement_families_cannot_finish_complete() {
    let arena = Allocator::default();
    for source in [
        "async function f(value) { return value; }",
        "function* f(value) { yield value; }",
        "declare function f(value: number): number;",
        "function f<T>(value) { return value; }",
        "function f(value): number { return value; }",
        "function f(this: object, value) { return value; }",
        "function f(value: number) { return value; }",
        "function f(value?) { return value; }",
        "function f(value = 1) { return value; }",
        "function f(...values) { return values; }",
        "function f({value}) { return value; }",
        "function f([value]) { return value; }",
        "function f(value) { 'use strict'; return value; }",
        "function f(value) { function nested() {} return value; }",
        "function f(value) { if (value) return value; }",
        "function f(value) { { const local = value; } }",
    ] {
        let parsed =
            Parser::new(&arena, source, SourceType::ts().with_module(true)).parse_observed();
        assert!(
            parsed.admitted().is_some(),
            "{source}: {:?}",
            parsed.diagnostics()
        );
        let file = finish(&arena, &parsed).unwrap();
        assert!(!file.is_complete(), "{source}");
        assert!(
            file.issues()
                .iter()
                .any(|issue| issue.kind == FileIssueKind::UnsupportedSyntax),
            "{source}: {:?}",
            file.issues()
        );
        assert_eq!(parsed.admitted().unwrap().program().body.len(), 1);
        assert!(core::ptr::eq(file.artifact().source(), source));
    }
}

#[test]
fn unresolved_implicit_and_outer_uses_keep_actual_partial_owner_facts() {
    let arena = Allocator::default();
    for source in [
        "function f(value) { return missing + value; }",
        "function f(value) { return arguments; }",
    ] {
        let parsed = parse(&arena, source);
        let file = finish(&arena, &parsed).unwrap();
        assert!(!file.is_complete());
        assert_eq!(file.scopes().len(), 2);
        assert_eq!(file.bindings().count(), 2);
        assert_eq!(file.references()[0].scope, file.scopes()[1].id);
        assert_eq!(file.references()[0].target, ReferenceTarget::Unresolved);
        assert!(
            file.issues()
                .iter()
                .any(|issue| issue.kind == FileIssueKind::UnresolvedReference)
        );
        assert_eq!(parsed.admitted().unwrap().program().body.len(), 1);
    }
}

#[test]
fn original_source_and_default_module_authority_cannot_be_substituted_for_functions() {
    let arena = Allocator::default();
    let source = String::from("function f(value) { return value; }");
    let foreign = source.clone();
    let parsed = parse(&arena, &source);
    assert!(matches!(
        ProgramInput::checked(parsed.admitted().unwrap(), SourceRoot::new(&foreign).unwrap().whole_block(), 0),
        Err(error) if error.kind == FileIssueKind::InvalidSource
    ));
    let block = SourceRoot::new(&source).unwrap().whole_block();
    let nondefault = Parser::new(&arena, &source, SourceType::mjs())
        .with_options(ParseOptions {
            allow_return_outside_function: true,
            ..ParseOptions::default()
        })
        .parse_observed();
    assert!(
        matches!(ProgramInput::checked(nondefault.admitted().unwrap(), block, 0), Err(error) if error.kind == FileIssueKind::InvalidProfile)
    );
    for (profile, text) in [
        (SourceType::cjs(), source.as_str()),
        (SourceType::jsx().with_module(true), source.as_str()),
        (
            SourceType::d_ts().with_module(true),
            "declare function f(value: number): number;",
        ),
    ] {
        let parsed = Parser::new(&arena, text, profile).parse_observed();
        let file = finish(&arena, &parsed).unwrap();
        assert!(!file.is_complete());
        assert_eq!(file.bindings().count(), 0);
        assert!(
            file.issues()
                .iter()
                .any(|issue| issue.kind == FileIssueKind::InvalidProfile)
        );
    }
    let outside = Parser::new(&arena, "return;", SourceType::mjs()).parse_observed();
    assert!(outside.admitted().is_none());
}
