use super::*;

#[test]
fn missing_collection_binding_does_not_resolve_to_its_own_same_named_alias() {
    let arena = Allocator::default();
    let file = "<template><b v-for='items in items'/></template>";
    let owner = selected(&arena, file);
    let lookup = Lookup::new(&[]);
    let original = input(&owner);
    let aliases = original.aliases().as_ptr();
    let collection = original.collection() as *const _;
    let rejected = resolve_for_head(original, &lookup).unwrap_err();
    assert_eq!(
        rejected.error,
        ForResolutionError {
            part: ForHeadPart::Collection,
            span: Span::new(0, 5),
            kind: ForResolutionErrorKind::Reference(ResolutionErrorKind::MissingBinding)
        }
    );
    assert_eq!(lookup.visits.get(), 1);
    assert_eq!(rejected.input().aliases().as_ptr(), aliases);
    assert_eq!(rejected.input().collection() as *const _, collection);
    assert_eq!(
        rejected.into_input().operand().raw_value(),
        "items in items"
    );
}

#[test]
fn duplicate_unicode_alias_refuses_at_second_original_alias_without_partial_table() {
    let arena = Allocator::default();
    let file = "<template><b v-for='(α, α) in items'/></template>";
    let owner = selected(&arena, file);
    let lookup = Lookup::new(&[("items", 2)]);
    let original = input(&owner);
    let original_span = original.operand().value_span();
    let rejected = resolve_for_head(original, &lookup).unwrap_err();
    assert_eq!(
        rejected.error,
        ForResolutionError {
            part: ForHeadPart::Aliases,
            span: Span::new(4, 6),
            kind: ForResolutionErrorKind::DuplicateAlias
        }
    );
    assert_eq!(lookup.visits.get(), 1);
    assert_eq!(rejected.input().operand().value_span(), original_span);
    assert_eq!(
        rejected.into_input().operand().raw_value(),
        "(α, α) in items"
    );
}

#[test]
fn generated_helper_and_context_alias_prefixes_remain_typed_bounded_refusals() {
    let arena = Allocator::default();
    for file in [
        "<template><b v-for='_renderList in items'/></template>",
        "<template><b v-for='$setup in items'/></template>",
        "<template><b v-for='(item, _key) in items'/></template>",
    ] {
        let owner = selected(&arena, file);
        let original = input(&owner);
        let raw = original.operand().raw_value();
        let rejected = resolve_for_head(original, &Lookup::new(&[("items", 2)])).unwrap_err();
        assert_eq!(rejected.error.part, ForHeadPart::Aliases);
        assert_eq!(rejected.error.kind, ForResolutionErrorKind::ReservedAlias);
        assert_eq!(rejected.input().operand().raw_value(), raw);
    }
}

#[test]
fn missing_collection_precedes_alias_policy_and_returns_the_entire_original_input() {
    let arena = Allocator::default();
    let owner = selected(
        &arena,
        "<template><b v-for='(_value, _value) in absent'/></template>",
    );
    let rejected = resolve_for_head(input(&owner), &Lookup::new(&[])).unwrap_err();
    assert_eq!(rejected.error.part, ForHeadPart::Collection);
    assert_eq!(
        rejected.error.kind,
        ForResolutionErrorKind::Reference(ResolutionErrorKind::MissingBinding)
    );
    assert_eq!(
        rejected.input().operand().raw_value(),
        "(_value, _value) in absent"
    );
}
