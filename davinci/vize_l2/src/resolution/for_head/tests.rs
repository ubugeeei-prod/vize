use super::*;
use core::cell::Cell;
use vize_l0::config::{VueDialect, VueVersion};
use vize_l0::{Allocator, Span};
use vize_l1::container::{Vue, vue::DescriptorOptions};
use vize_l1::embed::Lang;
use vize_l1::markup::NativeTemplateComponent;

fn selected<'a>(arena: &'a Allocator, source: &'a str) -> NativeTemplateComponent<'a> {
    let descriptor = Vue.observe_descriptor(
        arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: vize_l1::SurfaceParseOptions::default(),
        },
    );
    NativeTemplateComponent::parse_in(arena, descriptor.admitted().unwrap())
        .unwrap()
        .unwrap()
}

fn input<'a>(selected: &NativeTemplateComponent<'a>) -> NativeForInput<'a> {
    let element = selected.children().next().unwrap().into_element().unwrap();
    let attribute = element.attributes().next().unwrap();
    NativeForInput::new(selected.observe_attribute_for_head(attribute).unwrap()).unwrap()
}

struct Lookup<'a> {
    bindings: &'a [(&'a str, u32)],
    visits: Cell<usize>,
}
impl<'a> Lookup<'a> {
    fn new(bindings: &'a [(&'a str, u32)]) -> Self {
        Self {
            bindings,
            visits: Cell::new(0),
        }
    }
}
impl BindingLookup for Lookup<'_> {
    fn lookup(&self, name: &str) -> Option<BindingId> {
        self.visits.set(self.visits.get() + 1);
        self.bindings
            .iter()
            .find(|(binding, _)| *binding == name)
            .map(|(_, id)| BindingId::new(*id))
    }
}

#[test]
fn collection_resolves_enclosing_before_same_named_aliases_are_introduced() {
    let arena = Allocator::default();
    let owner = selected(
        &arena,
        "<template><b v-for='(items, key) in items'/></template>",
    );
    let lookup = Lookup::new(&[("items", 0), ("key", 99), ("other", 7)]);
    let table = resolve_for_head(input(&owner), &lookup).unwrap();
    assert_eq!(table.collection().name, "items");
    assert_eq!(table.collection().binding, BindingId::new(0));
    assert_eq!(table.collection().usage, super::super::Usage::Read);
    assert!(!table.collection().shorthand);
    assert!(!table.collection().constructor);
    assert_eq!(lookup.visits.get(), 1);
    assert_eq!(table.value().role(), ForAliasRole::Value);
    assert_eq!(table.value().id().index(), 0);
    assert_eq!(table.key().unwrap().role(), ForAliasRole::Key);
    assert_eq!(table.key().unwrap().id().index(), 1);
    assert_eq!(
        table.lookup_body("items", &lookup),
        Some(ForResolvedBinding::Local(table.value().id()))
    );
    assert_eq!(
        table.lookup_body("key", &lookup),
        Some(ForResolvedBinding::Local(table.key().unwrap().id()))
    );
    assert_eq!(lookup.visits.get(), 1);
    assert_eq!(
        table.lookup_body("other", &lookup),
        Some(ForResolvedBinding::Enclosing(BindingId::new(7)))
    );
    assert_eq!(table.lookup_body("missing", &lookup), None);
    assert_ne!(
        ForResolvedBinding::Local(table.value().id()),
        ForResolvedBinding::Enclosing(BindingId::new(0))
    );
}

#[test]
fn actual_ts_unicode_alias_and_collection_keep_distinct_exact_source_coordinates() {
    let arena = Allocator::default();
    let file = "頭<script lang='ts'></script><template><b v-for='(α, key) in β'/></template>";
    let owner = selected(&arena, file);
    let table = resolve_for_head(input(&owner), &Lookup::new(&[("β", 42)])).unwrap();
    assert_eq!(table.input().operand().syntax().grammar().lang, Lang::Ts);
    assert_eq!(table.value().decoded_span(), Span::new(0, 2));
    assert_eq!(table.key().unwrap().decoded_span(), Span::new(4, 7));
    assert_eq!(table.collection().span, Span::new(0, 2));
    let value_start = file.find("(α, key)").unwrap() as u32;
    assert_eq!(
        table.value().authored_span(),
        Span::new(value_start + 1, value_start + 3)
    );
    assert_eq!(
        table.key().unwrap().authored_span(),
        Span::new(value_start + 5, value_start + 8)
    );
    assert_eq!(
        table.collection_authored_span(),
        Span::new(value_start + 13, value_start + 15)
    );
    assert_eq!(table.value().authored_span().slice(file), "α");
    assert_eq!(table.key().unwrap().authored_span().slice(file), "key");
    assert_eq!(table.collection_authored_span().slice(file), "β");
}

#[test]
fn single_alias_in_and_of_have_no_synthetic_key_or_declaration_range() {
    let arena = Allocator::default();
    for file in [
        "<template><b v-for='item in items'/></template>",
        "<template><b v-for='item of items'/></template>",
    ] {
        let owner = selected(&arena, file);
        let table = resolve_for_head(input(&owner), &Lookup::new(&[("items", 17)])).unwrap();
        assert_eq!(table.value().name(), "item");
        assert_eq!(table.value().decoded_span(), Span::new(0, 4));
        assert_eq!(table.value().authored_span().slice(file), "item");
        assert_eq!(table.key(), None);
        assert_eq!(table.collection().span, Span::new(0, 5));
        assert_eq!(table.collection().binding, BindingId::new(17));
    }
}

#[test]
fn normal_resolution_owner_moves_and_rejoins_only_the_original_selected_header() {
    let arena = Allocator::default();
    let file =
        "<template><b v-for='(item, key) in items' v-for='(item, key) in items'/></template>";
    let owner = selected(&arena, file);
    let foreign = selected(&arena, file);
    let element = owner.children().next().unwrap().into_element().unwrap();
    let mut attributes = element.attributes();
    let first = attributes.next().unwrap();
    let second = attributes.next().unwrap();
    let original =
        NativeForInput::new(owner.observe_attribute_for_head(first.reborrow()).unwrap()).unwrap();
    let aliases = original.aliases().as_ptr();
    let collection = original.collection() as *const _;
    let table = resolve_for_head(original, &Lookup::new(&[("items", 3)])).unwrap();
    assert!(core::mem::needs_drop::<ForResolution<'_>>());
    let mut pending = alloc::vec::Vec::new();
    pending.push(table);
    pending.reserve(32);
    let table = pending.pop().unwrap();
    assert_eq!(table.input().aliases().as_ptr(), aliases);
    assert_eq!(table.input().collection() as *const _, collection);
    assert_eq!(table.value_declaration().parameter() as *const _, aliases);
    assert!(core::ptr::eq(
        table.value_declaration().resolution(),
        &table
    ));
    let key = table.key_declaration().unwrap();
    assert!(core::ptr::eq(key.parameter(), &table.input().aliases()[1]));
    assert_eq!(key.fact(), table.key().unwrap());
    assert!(table.input().admitted_for(&owner, first).is_some());
    assert!(table.input().admitted_for(&owner, second).is_none());
    let foreign_element = foreign.children().next().unwrap().into_element().unwrap();
    assert!(
        table
            .input()
            .admitted_for(&foreign, foreign_element.attributes().next().unwrap())
            .is_none()
    );
    assert_eq!(table.into_input().aliases().as_ptr(), aliases);
}

mod borrowed;
mod budget;
mod declaration;
mod refusal;
