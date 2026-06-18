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
