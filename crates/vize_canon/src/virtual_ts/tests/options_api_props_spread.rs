use super::{analyze_options_api_script, generate_virtual_ts_with_offsets_options_api};

#[test]
fn test_options_api_setup_return_bindings_use_default_instance_type() {
    let script = r#"import { defineComponent, ref } from '@nuxtjs/composition-api'

export default defineComponent({
    setup() {
        const count = ref(0)
        const store = { ready: true }
        return { count, store }
    },
})
"#;
    let allocator = vize_carton::Bump::new();
    let (root, _) = vize_armature::parse(
        &allocator,
        "<div>{{ count.toFixed(0) }} {{ store.ready }}</div>",
    );
    let mut analyzer = vize_croquis::Analyzer::with_options(vize_croquis::AnalyzerOptions::full())
        .with_options_api();
    analyzer.analyze_script_plain(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let output = generate_virtual_ts_with_offsets_options_api(
        &summary,
        Some(script),
        Some(&root),
        0,
        0,
        &Default::default(),
    );

    assert!(
        !output.code.contains("type __R_count = typeof count;"),
        "Options API setup return must not reference setup() locals outside their scope:\n{}",
        output.code
    );
    assert!(
        output
            .code
            .contains("type __R_count = __VizeOptionsSetupBinding<\"count\">;"),
        "setup return binding should be captured from the default component instance:\n{}",
        output.code
    );
    assert!(
        output
            .code
            .contains("type __R_store = __VizeOptionsSetupBinding<\"store\">;"),
        "plain setup return binding should also be exposed through the instance:\n{}",
        output.code
    );
    assert!(
        output
            .code
            .contains("var count: __U<__R_count> = undefined as any;"),
        "setup ref return should be available as an auto-unwrapped template variable:\n{}",
        output.code
    );
}

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

#[test]
fn test_options_api_props_with_type_cast_defers_to_setup_scope() {
    let script = r#"import { defineComponent } from 'vue'
import type { PropType } from 'vue'

type BreadcrumbsItem = { label: string }
type SelectedExamSubject = { id: string } | null

export default defineComponent({
    props: {
        breadcrumbs: {
            type: Array as PropType<BreadcrumbsItem[]>,
            required: true,
        },
        selectedExamSubject: {
            type: Object as PropType<SelectedExamSubject>,
            required: true,
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
            .contains("export type Props = __RuntimePropShape<{"),
        "type-cast props must not be emitted directly in type position:\n{}",
        output.code
    );
    assert!(
        output
            .code
            .contains("const __vize_options_props = ({\n        breadcrumbs:"),
        "type-cast props should stay in setup scope as a value:\n{}",
        output.code
    );
    assert!(
        output.code.contains(
            "export type Props = __RuntimePropShape<Awaited<ReturnType<typeof __setup>>[\"__vize_options_props\"]>;"
        ),
        "Props should be derived from the setup-scoped runtime props artifact:\n{}",
        output.code
    );
}
