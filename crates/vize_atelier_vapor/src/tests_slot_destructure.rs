//! Destructuring in `v-slot` and `v-for` reads each name from its own path in
//! the slot props or loop item, as Vue's compiler does: renames, nesting,
//! defaults, computed keys, rest elements and annotated patterns or aliases.

use super::{VaporCompilerOptions, compile_vapor};
use vize_carton::{Allocator, String};

fn compile(source: &str, davinci_retained_lane: bool) -> String {
    let allocator = Allocator::new();
    let result = compile_vapor(
        &allocator,
        source,
        VaporCompilerOptions {
            prefix_identifiers: true,
            davinci_retained_lane,
            ..Default::default()
        },
    );
    assert!(
        result.error_messages.is_empty(),
        "{:?}",
        result.error_messages
    );
    result.code
}

#[test]
fn a_renamed_slot_prop_reads_its_original_key() {
    let source =
        r#"<Child v-slot="{ props: trigger }"><button v-bind="trigger">x</button></Child>"#;
    for retained in [false, true] {
        let code = compile(source, retained);
        assert!(code.contains("_slotProps0.props"), "{code}");
        assert!(!code.contains("_slotProps0.trigger"), "{code}");
    }
}

#[test]
fn a_nested_slot_prop_reads_its_full_path() {
    let source = r#"<Child v-slot="{ item: { label }, index }"><span>{{ label }} {{ index }}</span></Child>"#;
    for retained in [false, true] {
        let code = compile(source, retained);
        assert!(code.contains("_slotProps0.item.label"), "{code}");
        assert!(code.contains("_slotProps0.index"), "{code}");
    }
}

#[test]
fn a_renamed_slot_prop_with_a_default_reads_its_original_key() {
    let source = r#"<Child v-slot="{ props: trigger = fallback }"><button v-bind="trigger">x</button></Child>"#;
    for retained in [false, true] {
        let code = compile(source, retained);
        assert!(
            code.contains("_getDefaultValue(_slotProps0.props, () => (_ctx.fallback))"),
            "{code}"
        );
        assert!(!code.contains("_slotProps0.trigger"), "{code}");
    }
}

#[test]
fn an_object_rest_slot_prop_excludes_its_sibling_keys() {
    let source = r#"<Child v-slot="{ a, 'kebab-key': k, [key]: v, ...rest }">{{ a }}{{ k }}{{ v }}{{ rest }}</Child>"#;
    for retained in [false, true] {
        let code = compile(source, retained);
        assert!(
            code.contains(r#"_getRestElement(_slotProps0, ["a", "kebab-key", _ctx.key])"#),
            "{code}"
        );
        assert!(code.contains("getRestElement as _getRestElement"), "{code}");
        assert!(code.contains("_slotProps0[_ctx.key]"), "{code}");
        assert!(!code.contains("_ctx.rest"), "{code}");
        assert!(!code.contains("_ctx.v)"), "{code}");
    }
}

#[test]
fn a_nested_rest_slot_prop_reads_from_its_own_parent() {
    let source = r#"<Child v-slot="{ item: { id, ...others } }">{{ id }}{{ others }}</Child>"#;
    for retained in [false, true] {
        let code = compile(source, retained);
        assert!(
            code.contains(r#"_getRestElement(_slotProps0.item, ["id"])"#),
            "{code}"
        );
        assert!(code.contains("_slotProps0.item.id"), "{code}");
    }
}

#[test]
fn an_array_rest_slot_prop_slices_after_the_named_positions() {
    let source = r#"<Child v-slot="{ list: [first, , third, ...tail] }">{{ first }}{{ third }}{{ tail }}</Child>"#;
    for retained in [false, true] {
        let code = compile(source, retained);
        assert!(code.contains("_slotProps0.list[0]"), "{code}");
        assert!(code.contains("_slotProps0.list[2]"), "{code}");
        assert!(code.contains("_slotProps0.list.slice(3)"), "{code}");
        assert!(!code.contains("_ctx.tail"), "{code}");
    }
}

#[test]
fn a_type_annotated_slot_pattern_still_binds_its_names() {
    for source in [
        r#"<Child v-slot="{ a }: any">{{ a }}</Child>"#,
        r#"<Child v-slot="{ a }: { a: string }">{{ a }}</Child>"#,
        r#"<Child v-slot="props: { a: string }">{{ props.a }}</Child>"#,
    ] {
        for retained in [false, true] {
            let code = compile(source, retained);
            assert!(code.contains("_toDisplayString(_slotProps0.a)"), "{code}");
            assert!(!code.contains("_ctx.a"), "{code}");
            assert!(!code.contains("_ctx.props"), "{code}");
        }
    }
}

#[test]
fn a_loop_value_with_a_default_rest_and_computed_key_reads_the_item() {
    let source = r#"<li v-for="{ [key]: v, id: itemId = 0, ...others } in rows" :key="itemId">{{ v }}{{ itemId }}{{ others }}</li>"#;
    for retained in [false, true] {
        let code = compile(source, retained);
        assert!(code.contains("_for_item0.value[_ctx.key]"), "{code}");
        assert!(
            code.contains("_getDefaultValue(_for_item0.value.id, () => (0))"),
            "{code}"
        );
        assert!(
            code.contains(r#"_getRestElement(_for_item0.value, [_ctx.key, "id"])"#),
            "{code}"
        );
        // The key callback destructures the raw item: its computed key reads
        // the component, and the key itself is the pattern's own local.
        assert!(
            code.contains("({ [_ctx.key]: v, id: itemId = 0, ...others }) => (itemId)"),
            "{code}"
        );
    }
}

#[test]
fn an_array_loop_value_with_a_rest_element_slices_the_item() {
    let source = r#"<li v-for="([first, ...tail], index) in rows" :key="first">{{ first }}{{ tail }}{{ index }}</li>"#;
    for retained in [false, true] {
        let code = compile(source, retained);
        assert!(code.contains("_for_item0.value[0]"), "{code}");
        assert!(code.contains("_for_item0.value.slice(1)"), "{code}");
        assert!(
            code.contains("([first, ...tail], index) => (first)"),
            "{code}"
        );
        assert!(!code.contains("_ctx.tail"), "{code}");
    }
}

#[test]
fn annotated_loop_aliases_bind_their_names() {
    let source = r#"<span v-for="(item: string, index?: number) in data" :key="item" @click="go(item, index)">{{ item }}{{ index }}</span>"#;
    for retained in [false, true] {
        let code = compile(source, retained);
        assert!(
            code.contains("_toDisplayString(_for_item0.value) + _toDisplayString(_for_key0.value)"),
            "{code}"
        );
        assert!(
            code.contains("_ctx.go(_for_item0.value, _for_key0.value)"),
            "{code}"
        );
        // The key callback is plain JavaScript, like upstream's.
        assert!(code.contains("(item, index) => (item)"), "{code}");
        assert!(!code.contains("_ctx.item"), "{code}");
    }
}
