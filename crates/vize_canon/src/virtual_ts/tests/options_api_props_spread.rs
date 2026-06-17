use super::{analyze_options_api_script, generate_virtual_ts_with_offsets_options_api};

#[test]
fn test_options_api_props_object_with_spread_defers_to_setup_scope() {
    let script = r#"import { defineComponent } from 'vue'

const sharedProps = {
    meta: { type: Object, required: true as const },
}
function useFakeStore() {
    return { cached: (s: string, _b: boolean) => s }
}
export default defineComponent({
    props: { ...sharedProps },
    setup() {
        const store = useFakeStore()
        return { store }
    },
    data() {
        return { missing: false }
    },
    computed: {
        url() {
            return this.store.cached('x', false)
        },
    },
    methods: {
        onError(_a: unknown) {
            this.missing = true
        },
    },
})
"#;
    let summary = analyze_options_api_script(script);
    let output = generate_virtual_ts_with_offsets_options_api(
        &summary,
        Some(script),
        None,
        0,
        0,
        &Default::default(),
    );

    assert!(
        !output
            .code
            .contains("export type Props = __RuntimePropShape<{ ...sharedProps }>;"),
        "object spread must not be emitted as invalid type-literal syntax:\n{}",
        output.code
    );
    assert!(
        output
            .code
            .contains("const __vize_options_props = ({ ...sharedProps });"),
        "spread props must be kept in setup scope where sharedProps resolves:\n{}",
        output.code
    );
    assert!(
        output.code.contains(
            "export type Props = __RuntimePropShape<Awaited<ReturnType<typeof __setup>>[\"__vize_options_props\"]>;"
        ),
        "Props should be derived from the setup-scoped runtime props artifact:\n{}",
        output.code
    );
    assert!(
        output.code.contains("    store: any;"),
        "Options API `this` bridge should include setup() return bindings:\n{}",
        output.code
    );
}
