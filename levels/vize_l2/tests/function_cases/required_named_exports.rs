use super::{Allocator, Parser, SourceType, finish, finish_block};
use vize_l0::{SourceRoot, cstr};
use vize_l2::file::{DeclarationKind, Namespace, ReferenceTarget};

#[test]
fn every_old_required_named_export_source_survives_as_complete_original_positive() {
    let arena = Allocator::default();
    for source in [
        "export function f(value: number) { return value; }",
        "export function f(value:number):number{return value;}",
        "export function f(value:number){return value;}f(1);",
        "export function f(value:number):number{return value;}f(1);",
    ] {
        let parsed =
            Parser::new(&arena, source, SourceType::ts().with_module(true)).parse_observed();
        let program = parsed.admitted().unwrap().program();
        let file = finish(&arena, &parsed).unwrap();
        assert!(file.is_complete(), "{source}: {:?}", file.issues());
        assert!(file.issues().is_empty());
        assert_eq!(file.scopes().len(), 2);
        let function = file
            .lookup(file.units()[0].scope, "f", Namespace::Value)
            .unwrap();
        let parameter = file
            .lookup(file.scopes()[1].id, "value", Namespace::Value)
            .unwrap();
        assert_eq!(
            function.declaration().unwrap().kind,
            DeclarationKind::Function
        );
        assert_eq!(
            parameter.declaration().unwrap().kind,
            DeclarationKind::Parameter
        );
        let export = &file.exports()[0];
        assert_eq!(file.exports().len(), 1);
        assert_eq!(export.local, Some(function.id()));
        assert_eq!(export.namespace, Namespace::Value);
        assert_eq!(export.unit, file.units()[0].id);
        assert_eq!(export.name, "f");
        assert_eq!(export.span, function.declaration().unwrap().span);
        assert!(export.source.is_none());
        assert_eq!(
            file.references()[0].target,
            ReferenceTarget::Resolved(parameter.id())
        );
        assert!(core::ptr::eq(parsed.admitted().unwrap().program(), program));
        assert!(core::ptr::eq(file.artifact().source(), source));
    }
}

#[test]
fn original_named_parameter_only_return_only_combined_and_untyped_roles_stay_distinct() {
    let arena = Allocator::default();
    for source in [
        "export function f(value:number){return value;}",
        "export function f(value):number{return value;}",
        "export function f(value:number):number{return value;}",
        "export function f(value){return value;}",
    ] {
        let parsed =
            Parser::new(&arena, source, SourceType::ts().with_module(true)).parse_observed();
        let file = finish(&arena, &parsed).unwrap();
        assert!(file.is_complete(), "{source}: {:?}", file.issues());
        assert!(file.issues().is_empty());
        assert_eq!(file.scopes().len(), 2);
        assert_eq!(file.bindings().count(), 2);
        let function = file
            .lookup(file.units()[0].scope, "f", Namespace::Value)
            .unwrap();
        let parameter = file
            .lookup(file.scopes()[1].id, "value", Namespace::Value)
            .unwrap();
        assert_eq!(file.exports().len(), 1);
        assert_eq!(file.exports()[0].local, Some(function.id()));
        assert_eq!(file.exports()[0].span, function.declaration().unwrap().span);
        assert_eq!(
            file.references()[0].target,
            ReferenceTarget::Resolved(parameter.id())
        );
        assert!(
            file.lookup(file.units()[0].scope, "f", Namespace::Type)
                .is_none()
        );
    }
}

#[test]
fn seven_original_required_keyword_exports_keep_one_actual_value_row_and_parameter() {
    let arena = Allocator::default();
    for keyword in [
        "bigint",
        "boolean",
        "null",
        "number",
        "string",
        "symbol",
        "undefined",
    ] {
        let source = cstr!("export function f(value:{keyword}):{keyword}{{return value;}}");
        let parsed =
            Parser::new(&arena, &source, SourceType::ts().with_module(true)).parse_observed();
        let file = finish(&arena, &parsed).unwrap();
        assert!(file.is_complete(), "{keyword}: {:?}", file.issues());
        let function = file
            .lookup(file.units()[0].scope, "f", Namespace::Value)
            .unwrap();
        let parameter = file
            .lookup(file.scopes()[1].id, "value", Namespace::Value)
            .unwrap();
        assert_eq!(file.exports()[0].local, Some(function.id()));
        assert_eq!(file.exports()[0].namespace, Namespace::Value);
        assert_eq!(file.bindings().count(), 2);
        assert_eq!(file.references().len(), 1);
        assert_eq!(
            file.references()[0].target,
            ReferenceTarget::Resolved(parameter.id())
        );
        assert!(
            file.lookup(file.units()[0].scope, "f", Namespace::Type)
                .is_none()
        );
        assert!(
            file.lookup(file.scopes()[1].id, keyword, Namespace::Type)
                .is_none()
        );
    }
}

#[test]
fn named_function_exports_keep_recursion_forward_calls_alias_and_sibling_scope_order() {
    let arena = Allocator::default();
    let source = "first(1);export function first(value:number):number{return second(value);}export function second(value:number):number{return first(value);}export {first as alias};";
    let parsed = Parser::new(&arena, source, SourceType::ts().with_module(true)).parse_observed();
    let file = finish(&arena, &parsed).unwrap();
    assert!(file.is_complete(), "{:?}", file.issues());
    assert_eq!(file.scopes().len(), 3);
    let root = file.units()[0].scope;
    let first = file.lookup(root, "first", Namespace::Value).unwrap();
    let second = file.lookup(root, "second", Namespace::Value).unwrap();
    let a = file
        .lookup(file.scopes()[1].id, "value", Namespace::Value)
        .unwrap();
    let b = file
        .lookup(file.scopes()[2].id, "value", Namespace::Value)
        .unwrap();
    assert_ne!(a.id(), b.id());
    assert_eq!(
        file.exports()
            .iter()
            .map(|e| (e.name.as_str(), e.local))
            .collect::<Vec<_>>(),
        [
            ("first", Some(first.id())),
            ("second", Some(second.id())),
            ("alias", Some(first.id()))
        ]
    );
    let targets = [
        first.id(),
        second.id(),
        a.id(),
        first.id(),
        b.id(),
        first.id(),
    ];
    assert_eq!(file.references().len(), targets.len());
    for (reference, target) in file.references().iter().zip(targets) {
        assert_eq!(reference.target, ReferenceTarget::Resolved(target));
    }
    assert_eq!(file.references()[0].scope, root);
    assert_eq!(file.references()[1].scope, file.scopes()[1].id);
    assert_eq!(file.references()[3].scope, file.scopes()[2].id);
    assert_eq!(file.references()[5].scope, root);
    assert!(
        file.references()
            .windows(2)
            .all(|pair| pair[0].span.start < pair[1].span.start)
    );
}

mod custody;
mod rejection;
mod setup;
