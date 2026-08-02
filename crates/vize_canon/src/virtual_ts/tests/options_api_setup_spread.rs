use super::generate_virtual_ts_with_offsets_options_api;

#[test]
fn options_api_setup_return_spread_exposes_template_references() {
    let script = r#"import { defineComponent, toRefs } from '@nuxtjs/composition-api'

function useAiSupportForm() {
    return {
        formInput: {
            aiSupportTitle: '',
            aiSupportType: '',
            aiSupportTagName: '',
        },
    }
}

export default defineComponent({
    setup() {
        const { formInput } = useAiSupportForm()
        return {
            ...toRefs(formInput),
        }
    },
})
"#;
    let allocator = vize_carton::Bump::new();
    let (root, _) = vize_armature::parse(
        &allocator,
        "<div>{{ aiSupportTitle }} {{ aiSupportType }} {{ aiSupportTagName }}</div>",
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

    for name in ["aiSupportTitle", "aiSupportType", "aiSupportTagName"] {
        assert!(
            output.code.contains(&format!(
                "type __R_{name} = __VizeOptionsSetupBinding<\"{name}\">;"
            )),
            "setup spread template reference should be captured from the default instance:\n{}",
            output.code
        );
        assert!(
            output
                .code
                .contains(&format!("var {name}: __U<__R_{name}> = undefined as any;")),
            "setup spread template reference should be declared in template scope:\n{}",
            output.code
        );
    }
}

#[test]
fn options_api_setup_return_spread_deduplicates_instance_globals() {
    let script = r#"import { defineComponent, useFetch } from '@nuxtjs/composition-api'

export default defineComponent({
    setup() {
        const { $fetch } = useFetch(async () => {})
        const refreshFromSetup = () => $fetch()
        const extra = {}

        return { refreshFromSetup, ...extra }
    },
})
"#;
    let allocator = vize_carton::Bump::new();
    let (root, _) = vize_armature::parse(&allocator, r#"<button @click="$fetch">Refresh</button>"#);
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
        output
            .code
            .contains(r#"type __R_$fetch = __VizeOptionsSetupBinding<"$fetch">;"#),
        "spread fallback should preserve the setup-return binding type:\n{}",
        output.code
    );
    assert_eq!(
        output.code.matches("var $fetch:").count(),
        1,
        "the spread setup binding should be declared exactly once:\n{}",
        output.code
    );
    assert!(
        !output
            .code
            .contains("const $fetch: __VizeInstanceGlobal<'$fetch'>"),
        "an instance-global declaration must not duplicate the spread setup binding:\n{}",
        output.code
    );
}

#[test]
fn options_api_without_setup_spread_keeps_instance_global() {
    let script = r#"import { defineComponent } from '@nuxtjs/composition-api'

export default defineComponent({
    setup() {
        return { refreshFromSetup: () => {} }
    },
})
"#;
    let allocator = vize_carton::Bump::new();
    let (root, _) = vize_armature::parse(&allocator, r#"<button @click="$fetch">Refresh</button>"#);
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
        output
            .code
            .contains("const $fetch: __VizeInstanceGlobal<'$fetch'>"),
        "instance globals should remain available without a spread setup return:\n{}",
        output.code
    );
    assert!(
        !output.code.contains("type __R_$fetch"),
        "a non-spread setup return must not infer an unrelated setup binding:\n{}",
        output.code
    );
}
