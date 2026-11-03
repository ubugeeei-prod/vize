use super::{Allocator, DeclarationKind, Namespace, Parser, SourceType, finish};
use vize_l0::cstr;
use vize_l2::file::{FileIssueKind, ReferenceTarget};

#[test]
fn unsupported_optional_types_retain_both_original_issue_sites() {
    let arena = Allocator::default();
    for annotation in [
        "T",
        "Namespace.T",
        "number[]",
        "{id:number}",
        "number|string",
        "[number]",
        "any",
        "unknown",
        "never",
        "void",
    ] {
        let source = cstr!("function f(value?: {annotation}) {{ return value; }}");
        let observed =
            Parser::new(&arena, &source, SourceType::ts().with_module(true)).parse_observed();
        assert!(
            observed.admitted().is_some(),
            "{annotation}: {:?}",
            observed.diagnostics()
        );
        let file = finish(&arena, &observed).unwrap();
        assert!(!file.is_complete(), "{annotation}");
        assert_eq!(file.scopes().len(), 2);
        assert_eq!(file.bindings().count(), 1);
        assert!(
            file.lookup(file.scopes()[1].id, "value", Namespace::Value)
                .is_none()
        );
        let sites = file
            .issues()
            .iter()
            .map(|issue| (issue.kind, issue.span.slice(&source)))
            .collect::<Vec<_>>();
        assert_eq!(
            sites,
            [
                (
                    FileIssueKind::UnsupportedSyntax,
                    cstr!("value?: {annotation}").as_str()
                ),
                (FileIssueKind::UnresolvedReference, "value")
            ],
            "{annotation}"
        );
        assert_eq!(file.references().len(), 1);
        assert_eq!(file.references()[0].target, ReferenceTarget::Unresolved);
        assert_eq!(observed.admitted().unwrap().program().body.len(), 1);
    }
}

#[test]
fn direct_optional_typed_exports_refuse_and_later_export_events_survive() {
    let arena = Allocator::default();
    let source = "export function f(value?: number) { return value; }";
    let observed = Parser::new(&arena, source, SourceType::ts().with_module(true)).parse_observed();
    let file = finish(&arena, &observed).unwrap();
    assert!(!file.is_complete());
    assert_eq!(file.exports().len(), 1);
    let function = file
        .lookup(file.units()[0].scope, "f", Namespace::Value)
        .unwrap();
    assert_eq!(file.exports()[0].local, Some(function.id()));
    assert_eq!(
        file.issues()
            .iter()
            .map(|issue| (issue.kind, issue.span.slice(source)))
            .collect::<Vec<_>>(),
        [
            (FileIssueKind::UnsupportedSyntax, "value?: number"),
            (FileIssueKind::UnresolvedReference, "value")
        ]
    );
    for source in [
        "export function f(value) { return value; }",
        "function f(value?: number) { return value; } export { f };",
    ] {
        let observed =
            Parser::new(&arena, source, SourceType::ts().with_module(true)).parse_observed();
        let file = finish(&arena, &observed).unwrap();
        assert!(file.is_complete(), "{source}: {:?}", file.issues());
        assert_eq!(file.exports().len(), 1);
        let root = file.units()[0].scope;
        let function = file.lookup(root, "f", Namespace::Value).unwrap();
        let parameter = file
            .lookup(file.scopes()[1].id, "value", Namespace::Value)
            .unwrap();
        assert_eq!(
            parameter.declaration().unwrap().kind,
            DeclarationKind::Parameter
        );
        assert_eq!(file.exports()[0].local, Some(function.id()));
        assert_eq!(
            file.references()[0].target,
            ReferenceTarget::Resolved(parameter.id())
        );
    }
}

#[test]
fn optional_keyword_does_not_complete_other_function_or_body_fields() {
    let arena = Allocator::default();
    for source in [
        "async function f(value?:number){return value;}",
        "function* f(value?:number){yield value;}",
        "function f<T>(value?:number){return value;}",
        "function f(this:object,value?:number){return value;}",
        "function f(value:number=1){return value;}",
        "function f(...value:number[]){return value;}",
        "function f({value}:{value:number}){return value;}",
        "function f([value]:number[]){return value;}",
        "function f(value?:number){'use strict';return value;}",
        "function f(value?:number){function nested(){return value;}return value;}",
        "function f(value?:number){if(value)return value;}",
        "function f(value?:number){{const local=value;}return value;}",
        "function f(value?:number){return missing+value;}",
        "function f(value?:number){var value=1;return value;}",
        "function f(value?:number){return value;}class Unsupported{}",
    ] {
        let observed =
            Parser::new(&arena, source, SourceType::ts().with_module(true)).parse_observed();
        assert!(
            observed.admitted().is_some(),
            "{source}: {:?}",
            observed.diagnostics()
        );
        let body = observed.admitted().unwrap().program().body.as_ptr();
        let file = finish(&arena, &observed).unwrap();
        assert!(!file.is_complete(), "{source}");
        assert!(!file.issues().is_empty(), "{source}");
        assert!(core::ptr::eq(file.artifact().source(), source));
        assert_eq!(observed.admitted().unwrap().program().body.as_ptr(), body);
    }
}

#[test]
fn the_exact_unannotated_optional_negative_remains_a_whole_two_site_refusal() {
    let arena = Allocator::default();
    let source = "function f(value?) { return value; }";
    let observed = Parser::new(&arena, source, SourceType::ts().with_module(true)).parse_observed();
    let file = finish(&arena, &observed).unwrap();
    assert!(!file.is_complete());
    assert_eq!(file.bindings().count(), 1);
    assert_eq!(
        file.issues()
            .iter()
            .map(|issue| (issue.kind, issue.span.slice(source)))
            .collect::<Vec<_>>(),
        [
            (FileIssueKind::UnsupportedSyntax, "value?"),
            (FileIssueKind::UnresolvedReference, "value"),
        ]
    );
    assert_eq!(file.references().len(), 1);
    assert_eq!(file.references()[0].target, ReferenceTarget::Unresolved);
    assert!(
        file.lookup(file.scopes()[1].id, "value", Namespace::Value)
            .is_none()
    );
    assert!(core::ptr::eq(file.artifact().source(), source));
}
