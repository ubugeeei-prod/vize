use super::{Allocator, Namespace, SourceType, file, offset};

#[test]
fn declaration_and_reference_keep_their_original_file_and_resolved_identity() {
    let arena = Allocator::default();
    let source = "const first = later; const later = 1; first;";
    let owner = file(&arena, source, SourceType::mjs());
    assert!(owner.is_complete());
    let declaration = owner
        .binding_at_offset(offset(source, "later ="))
        .unwrap()
        .unwrap();
    let reference = owner
        .reference_at_offset(offset(source, "later;"))
        .unwrap()
        .unwrap();
    assert!(core::ptr::eq(reference.file(), &owner));
    assert!(core::ptr::eq(reference.reference(), &owner.references()[0]));
    assert!(declaration.same_owner(reference.binding().unwrap()));
    assert_eq!(declaration.id(), reference.binding().unwrap().id());
    assert_eq!(
        declaration.id(),
        owner
            .binding_at_offset(offset(source, "later;"))
            .unwrap()
            .unwrap()
            .id()
    );
    assert!(
        owner
            .reference_at_offset(offset(source, "later ="))
            .unwrap()
            .is_none()
    );

    let second = file(&arena, source, SourceType::mjs());
    let other = second
        .binding_at_offset(offset(source, "later ="))
        .unwrap()
        .unwrap();
    assert_eq!(declaration.id(), other.id());
    assert!(!declaration.same_owner(other));
}

#[test]
fn import_and_export_aliases_use_local_sites_without_inventing_remote_bindings() {
    let arena = Allocator::default();
    let source = "import {remote as local} from 'dep'; export {local as publicName}; export {remote as external} from 'other'; local;";
    let owner = file(&arena, source, SourceType::mjs());
    assert!(owner.is_complete());
    let local = owner
        .binding_at_offset(offset(source, "local}"))
        .unwrap()
        .unwrap();
    let declaration = local.declaration().unwrap();
    assert_eq!(declaration.name, "local");
    assert_eq!(declaration.imported_name.as_deref(), Some("remote"));
    let exported_use = owner
        .reference_at_offset(offset(source, "local as public"))
        .unwrap()
        .unwrap();
    assert_eq!(exported_use.binding().unwrap().id(), local.id());
    for needle in [
        "remote as local",
        "publicName",
        "remote as external",
        "external",
        "'dep'",
    ] {
        let at = offset(source, needle);
        assert!(owner.binding_at_offset(at).unwrap().is_none(), "{needle}");
        assert!(owner.reference_at_offset(at).unwrap().is_none(), "{needle}");
    }
}

#[test]
fn type_only_imports_and_export_uses_preserve_the_original_namespace() {
    let arena = Allocator::default();
    let source = "import type {Thing as Local} from 'types'; export type {Local as Public};";
    let owner = file(&arena, source, SourceType::ts().with_module(true));
    assert!(owner.is_complete());
    let local = owner
        .binding_at_offset(offset(source, "Local}"))
        .unwrap()
        .unwrap();
    assert_eq!(local.declaration().unwrap().namespace, Namespace::Type);
    let reference = owner
        .reference_at_offset(offset(source, "Local as Public"))
        .unwrap()
        .unwrap();
    assert_eq!(reference.reference().namespace, Namespace::Type);
    assert_eq!(reference.binding().unwrap().id(), local.id());
    let scope = owner
        .scope_at_offset(offset(source, "Local as Public"))
        .unwrap()
        .unwrap();
    assert!(scope.lookup("Local", Namespace::Value).is_none());
    assert_eq!(
        scope.lookup("Local", Namespace::Type).unwrap().id(),
        local.id()
    );
    assert!(
        owner
            .binding_at_offset(offset(source, "Public"))
            .unwrap()
            .is_none()
    );
}

#[test]
fn function_name_queries_retain_the_parent_while_parameters_and_body_use_the_child() {
    let arena = Allocator::default();
    let source =
        "const value = 1; function fnName(value) { const local = value; return local; } value;";
    let owner = file(&arena, source, SourceType::mjs());
    assert!(owner.is_complete());
    let root = owner.units()[0].scope;
    let function_name = owner
        .scope_at_offset(offset(source, "fnName"))
        .unwrap()
        .unwrap();
    assert_eq!(function_name.scope().id, root);
    let parameter = owner
        .binding_at_offset(offset(source, "value)"))
        .unwrap()
        .unwrap();
    let child = owner
        .scope_at_offset(offset(source, "value)"))
        .unwrap()
        .unwrap();
    assert_ne!(child.scope().id, root);
    assert!(core::ptr::eq(child.file(), &owner));
    assert_eq!(child.scope().id, parameter.declaration().unwrap().scope);
    assert_eq!(
        child.lookup("value", Namespace::Value).unwrap().id(),
        parameter.id()
    );
    assert_eq!(
        owner
            .scope_at_offset(offset(source, "const local"))
            .unwrap()
            .unwrap()
            .scope()
            .id,
        child.scope().id
    );
    assert_eq!(
        owner
            .scope_at_offset(offset(source, "return local"))
            .unwrap()
            .unwrap()
            .scope()
            .id,
        child.scope().id
    );
    let after = offset(source, "} value") + 1;
    assert_eq!(
        owner.scope_at_offset(after).unwrap().unwrap().scope().id,
        root
    );
    let final_use = source.rfind("value").unwrap() as u32;
    assert_eq!(
        owner
            .scope_at_offset(final_use)
            .unwrap()
            .unwrap()
            .scope()
            .id,
        root
    );
    assert_ne!(
        owner.binding_at_offset(final_use).unwrap().unwrap().id(),
        parameter.id()
    );
}
