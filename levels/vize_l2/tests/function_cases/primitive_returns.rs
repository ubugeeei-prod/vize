mod custody;
mod rejection;
mod setup;

use super::{finish, finish_block};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::{Allocator, SourceRoot, cstr};
use vize_l2::file::{DeclarationKind, Namespace, ReferenceTarget};
use vize_l2::lang::js::JsxFileProducer;

#[test]
fn all_three_original_primitive_return_refusals_now_keep_complete_neutral_facts() {
    let arena = Allocator::default();
    // Exact former inputs from the original, typed-parameter and Canon lists.
    for source in [
        "function f(value): number { return value; }",
        "function f(value:number):number{return value;}",
        "function f(value:number):number{return value;}f(1);",
    ] {
        let observed =
            Parser::new(&arena, source, SourceType::ts().with_module(true)).parse_observed();
        let program = observed.admitted().unwrap().program();
        let file = finish(&arena, &observed).unwrap();
        assert!(file.is_complete(), "{source}: {:?}", file.issues());
        assert!(file.issues().is_empty());
        assert_eq!(file.units().len(), 1);
        assert_eq!(file.units()[0].id.index(), 7);
        assert_eq!(file.scopes().len(), 2);
        let root = file.units()[0].scope;
        let scope = file.scopes()[1].id;
        assert_eq!(file.scopes()[1].parent, Some(root));
        let function = file.lookup(root, "f", Namespace::Value).unwrap();
        let parameter = file.lookup(scope, "value", Namespace::Value).unwrap();
        assert_eq!(
            function.declaration().unwrap().kind,
            DeclarationKind::Function
        );
        assert_eq!(
            parameter.declaration().unwrap().kind,
            DeclarationKind::Parameter
        );
        assert_eq!(parameter.declaration().unwrap().span.slice(source), "value");
        assert_eq!(file.bindings().count(), 2);
        assert_eq!(
            file.references()[0].target,
            ReferenceTarget::Resolved(parameter.id())
        );
        assert_eq!(file.references()[0].scope, scope);
        if source.ends_with("f(1);") {
            assert_eq!(file.references().len(), 2);
            assert_eq!(
                file.references()[1].target,
                ReferenceTarget::Resolved(function.id())
            );
            assert_eq!(file.references()[1].scope, root);
        } else {
            assert_eq!(file.references().len(), 1);
        }
        assert!(file.lookup(scope, "number", Namespace::Type).is_none());
        assert!(file.ordinary_empty_script().is_none());
        assert!(core::ptr::eq(
            observed.admitted().unwrap().program(),
            program
        ));
        assert!(core::ptr::eq(file.artifact().source(), source));
    }
}

#[test]
fn all_seven_original_keyword_returns_have_no_fabricated_type_reads() {
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
        let source = cstr!("function f(value:{keyword}):{keyword}{{return value;}}");
        let observed =
            Parser::new(&arena, &source, SourceType::ts().with_module(true)).parse_observed();
        let file = finish(&arena, &observed).unwrap();
        assert!(file.is_complete(), "{keyword}: {:?}", file.issues());
        assert_eq!(file.bindings().count(), 2);
        assert_eq!(file.references().len(), 1);
        let parameter = file
            .lookup(file.scopes()[1].id, "value", Namespace::Value)
            .unwrap();
        assert_eq!(
            file.references()[0].target,
            ReferenceTarget::Resolved(parameter.id())
        );
        assert_eq!(file.references()[0].namespace, Namespace::Value);
        assert!(
            file.lookup(file.scopes()[1].id, keyword, Namespace::Type)
                .is_none()
        );
        assert!(file.units()[0].profile.typescript);
        assert!(file.units()[0].profile.module);
        assert!(!file.units()[0].profile.jsx);
        assert_eq!(parameter.declaration().unwrap().unit, file.units()[0].id);
    }
}

#[test]
fn original_return_scopes_restore_recursion_forward_reads_and_sibling_shadowing() {
    let arena = Allocator::default();
    let source = "const value=1;function f(value:number):number{const local=value+later;return f(local);}const later=1;let after=value;function g(value:string):string{return value;}";
    let observed = Parser::new(&arena, source, SourceType::ts().with_module(true)).parse_observed();
    let file = finish(&arena, &observed).unwrap();
    assert!(file.is_complete(), "{:?}", file.issues());
    assert_eq!(file.scopes().len(), 3);
    let root = file.units()[0].scope;
    let first = file.scopes()[1].id;
    let second = file.scopes()[2].id;
    assert_eq!(file.scopes()[1].parent, Some(root));
    assert_eq!(file.scopes()[2].parent, Some(root));
    let outer = file.lookup(root, "value", Namespace::Value).unwrap();
    let parameter = file.lookup(first, "value", Namespace::Value).unwrap();
    let sibling = file.lookup(second, "value", Namespace::Value).unwrap();
    assert_ne!(outer.id(), parameter.id());
    assert_ne!(parameter.id(), sibling.id());
    let local = file.lookup(first, "local", Namespace::Value).unwrap();
    let later = file.lookup(root, "later", Namespace::Value).unwrap();
    let function = file.lookup(root, "f", Namespace::Value).unwrap();
    assert_eq!(file.references().len(), 6);
    for (reference, expected) in file.references().iter().zip([
        parameter.id(),
        later.id(),
        function.id(),
        local.id(),
        outer.id(),
        sibling.id(),
    ]) {
        assert_eq!(reference.target, ReferenceTarget::Resolved(expected));
        assert_eq!(reference.unit, file.units()[0].id);
        assert_eq!(reference.span.slice(source), reference.name.as_str());
    }
    assert!(file.lookup(second, "local", Namespace::Value).is_none());
    assert_eq!(file.references()[4].scope, root);
}

#[test]
fn genuine_tsx_owner_retains_each_original_keyword_return_and_fragment_on_move() {
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
        let source = cstr!(
            "/* original */ function f(value:{keyword}):{keyword}{{return value;}} const view=<></>;view;"
        );
        let block = SourceRoot::new(&source).unwrap().whole_block();
        let observed =
            Parser::new(&arena, &source, SourceType::tsx().with_module(true)).parse_observed();
        let body = observed.admitted().unwrap().program().body.as_ptr();
        let mut producer = JsxFileProducer::new(&arena, observed, block, 7)
            .unwrap_or_else(|rejected| panic!("original TSX: {:?}", rejected.error()));
        producer.walk().unwrap();
        let owner =
            Box::new(producer.finish().unwrap_or_else(|rejected| {
                panic!("complete TSX {keyword}: {:?}", rejected.error())
            }));
        let file = owner.file();
        assert!(file.is_complete(), "{:?}", file.issues());
        assert!(file.units()[0].profile.typescript);
        assert!(file.units()[0].profile.jsx);
        assert!(file.units()[0].profile.module);
        let parameter = file
            .lookup(file.scopes()[1].id, "value", Namespace::Value)
            .unwrap();
        assert_eq!(
            file.references()[0].target,
            ReferenceTarget::Resolved(parameter.id())
        );
        assert_eq!(owner.observation().comments().len(), 1);
        assert_eq!(
            owner
                .observation()
                .admitted()
                .unwrap()
                .program()
                .body
                .as_ptr(),
            body
        );
        assert!(owner.nodes().any(|node| node.source() == Some("<></>")));
        assert!(core::ptr::eq(file.artifact().source(), source.as_str()));
        let at = source.find("return value").unwrap() as u32 + 7;
        for _ in 0..2 {
            let queried = file.binding_at_offset(at).unwrap().unwrap();
            assert_eq!(queried.id(), parameter.id());
            assert!(core::ptr::eq(queried.file(), file));
        }
    }
}
