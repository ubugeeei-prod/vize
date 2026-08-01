//! The Options API typed-instance bridge is emitted for Vue 2 only.

use super::super::{
    generate_virtual_ts_with_offsets_legacy_vue2, generate_virtual_ts_with_offsets_options_api,
};
use super::analyze_options_api_script;

/// The typed-instance bridge is the Vue 2 dialect's only `this` checker: its
/// `defineComponent` shim types no receiver, so the authored copy of a
/// `methods`/`computed` body checks nothing on its own.
#[test]
fn test_options_api_virtual_ts_emits_this_bridge() {
    let script = r#"import { defineComponent } from 'vue'

function useFakeStore() {
    return {
        ready: false,
        items: [] as Array<{ id: number; label: string }>,
    }
}

export default defineComponent({
    setup() {
        const store = useFakeStore()
        return { store }
    },
    data() {
        return { count: 0 }
    },
    computed: {
        status() {
            return this.store.ready
        },
    },
    methods: {
        bump(step: number) {
            this.count = this.count + step
            return this.status
        },
    },
    props: {
        initial: { type: Number, default: 0 },
    },
})
"#;
    let summary = analyze_options_api_script(script);
    let output = generate_virtual_ts_with_offsets_legacy_vue2(
        &summary,
        Some(script),
        None,
        0,
        0,
        &Default::default(),
    );

    assert!(
        output.code.contains("type __VizeThis ="),
        "expected typed Options API `this` bridge:\n{}",
        output.code
    );
    assert!(
        output.code.contains("__vize_method_bump"),
        "expected method body to be checked through a typed wrapper:\n{}",
        output.code
    );
    assert!(
        output.code.contains("__vize_computed_status"),
        "expected computed body to be checked through a typed wrapper:\n{}",
        output.code
    );

    // Vue 3 needs no bridge: the same bodies sit inside the authored
    // `defineComponent({ ... })` copy, where Vue types `this` exactly. A second
    // copy checked against the approximate `__VizeThis` shape would only add
    // findings vue-tsc does not report.
    let vue3 = generate_virtual_ts_with_offsets_options_api(
        &summary,
        Some(script),
        None,
        0,
        0,
        &Default::default(),
    );
    assert!(
        !vue3.code.contains("__VizeThis"),
        "Vue 3 `defineComponent` already types `this`; the bridge must stay off:\n{}",
        vue3.code
    );
}

/// Dropping the authored `async`/`*` modifiers would make TypeScript report
/// TS1308 on the author's `await` and TS1163 on their `yield`, anchored inside
/// the authored body the copy now maps to.
#[test]
fn test_options_api_bridge_preserves_async_and_generator_modifiers() {
    let script = r#"import { defineComponent } from 'vue'

export default defineComponent({
    data() {
        return { count: 0 }
    },
    methods: {
        async load() {
            this.count = await Promise.resolve(1)
        },
        *walk() {
            yield this.count
        },
    },
})
"#;
    let summary = analyze_options_api_script(script);
    let output = generate_virtual_ts_with_offsets_legacy_vue2(
        &summary,
        Some(script),
        None,
        0,
        0,
        &Default::default(),
    );

    assert!(
        output.code.contains("async function __vize_method_load("),
        "expected the authored `async` modifier to survive the copy:\n{}",
        output.code
    );
    assert!(
        output.code.contains("function* __vize_method_walk("),
        "expected the authored generator star to survive the copy:\n{}",
        output.code
    );
}

/// `safe_identifier` collapses every non-identifier byte to `_`, so distinct
/// authored keys can request one name. Two declarations sharing it would raise
/// TS2393 against an authored body.
#[test]
fn test_options_api_bridge_disambiguates_colliding_sanitized_names() {
    let script = r#"import { defineComponent } from 'vue'

export default defineComponent({
    data() {
        return { count: 0 }
    },
    methods: {
        'a-b'() {
            return this.count
        },
        'a.b'() {
            return this.count
        },
    },
})
"#;
    let summary = analyze_options_api_script(script);
    let output = generate_virtual_ts_with_offsets_legacy_vue2(
        &summary,
        Some(script),
        None,
        0,
        0,
        &Default::default(),
    );

    assert!(
        output.code.contains("function __vize_method_a_b(")
            && output.code.contains("function __vize_method_a_b_2("),
        "colliding sanitized keys must produce distinct bridge functions:\n{}",
        output.code
    );
}
