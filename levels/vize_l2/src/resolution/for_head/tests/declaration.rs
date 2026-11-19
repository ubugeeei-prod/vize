use super::*;
use oxc_ast::ast::BindingPattern;
use oxc_span::GetSpan;

#[test]
fn original_ts_unicode_declarations_retain_exact_parameter_pattern_and_coordinates() {
    let arena = Allocator::default();
    let file = "頭<script lang='ts'></script><template><b v-for='(α, key) in β'/></template>";
    let owner = selected(&arena, file);
    let original = input(&owner);
    let value_root = &original.aliases()[0];
    let key_root = &original.aliases()[1];
    let table = resolve_for_head(original, &Lookup::new(&[("β", 42)])).unwrap();
    let value = table.value_declaration();
    let key = table.key_declaration().unwrap();
    assert!(core::ptr::eq(value.parameter(), value_root));
    assert!(core::ptr::eq(key.parameter(), key_root));
    assert!(core::ptr::eq(value.pattern(), &value_root.pattern));
    assert!(core::ptr::eq(key.pattern(), &key_root.pattern));
    assert!(core::ptr::eq(value.resolution(), &table));
    assert!(core::ptr::eq(key.resolution(), &table));
    assert_eq!(value.fact(), table.value());
    assert_eq!(key.fact(), table.key().unwrap());
    assert_eq!(value.fact().role(), ForAliasRole::Value);
    assert_eq!(key.fact().role(), ForAliasRole::Key);
    assert_eq!(value.decoded_span(), Span::new(0, 2));
    assert_eq!(key.decoded_span(), Span::new(4, 7));
    assert_eq!(value.authored_span().slice(file), "α");
    assert_eq!(key.authored_span().slice(file), "key");
    let BindingPattern::BindingIdentifier(binding) = value.pattern() else {
        panic!("the admitted original declaration must remain a binding identifier");
    };
    assert_eq!(binding.name.as_str(), value.fact().name());
    assert_eq!(
        table.input().alias_decoded_span(binding.span).unwrap(),
        value.decoded_span()
    );
    assert_eq!(
        table
            .input()
            .alias_authored_span(value.parameter().span())
            .unwrap(),
        value.authored_span()
    );
    assert_eq!(table.collection().span, Span::new(0, 2));
    assert_eq!(table.collection_authored_span().slice(file), "β");
}

#[test]
fn single_alias_keeps_one_original_declaration_without_a_synthetic_key() {
    let arena = Allocator::default();
    for file in [
        "<template><b v-for='item in items'/></template>",
        "<template><b v-for='item of items'/></template>",
    ] {
        let owner = selected(&arena, file);
        let original = input(&owner);
        let root = &original.aliases()[0];
        let table = resolve_for_head(original, &Lookup::new(&[("items", 7)])).unwrap();
        assert!(core::ptr::eq(table.value_declaration().parameter(), root));
        assert_eq!(table.value_declaration().fact(), table.value());
        assert!(table.key_declaration().is_none());
        assert!(!core::mem::needs_drop::<ForAliasDeclaration<'_, '_>>());
        assert!(core::mem::needs_drop::<ForResolution<'_>>());
    }
}

#[test]
fn equal_alias_facts_do_not_substitute_another_original_header_owner() {
    let arena = Allocator::default();
    let file = "<template><b v-for='item in items' v-for='item in items'/></template>";
    let owner = selected(&arena, file);
    let foreign = selected(&arena, file);
    let element = owner.children().next().unwrap().into_element().unwrap();
    let mut attributes = element.attributes();
    let first = attributes.next().unwrap();
    let second = attributes.next().unwrap();
    let first_input =
        NativeForInput::new(owner.observe_attribute_for_head(first.reborrow()).unwrap()).unwrap();
    let second_input =
        NativeForInput::new(owner.observe_attribute_for_head(second.reborrow()).unwrap()).unwrap();
    let first_table = resolve_for_head(first_input, &Lookup::new(&[("items", 5)])).unwrap();
    let second_table = resolve_for_head(second_input, &Lookup::new(&[("items", 5)])).unwrap();
    let first_alias = first_table.value_declaration();
    let second_alias = second_table.value_declaration();
    assert_eq!(first_alias.fact().name(), second_alias.fact().name());
    assert_eq!(first_alias.fact().id(), second_alias.fact().id());
    assert!(!core::ptr::eq(
        first_alias.parameter(),
        second_alias.parameter()
    ));
    assert!(
        first_alias
            .resolution()
            .input()
            .admitted_for(&owner, first)
            .is_some()
    );
    assert!(
        first_alias
            .resolution()
            .input()
            .admitted_for(&owner, second)
            .is_none()
    );
    let foreign_element = foreign.children().next().unwrap().into_element().unwrap();
    assert!(
        first_alias
            .resolution()
            .input()
            .admitted_for(&foreign, foreign_element.attributes().next().unwrap(),)
            .is_none()
    );
    assert!(core::ptr::eq(first_alias.resolution(), &first_table));
    assert!(core::ptr::eq(second_alias.resolution(), &second_table));
}
