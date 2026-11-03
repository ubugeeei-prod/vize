use super::{finish, finish_block, parse};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::{Allocator, SourceRoot};
use vize_l2::file::{DeclarationKind, Namespace, ReferenceTarget};

#[test]
fn authentic_function_scope_resolves_parameters_forward_closures_and_recursion_once() {
    let arena = Allocator::default();
    let source = "function f(value) { const local = value + later; return f(local); } const later = 1; let after = later;";
    let parsed = parse(&arena, source);
    let file = finish(&arena, &parsed).unwrap();
    assert!(file.is_complete(), "{:?}", file.issues());
    assert_eq!(file.scopes().len(), 2);
    assert_eq!(file.scopes()[1].parent, Some(file.scopes()[0].id));
    assert_eq!(file.bindings().count(), 5);
    let root = file.units()[0].scope;
    let child = file.scopes()[1].id;
    let f = file.lookup(root, "f", Namespace::Value).unwrap();
    let parameter = file.lookup(child, "value", Namespace::Value).unwrap();
    let local = file.lookup(child, "local", Namespace::Value).unwrap();
    let later = file.lookup(root, "later", Namespace::Value).unwrap();
    assert_eq!(
        parameter.declaration().unwrap().kind,
        DeclarationKind::Parameter
    );
    assert_eq!(file.references().len(), 5);
    for (reference, expected) in
        file.references()
            .iter()
            .zip([parameter.id(), later.id(), f.id(), local.id(), later.id()])
    {
        assert_eq!(reference.target, ReferenceTarget::Resolved(expected));
        assert_eq!(reference.unit, file.units()[0].id);
        assert_eq!(
            source.get(reference.span.start as usize..reference.span.end as usize),
            Some(reference.name.as_str())
        );
    }
    assert!(file.lookup(root, "value", Namespace::Value).is_none());
    assert!(file.lookup(root, "local", Namespace::Value).is_none());
    assert_eq!(file.references()[4].scope, root);
}

#[test]
fn sibling_functions_restore_their_real_parent_and_keep_shadowed_identities_distinct() {
    let arena = Allocator::default();
    let source = "const value = 1; function f(value) { let local = value; return local; } let after = value; function g() { return value; }";
    let parsed = parse(&arena, source);
    let file = finish(&arena, &parsed).unwrap();
    assert!(file.is_complete());
    assert_eq!(file.scopes().len(), 3);
    let root = file.units()[0].scope;
    let first = file.scopes()[1].id;
    let second = file.scopes()[2].id;
    let outer = file.lookup(root, "value", Namespace::Value).unwrap();
    let parameter = file.lookup(first, "value", Namespace::Value).unwrap();
    assert_ne!(outer.id(), parameter.id());
    assert_eq!(
        file.lookup(second, "value", Namespace::Value).unwrap().id(),
        outer.id()
    );
    assert!(file.lookup(second, "local", Namespace::Value).is_none());
    assert_eq!(file.references()[0].scope, first);
    assert_eq!(file.references()[2].scope, root);
    assert_eq!(file.references()[3].scope, second);
    assert_eq!(
        file.references()[2].target,
        ReferenceTarget::Resolved(outer.id())
    );
    assert_eq!(
        file.references()[3].target,
        ReferenceTarget::Resolved(outer.id())
    );
}

#[test]
fn named_exports_and_unicode_parameters_keep_original_full_file_sites() {
    let arena = Allocator::default();
    let source = "é<script>/* retained */ export function 表示(値) { return 値; }</script>";
    let start = source.find("/*").unwrap();
    let end = source.find("</script>").unwrap();
    let block = SourceRoot::new(source)
        .unwrap()
        .block(source.get(start..end).unwrap(), start as u32)
        .unwrap();
    let parsed = Parser::new(&arena, block.source(), SourceType::mjs()).parse_observed();
    let program = parsed.admitted().unwrap().program();
    let file = finish_block(&arena, &parsed, block).unwrap();
    assert!(file.is_complete());
    assert_eq!(program.comments.len(), 1);
    assert_eq!(program.body.len(), 1);
    let function = file
        .lookup(file.units()[0].scope, "表示", Namespace::Value)
        .unwrap();
    let declaration = function.declaration().unwrap();
    assert_eq!(declaration.kind, DeclarationKind::Function);
    assert_eq!(
        declaration.span.start as usize,
        source.find("表示").unwrap()
    );
    assert_eq!(file.exports()[0].local, Some(function.id()));
    assert_eq!(file.exports()[0].unit.index(), 7);
    let parameter = file
        .lookup(file.scopes()[1].id, "値", Namespace::Value)
        .unwrap();
    assert_eq!(
        file.references()[0].target,
        ReferenceTarget::Resolved(parameter.id())
    );
    assert!(core::ptr::eq(parsed.admitted().unwrap().program(), program));
    assert_eq!(
        source.get(file.scopes()[1].span.start as usize..file.scopes()[1].span.end as usize),
        Some("function 表示(値) { return 値; }")
    );
}

#[test]
fn plain_typescript_functions_and_bare_return_use_the_same_checked_producer() {
    let arena = Allocator::default();
    let source = "function f(value) { value; return; }";
    let parsed = Parser::new(&arena, source, SourceType::ts().with_module(true)).parse_observed();
    let file = finish(&arena, &parsed).unwrap();
    assert!(file.is_complete());
    assert!(file.units()[0].profile.typescript);
    assert_eq!(file.references().len(), 1);
    assert_eq!(file.bindings().count(), 2);
}
