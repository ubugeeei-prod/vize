//! Preserve old inputs as complete neutral positives before backend qualification.
use super::{Allocator, Lang, file};
use vize_l2::file::{DeclarationKind, Namespace, ReferenceTarget};

#[test]
fn all_old_checker_required_named_export_inputs_keep_complete_original_function_rows() {
    let arena = Allocator::default();
    for source in [
        "export function f(value:number){return value;}f(1);",
        "export function f(value:number):number{return value;}f(1);",
    ] {
        let original = file(&arena, source, Lang::Ts);
        assert!(original.is_complete(), "{source}: {:?}", original.issues());
        assert!(original.issues().is_empty());
        assert!(core::ptr::eq(original.artifact().source(), source));
        assert_eq!(original.scopes().len(), 2);
        let function = original
            .lookup(original.units()[0].scope, "f", Namespace::Value)
            .unwrap();
        let parameter = original
            .lookup(original.scopes()[1].id, "value", Namespace::Value)
            .unwrap();
        assert_eq!(
            function.declaration().unwrap().kind,
            DeclarationKind::Function
        );
        assert_eq!(
            parameter.declaration().unwrap().kind,
            DeclarationKind::Parameter
        );
        assert_eq!(original.exports().len(), 1);
        let export = &original.exports()[0];
        assert_eq!(export.local, Some(function.id()));
        assert_eq!(export.namespace, Namespace::Value);
        assert_eq!(export.unit, original.units()[0].id);
        assert_eq!(export.span, function.declaration().unwrap().span);
        assert!(export.source.is_none());
        assert_eq!(original.references().len(), 2);
        assert_eq!(
            original.references()[0].target,
            ReferenceTarget::Resolved(parameter.id())
        );
        assert_eq!(
            original.references()[1].target,
            ReferenceTarget::Resolved(function.id())
        );
    }
}
