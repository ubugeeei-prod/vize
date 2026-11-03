use super::{
    Allocator, TemplateSiteScope, TemplateSymbolRef, at, attempt, file, native, site, uses,
};

#[test]
fn original_for_alias_declarations_and_collection_keep_distinct_real_file_origins() {
    let arena = Allocator::default();
    let source = "<script setup>const items=2;</script><template><div @click='item' v-for='(item,index) in items'><span @blur='index'/></div></template>";
    let output = native(&arena, source);
    let file = file(&output);
    let alias = site(file, source, "item,index");
    let key = site(file, source, "index) in");
    let collection = site(file, source, "items'>");
    let use_alias = site(file, source, "item' v-for");
    let use_key = site(file, source, "index'/>");
    assert!(alias.symbol().same_symbol(use_alias.symbol()));
    assert!(key.symbol().same_symbol(use_key.symbol()));
    assert!(!alias.symbol().same_symbol(collection.symbol()));
    assert_eq!(uses(file, alias.symbol()), [use_alias.span()]);
    assert_eq!(uses(file, key.symbol()), [use_key.span()]);
    let TemplateSymbolRef::File(binding) = alias.symbol() else {
        panic!("template alias")
    };
    assert!(binding.declaration().is_none());
    let row = binding.template_declaration().unwrap();
    let original = row.declaration().original().unwrap();
    let head = alias.for_head().unwrap();
    assert_eq!(row.declaration().origin(), head.id());
    assert_eq!(original.authored_span(), alias.span());
    assert!(core::ptr::eq(
        original.resolution(),
        head.resolution().unwrap()
    ));
    let TemplateSiteScope::File(alias_scope) = alias.scope() else {
        panic!("alias scope")
    };
    let TemplateSiteScope::File(collection_scope) = collection.scope() else {
        panic!("collection scope")
    };
    assert_ne!(alias_scope, collection_scope);
    assert_eq!(Some(alias_scope), head.scope());
    assert_eq!(Some(collection_scope), head.enclosing_scope());
}

#[test]
fn original_nested_for_collection_resolves_parent_alias_before_its_own_shadowing() {
    let arena = Allocator::default();
    let source = "<script setup>const items=2;</script><template><div v-for='item in items'><span v-for='item in item' @click='item'/><span @blur='item'/></div></template>";
    let output = native(&arena, source);
    let file = file(&output);
    let parent = site(file, source, "item in items");
    let nested = site(file, source, "item in item'");
    let collection = site(file, source, "item' @click");
    let inner_use = site(file, source, "item'/><span");
    let outer_use = site(file, source, "item'/></div>");
    assert!(collection.symbol().same_symbol(parent.symbol()));
    assert!(outer_use.symbol().same_symbol(parent.symbol()));
    assert!(inner_use.symbol().same_symbol(nested.symbol()));
    assert!(!nested.symbol().same_symbol(parent.symbol()));
    assert_eq!(
        uses(file, parent.symbol()),
        [collection.span(), outer_use.span()]
    );
    assert_eq!(uses(file, nested.symbol()), [inner_use.span()]);
    assert!(
        file.template_symbol_at_offset(at(source, " in "))
            .unwrap()
            .is_none()
    );
    assert!(
        file.template_symbol_at_offset(parent.span().end)
            .unwrap()
            .is_none()
    );
}

#[test]
fn original_for_unicode_sites_and_entity_refusals_keep_actual_source_namespaces() {
    let arena = Allocator::default();
    let source = "<script setup>const items=2;</script><template><div v-for='café in items' @click='café'/></template>";
    let output = native(&arena, source);
    let file = file(&output);
    let alias = site(file, source, "café in");
    let collection = site(file, source, "items' @click");
    let reference = site(file, source, "café'/>");
    assert_eq!(alias.span().slice(source), "café");
    assert_eq!(collection.span().slice(source), "items");
    assert_eq!(reference.span().slice(source), "café");
    assert!(alias.symbol().same_symbol(reference.symbol()));
    assert!(!alias.symbol().same_symbol(collection.symbol()));
    for byte in collection.span().start..collection.span().end {
        assert!(
            file.template_symbol_at_offset(byte)
                .unwrap()
                .unwrap()
                .symbol()
                .same_symbol(collection.symbol())
        );
    }
    assert!(
        file.template_symbol_at_offset(collection.span().end)
            .unwrap()
            .is_none()
    );
    assert_eq!(uses(file, alias.symbol()), [reference.span()]);
    assert!(matches!(
        file.template_symbol_at_offset(alias.span().start + 4),
        Err(super::TemplateQueryError::Position(
            super::PositionQueryError::NotCharBoundary
        ))
    ));

    // The current genuine native For family refuses entity output itself.
    // Preserve that original refusal, not a positive query over a partial File.
    let encoded = "<script setup>const items=2;</script><template><div v-for='caf&#233; in it&#101;ms' @click='café'/></template>";
    let refused = attempt(&arena, encoded, false);
    assert!(refused.view().is_err());
    let partial = refused.file().unwrap();
    let [crate::file::RejectedFileFor::Syntax(original)] = partial.rejected_for_heads() else {
        panic!("original native For syntax refusal")
    };
    assert_eq!(
        original.kind,
        vize_l1::embed::syntax::NativeForRefusal::EntityOutput
    );
    assert_eq!(original.operand().raw_value(), "caf&#233; in it&#101;ms");
    assert!(original.operand().syntax().source().decode_map().is_some());
    assert!(matches!(
        partial.template_symbol_at_offset(at(encoded, "caf&#233;")),
        Err(super::TemplateQueryError::Position(
            super::PositionQueryError::IncompleteFile
        ))
    ));
}
