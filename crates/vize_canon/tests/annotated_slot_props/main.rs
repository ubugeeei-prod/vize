#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
//! An annotated `v-slot` parameter is checked against the slot the child
//! declares, as vue-tsc's `__VLS_vSlot(slot, (_: Annotation) => …)` does: the
//! annotated callback must be assignable to the declared slot member.
use std::path::Path;

use davinci_test_support::script_block_project as project;

const PARENT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/annotated-slot-props/Parent.vue.txt"
));
const OBJECT_SLOT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/annotated-slot-props/ObjectSlot.vue.txt"
));
const PROPERTY_SLOT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/annotated-slot-props/PropertySlot.vue.txt"
));
const METHOD_SLOT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/annotated-slot-props/MethodSlot.vue.txt"
));
const DYNAMIC_SLOT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/annotated-slot-props/DynamicSlot.vue.txt"
));
const DYNAMIC_PARENT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/annotated-slot-props/DynamicParent.vue.txt"
));

fn check(child: &str, parent: &str, slot: &str) -> Vec<String> {
    let parent = parent.replace("SLOT_EXPRESSION", slot);
    project::check(&[("src/Child.vue", child), ("src/Parent.vue", &parent)])
}

/// n8n's `ModalRoot.vue` declares its slot as a props object, not a function.
#[test]
fn an_annotated_slot_on_an_object_declared_slot_is_rejected() {
    let child = OBJECT_SLOT;
    assert_eq!(
        check(child, PARENT, "{ data }: { data: { mode: 'new' } }"),
        [
            "src/Parent.vue(6,25): error TS2345: Argument of type '(_: { data: { mode: 'new'; }; }) => any' is not assignable to parameter of type '{ name: string; data: Record<string, unknown>; }'."
        ]
    );
    // Without an annotation there is nothing to check.
    assert_eq!(check(child, PARENT, "{ data }"), [] as [String; 0]);
}

/// A property-declared slot function compares its parameter strictly.
#[test]
fn an_annotation_must_accept_a_property_declared_slot_payload() {
    let child = PROPERTY_SLOT;
    assert_eq!(
        check(child, PARENT, "{ n }: { n: string }"),
        [
            "src/Parent.vue(6,25): error TS2345: Argument of type '(_: { n: string; }) => any' is not assignable to parameter of type '(props: { n: number; }) => any'.\nTypes of parameters '_' and 'props' are incompatible.\nType '{ n: number; }' is not assignable to type '{ n: string; }'.\nTypes of property 'n' are incompatible.\nType 'number' is not assignable to type 'string'."
        ]
    );
    assert_eq!(
        check(child, PARENT, "{ n }: { n: number }"),
        [] as [String; 0]
    );
}

/// A method-declared slot compares its parameter bivariantly, so a narrower
/// annotation passes and only an unrelated one fails.
#[test]
fn a_method_declared_slot_accepts_a_narrower_annotation() {
    let child = METHOD_SLOT;
    assert_eq!(
        check(child, PARENT, "{ data }: { data: { mode: 'new' } }"),
        [] as [String; 0]
    );
    assert_eq!(
        check(child, PARENT, "{ data }: { data: number }"),
        [
            "src/Parent.vue(6,25): error TS2345: Argument of type '(_: { data: number; }) => any' is not assignable to parameter of type '(props: { data: Record<string, unknown>; }) => any'.\nTypes of parameters '_' and 'props' are incompatible.\nType '{ data: Record<string, unknown>; }' is not assignable to type '{ data: number; }'.\nTypes of property 'data' are incompatible.\nType 'Record<string, unknown>' is not assignable to type 'number'."
        ]
    );
}

/// An unresolved child has `any` slots, so only the import is reported.
#[test]
fn an_unresolved_child_accepts_any_annotation() {
    let parent = PARENT
        .replace("'./Child.vue'", "'missing-lib'")
        .replace("SLOT_EXPRESSION", "{ n }: { n: string }");
    assert_eq!(
        project::check(&[("src/Parent.vue", &parent)]),
        [
            "src/Parent.vue(2,19): error TS2307: Cannot find module 'missing-lib' or its corresponding type declarations."
        ]
    );
}

/// A dynamic slot name selects the declared member it evaluates to.
#[test]
fn a_dynamic_slot_name_checks_the_member_it_selects() {
    let child = DYNAMIC_SLOT;
    let parent = DYNAMIC_PARENT;
    assert_eq!(
        check(child, parent, "{ s }: { s: string }"),
        [] as [String; 0]
    );
    assert_eq!(
        check(child, parent, "{ s }: { s: number }"),
        [
            "src/Parent.vue(7,24): error TS2345: Argument of type '(_: { s: number; }) => any' is not assignable to parameter of type '(props: { s: string; }) => any'.\nTypes of parameters '_' and 'props' are incompatible.\nType '{ s: string; }' is not assignable to type '{ s: number; }'.\nTypes of property 's' are incompatible.\nType 'string' is not assignable to type 'number'."
        ]
    );
}

/// Through the content mapper the callback keeps a span from its `(` to the
/// slot expression, so the host reports it instead of ignoring the range.
#[test]
fn the_content_mapper_maps_the_checked_callback_to_the_slot_expression() {
    let source = PARENT.replace("SLOT_EXPRESSION", "{ data }: { data: { mode: 'new' } }");
    let transform =
        vize_canon::generate_vue_content_mapper_transform(Path::new("Parent.vue"), &source)
            .expect("complete mapper projection");
    let callback = transform
        .text
        .find("(_: { data: { mode: 'new' } }) => [] as any")
        .expect("the annotated callback is emitted");
    let expression = source
        .find("{ data }: {")
        .expect("the slot expression is authored");
    let mapped: Vec<[usize; 2]> = transform
        .mappings
        .iter()
        .filter(|span| span.0[0] <= callback && callback < span.0[0] + span.0[1])
        .map(|span| [span.0[2], span.0[3]])
        .collect();
    assert_eq!(mapped, [[expression, 1]]);
    let ignored = transform
        .diagnostic_directives
        .iter()
        .flat_map(|table| &table.directives)
        .filter(|directive| directive.0[2] <= callback && callback < directive.0[3])
        .count();
    assert_eq!(ignored, 0);
}
