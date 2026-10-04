use super::{Allocator, InitializerKind, Namespace, Parser, SourceRoot, SourceType, lower};
use oxc_ast::ast::{Statement, TSType};

#[test]
fn original_array_nodes_keep_authored_declarations_and_value_navigation() {
    let arena = Allocator::default();
    let source = "é<script>/* 🌸 */const seed=1;const items: { id: number }[] = [{ id: seed }];items[0].id;</script>";
    let start = source.find("/*").unwrap();
    let end = source.find("</script>").unwrap();
    let block = SourceRoot::new(source)
        .unwrap()
        .block(source.get(start..end).unwrap(), start as u32)
        .unwrap();
    let observed =
        Parser::new(&arena, block.source(), SourceType::ts().with_module(true)).parse_observed();
    assert!(observed.diagnostics().is_empty());
    let original = observed.admitted().unwrap().program();
    let Statement::VariableDeclaration(declaration) = &original.body[1] else {
        panic!("actual original variable");
    };
    let annotation = declaration.declarations[0]
        .type_annotation
        .as_ref()
        .unwrap();
    let TSType::TSArrayType(array) = &annotation.type_annotation else {
        panic!("actual original array");
    };
    let TSType::TSTypeLiteral(element) = &array.element_type else {
        panic!("actual original element");
    };
    assert_eq!(element.members.len(), 1);
    assert_eq!(
        block
            .source()
            .get(annotation.span.start as usize..annotation.span.end as usize),
        Some(": { id: number }[]")
    );
    let file = lower(&arena, source, block, &observed);
    assert!(file.is_complete());
    assert!(core::ptr::eq(file.artifact().source(), source));
    assert!(core::ptr::eq(
        observed.admitted().unwrap().program(),
        original
    ));
    assert_eq!(original.comments.len(), 1);
    assert_eq!(file.units()[0].span, block.span());
    assert_eq!(file.units()[0].id.index(), 7);
    let scope = file.units()[0].scope;
    let items = file.lookup(scope, "items", Namespace::Value).unwrap();
    assert_eq!(
        items.declaration().unwrap().initializer,
        InitializerKind::Unknown
    );
    assert!(file.lookup(scope, "items", Namespace::Type).is_none());
    assert_eq!(file.references().len(), 2);
    for (reference, name) in file.references().iter().zip(["seed", "items"]) {
        let bound = file
            .binding_at_offset(reference.span.start)
            .unwrap()
            .unwrap();
        assert!(core::ptr::eq(bound.file(), &file));
        assert_eq!(bound.declaration().unwrap().name.as_str(), name);
        assert_eq!(
            source.get(reference.span.start as usize..reference.span.end as usize),
            Some(name)
        );
    }
    let type_property = source.find("id: number").unwrap() as u32;
    assert!(file.binding_at_offset(type_property).unwrap().is_none());
    let declaration_id = items.id();
    let moved = Box::new(file);
    assert_eq!(
        moved
            .binding_at_offset(source.rfind("items").unwrap() as u32)
            .unwrap()
            .unwrap()
            .id(),
        declaration_id
    );
}

#[test]
fn original_import_rows_and_default_expression_do_not_gain_ordinary_eligibility() {
    let arena = Allocator::default();
    let source = "import {items as original} from 'dependency';const items:number[]=original;export default {items};";
    let observed = Parser::new(&arena, source, SourceType::ts().with_module(true)).parse_observed();
    assert!(observed.diagnostics().is_empty());
    let file = lower(
        &arena,
        source,
        SourceRoot::new(source).unwrap().whole_block(),
        &observed,
    );
    assert!(file.is_complete());
    assert!(file.ordinary_empty_script().is_none());
    assert_eq!(file.imports().len(), 1);
    assert_eq!(file.exports().len(), 1);
    let scope = file.units()[0].scope;
    let import = file.lookup(scope, "original", Namespace::Value).unwrap();
    assert_eq!(
        import.declaration().unwrap().imported_name.as_deref(),
        Some("items")
    );
    assert!(file.lookup(scope, "original", Namespace::Type).is_none());
    for reference in file.references() {
        assert!(
            file.binding_at_offset(reference.span.start)
                .unwrap()
                .is_some()
        );
    }
}
