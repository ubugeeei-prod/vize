use super::*;
use vize_l2::file::{FileIssueKind, PositionQueryError};

#[test]
fn optional_named_exports_keep_full_genuine_refusals_with_and_without_keyword_return() {
    let arena = Allocator::default();
    for (source, parameter, references) in [
        (
            "export function f(value?: number) { return value; }",
            "value?: number",
            1,
        ),
        (
            "export function f(value?:number){return value;}f(1);",
            "value?:number",
            2,
        ),
        (
            "export function f(value?: number):number { return value; }",
            "value?: number",
            1,
        ),
    ] {
        let parsed =
            Parser::new(&arena, source, SourceType::ts().with_module(true)).parse_observed();
        let file = finish(&arena, &parsed).unwrap();
        assert!(!file.is_complete());
        assert_eq!(file.scopes().len(), 2);
        assert_eq!(file.bindings().count(), 1);
        let root = file.units()[0].scope;
        let function = file.lookup(root, "f", Namespace::Value).unwrap();
        assert_eq!(file.exports().len(), 1);
        assert_eq!(file.exports()[0].local, Some(function.id()));
        assert!(
            file.lookup(file.scopes()[1].id, "value", Namespace::Value)
                .is_none()
        );
        assert_eq!(
            file.issues()
                .iter()
                .map(|i| (i.kind, i.span.slice(source)))
                .collect::<Vec<_>>(),
            [
                (FileIssueKind::UnsupportedSyntax, parameter),
                (FileIssueKind::UnresolvedReference, "value")
            ]
        );
        assert_eq!(file.references()[0].target, ReferenceTarget::Unresolved);
        assert!(matches!(
            file.binding_at_offset(file.exports()[0].span.start),
            Err(PositionQueryError::IncompleteFile)
        ));
        assert_eq!(file.references().len(), references);
    }
}

#[test]
fn keyword_named_exports_keep_exact_default_type_only_and_source_reexport_roles() {
    let arena = Allocator::default();
    let source = "export default function f(value:number):number{return value;}";
    let parsed = Parser::new(&arena, source, SourceType::ts().with_module(true)).parse_observed();
    let file = finish(&arena, &parsed).unwrap();
    assert!(!file.is_complete());
    assert_eq!(file.scopes().len(), 1);
    assert_eq!(file.bindings().count(), 0);
    assert_eq!(file.exports()[0].name.as_str(), "default");
    assert!(file.exports()[0].local.is_none());
    assert!(file.references().is_empty());
    assert_eq!(
        file.issues()
            .iter()
            .map(|i| (i.kind, i.span.slice(source)))
            .collect::<Vec<_>>(),
        [(
            FileIssueKind::UnsupportedSyntax,
            "function f(value:number):number{return value;}"
        )]
    );
    let source = "export function f(value:number):number{return value;}export type {f};";
    let parsed = Parser::new(&arena, source, SourceType::ts().with_module(true)).parse_observed();
    let file = finish(&arena, &parsed).unwrap();
    assert!(!file.is_complete());
    assert_eq!(file.exports().len(), 2);
    assert_eq!(file.exports()[0].namespace, Namespace::Value);
    assert_eq!(file.exports()[1].namespace, Namespace::Type);
    assert!(file.exports()[1].local.is_none());
    assert_eq!(
        file.issues()
            .iter()
            .map(|i| (i.kind, i.span.slice(source)))
            .collect::<Vec<_>>(),
        [(FileIssueKind::UnresolvedReference, "f")]
    );
    let source = "export function f(value:number):number{return value;}export {other as remote} from './source';export * as space from './namespace';";
    let parsed = Parser::new(&arena, source, SourceType::ts().with_module(true)).parse_observed();
    let file = finish(&arena, &parsed).unwrap();
    assert!(file.is_complete(), "{:?}", file.issues());
    assert_eq!(file.exports().len(), 3);
    for (row, name, request) in [
        (&file.exports()[1], "remote", "./source"),
        (&file.exports()[2], "space", "./namespace"),
    ] {
        assert_eq!(row.name.as_str(), name);
        assert_eq!(row.source.as_ref().unwrap().as_str(), request);
        assert!(row.local.is_none());
        assert_eq!(row.namespace, Namespace::Value);
        assert_eq!(row.unit, file.units()[0].id);
    }
    assert_eq!(file.references().len(), 1);
}

#[test]
fn newly_admitted_named_return_never_hides_original_parameter_body_or_late_errors() {
    let arena = Allocator::default();
    for (source, expected) in [
        (
            "export function f(value:number|string):number{return value;}",
            vec![
                (FileIssueKind::UnsupportedSyntax, "value:number|string"),
                (FileIssueKind::UnresolvedReference, "value"),
            ],
        ),
        (
            "export function f(value:number):number{return missing+value;}",
            vec![(FileIssueKind::UnresolvedReference, "missing")],
        ),
        (
            "export function f(value:number):number{return value;}class Unsupported{}",
            vec![(FileIssueKind::UnsupportedSyntax, "class Unsupported{}")],
        ),
    ] {
        let parsed =
            Parser::new(&arena, source, SourceType::ts().with_module(true)).parse_observed();
        let file = finish(&arena, &parsed).unwrap();
        assert!(!file.is_complete());
        assert_eq!(file.exports().len(), 1);
        assert_eq!(
            file.issues()
                .iter()
                .map(|i| (i.kind, i.span.slice(source)))
                .collect::<Vec<_>>(),
            expected
        );
    }
    for source in [
        "export async function f(value:number):number{return value;}",
        "export function* f(value:number):number{yield value;}",
        "export function f<T>(value:number):number{return value;}",
        "export function f(value:number):number[]{return value;}",
        "export function f(value:number=1):number{return value;}",
        "export function f(value:number):number{'use strict';return value;}",
    ] {
        let parsed =
            Parser::new(&arena, source, SourceType::ts().with_module(true)).parse_observed();
        assert!(
            parsed.admitted().is_some(),
            "{source}: {:?}",
            parsed.diagnostics()
        );
        let file = finish(&arena, &parsed).unwrap();
        assert!(!file.is_complete());
        assert!(!file.issues().is_empty());
        assert_eq!(file.exports().len(), 1);
        assert!(core::ptr::eq(file.artifact().source(), source));
    }
}
