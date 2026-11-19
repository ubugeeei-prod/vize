use super::*;
use vize_l0::Span;
use vize_l2::file::{FileIssue, FileIssueKind};

#[test]
fn every_non_keyword_return_retains_the_original_whole_function_partial_issue() {
    let arena = Allocator::default();
    for annotation in [
        "void",
        "never",
        "any",
        "unknown",
        "object",
        "1",
        "'literal'",
        "(number)",
        "number|string",
        "number&string",
        "number[]",
        "[number]",
        "Missing",
        "typeof value",
        "{id:number}",
        "()=>number",
        "value is number",
        "asserts value is number",
    ] {
        let source = cstr!("function f(value:number):{annotation}{{return value;}}");
        let observed =
            Parser::new(&arena, &source, SourceType::ts().with_module(true)).parse_observed();
        assert!(
            observed.admitted().is_some(),
            "{source}: {:?}",
            observed.diagnostics()
        );
        let file = finish(&arena, &observed).unwrap();
        assert!(!file.is_complete());
        assert_eq!(
            file.issues(),
            &[FileIssue {
                unit: file.units()[0].id,
                span: Span::new(0, source.len() as u32),
                kind: FileIssueKind::UnsupportedSyntax,
            }]
        );
        assert_eq!(file.bindings().count(), 1);
        assert_eq!(file.scopes().len(), 1);
        assert!(file.references().is_empty());
        assert!(core::ptr::eq(file.artifact().source(), source.as_str()));
    }
}

#[test]
fn original_direct_required_typed_return_export_and_later_export_keep_same_binding() {
    let arena = Allocator::default();
    let source = "export function f(value:number):number{return value;}";
    let observed = Parser::new(&arena, source, SourceType::ts().with_module(true)).parse_observed();
    let file = finish(&arena, &observed).unwrap();
    let function = file
        .lookup(file.units()[0].scope, "f", Namespace::Value)
        .unwrap();
    assert!(file.is_complete(), "{:?}", file.issues());
    assert!(file.issues().is_empty());
    assert_eq!(file.exports().len(), 1);
    assert_eq!(file.exports()[0].local, Some(function.id()));
    assert_eq!(file.exports()[0].unit, file.units()[0].id);
    assert_eq!(file.exports()[0].namespace, Namespace::Value);
    let parameter = file
        .lookup(file.scopes()[1].id, "value", Namespace::Value)
        .unwrap();
    assert_eq!(file.references().len(), 1);
    assert_eq!(
        file.references()[0].target,
        ReferenceTarget::Resolved(parameter.id())
    );
    for source in [
        "function f(value:number):number{return value;}export {f};",
        "export function f(value){return value;}",
    ] {
        let observed =
            Parser::new(&arena, source, SourceType::ts().with_module(true)).parse_observed();
        let file = finish(&arena, &observed).unwrap();
        assert!(file.is_complete(), "{source}: {:?}", file.issues());
        let function = file
            .lookup(file.units()[0].scope, "f", Namespace::Value)
            .unwrap();
        assert_eq!(file.exports()[0].local, Some(function.id()));
        assert_eq!(file.exports().len(), 1);
        assert_eq!(file.scopes().len(), 2);
    }
}

#[test]
fn primitive_return_does_not_hide_real_unresolved_body_or_late_sibling_issues() {
    let arena = Allocator::default();
    let source = "function f(value:number):number{return missing+value;}";
    let observed = Parser::new(&arena, source, SourceType::ts().with_module(true)).parse_observed();
    let file = finish(&arena, &observed).unwrap();
    assert!(!file.is_complete());
    assert_eq!(
        file.issues(),
        &[FileIssue {
            unit: file.units()[0].id,
            span: Span::new(
                source.find("missing").unwrap() as u32,
                source.find("missing").unwrap() as u32 + 7
            ),
            kind: FileIssueKind::UnresolvedReference,
        }]
    );
    assert_eq!(file.bindings().count(), 2);
    assert_eq!(file.references().len(), 2);
    let source = "function f(value:number):number{return value;}class Unsupported{}";
    let observed = Parser::new(&arena, source, SourceType::ts().with_module(true)).parse_observed();
    let file = finish(&arena, &observed).unwrap();
    assert!(!file.is_complete());
    assert_eq!(
        file.issues(),
        &[FileIssue {
            unit: file.units()[0].id,
            span: Span::new(source.find("class").unwrap() as u32, source.len() as u32),
            kind: FileIssueKind::UnsupportedSyntax,
        }]
    );
    assert_eq!(file.references().len(), 1);
    assert_eq!(file.scopes().len(), 2);
}

#[test]
fn original_keyword_return_never_bypasses_the_existing_function_envelope_guards() {
    let arena = Allocator::default();
    for source in [
        "async function f(value:number):number{return value;}",
        "function* f(value:number):number{yield value;}",
        "function f<T>(value:number):number{return value;}",
        "function f(this:object,value:number):number{return value;}",
        "function f(...value:number[]):number{return value;}",
    ] {
        let observed =
            Parser::new(&arena, source, SourceType::ts().with_module(true)).parse_observed();
        assert!(
            observed.admitted().is_some(),
            "{source}: {:?}",
            observed.diagnostics()
        );
        let file = finish(&arena, &observed).unwrap();
        assert!(!file.is_complete());
        assert_eq!(
            file.issues(),
            &[FileIssue {
                unit: file.units()[0].id,
                span: Span::new(0, source.len() as u32),
                kind: FileIssueKind::UnsupportedSyntax,
            }]
        );
        assert_eq!(file.bindings().count(), 1);
        assert_eq!(file.scopes().len(), 1);
        assert!(file.references().is_empty());
    }
}
