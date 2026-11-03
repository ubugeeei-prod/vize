extern crate std;

use super::super::borrowed::resolve_for_facts;
use super::*;

#[test]
fn borrowed_facts_join_only_the_complete_original_owner_after_pending_growth() {
    let arena = Allocator::default();
    let file = "頭<script lang='ts'></script><template><b v-for='(α, key) in β'/></template>";
    let selected = selected(&arena, file);
    let original = input(&selected);
    let alias_root = original.aliases().as_ptr();
    let collection_root = original.collection() as *const _;
    let whole_source = original.operand().syntax().source();
    let mut pending = alloc::vec::Vec::new();
    pending.push(original);
    let facts = resolve_for_facts(&pending[0], &Lookup::new(&[("β", 42)])).unwrap();
    pending.reserve(32);
    let table = facts.join(pending.pop().unwrap()).unwrap();
    assert_eq!(table.input().aliases().as_ptr(), alias_root);
    assert_eq!(table.input().collection() as *const _, collection_root);
    assert!(core::ptr::eq(
        table.input().operand().syntax().source().text(),
        whole_source.text()
    ));
    assert!(core::ptr::eq(
        table.input().operand().syntax().source().authored_root(),
        file
    ));
    assert_eq!(table.value().decoded_span(), Span::new(0, 2));
    assert_eq!(table.key().unwrap().decoded_span(), Span::new(4, 7));
    assert_eq!(table.collection().span, Span::new(0, 2));
    assert_eq!(table.value().authored_span().slice(file), "α");
    assert_eq!(table.collection_authored_span().slice(file), "β");
    assert!(core::ptr::eq(
        table.value_declaration().parameter(),
        &table.input().aliases()[0]
    ));
}

#[test]
fn borrowed_facts_refuse_an_independent_equal_file_and_return_its_whole_input() {
    let arena = Allocator::default();
    let file = "<template><b v-for='item in items'/></template>";
    let selected = selected(&arena, file);
    let foreign = super::selected(&arena, file);
    let original = input(&selected);
    let foreign_input = input(&foreign);
    let foreign_aliases = foreign_input.aliases().as_ptr();
    let foreign_collection = foreign_input.collection() as *const _;
    let facts = resolve_for_facts(&original, &Lookup::new(&[("items", 8)])).unwrap();
    let rejected = facts.join(foreign_input).unwrap_err();
    assert_eq!(rejected.aliases().as_ptr(), foreign_aliases);
    assert_eq!(rejected.collection() as *const _, foreign_collection);
    assert_eq!(rejected.operand().raw_value(), "item in items");
    let element = foreign.children().next().unwrap().into_element().unwrap();
    assert!(
        rejected
            .admitted_for(&foreign, element.attributes().next().unwrap())
            .is_some()
    );
    assert!(core::ptr::eq(
        original.aliases(),
        original
            .operand()
            .syntax()
            .admitted_dense()
            .unwrap()
            .aliases()
            .parameters()
            .items
            .as_slice()
    ));
}

#[test]
fn borrowed_facts_refuse_an_equal_spelling_sibling_original_attribute() {
    let arena = Allocator::default();
    let file = "<template><b v-for='item in items' v-for='item in items'/></template>";
    let selected = selected(&arena, file);
    let element = selected.children().next().unwrap().into_element().unwrap();
    let mut attributes = element.attributes();
    let first = attributes.next().unwrap();
    let second = attributes.next().unwrap();
    let original = NativeForInput::new(
        selected
            .observe_attribute_for_head(first.reborrow())
            .unwrap(),
    )
    .unwrap();
    let sibling = NativeForInput::new(
        selected
            .observe_attribute_for_head(second.reborrow())
            .unwrap(),
    )
    .unwrap();
    let sibling_aliases = sibling.aliases().as_ptr();
    let facts = resolve_for_facts(&original, &Lookup::new(&[("items", 8)])).unwrap();
    let rejected = facts.join(sibling).unwrap_err();
    assert_eq!(rejected.aliases().as_ptr(), sibling_aliases);
    assert!(rejected.admitted_for(&selected, second).is_some());
    assert!(rejected.admitted_for(&selected, first).is_none());
}

struct UnwindLookup;
impl BindingLookup for UnwindLookup {
    fn lookup(&self, _: &str) -> Option<BindingId> {
        panic!("authored enclosing lookup interruption");
    }
}

#[test]
fn original_owner_stays_parked_when_the_actual_collection_lookup_unwinds() {
    let arena = Allocator::default();
    let file = "<template><b v-for='(item, key) in items'/></template>";
    let selected = selected(&arena, file);
    let original = input(&selected);
    let aliases = original.aliases().as_ptr();
    let collection = original.collection() as *const _;
    let pending = original;
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        resolve_for_facts(&pending, &UnwindLookup)
    }));
    assert!(result.is_err());
    assert_eq!(pending.aliases().as_ptr(), aliases);
    assert_eq!(pending.collection() as *const _, collection);
    assert_eq!(pending.operand().raw_value(), "(item, key) in items");
    let element = selected.children().next().unwrap().into_element().unwrap();
    assert!(
        pending
            .admitted_for(&selected, element.attributes().next().unwrap())
            .is_some()
    );
    assert!(pending.operand().syntax().aliases().unwrap().is_ok());
    assert!(pending.operand().syntax().collection().unwrap().is_ok());
    let facts = resolve_for_facts(&pending, &Lookup::new(&[("items", 9)])).unwrap();
    assert_eq!(
        facts.join(pending).unwrap().collection().binding,
        BindingId::new(9)
    );
}
