use crate::virtual_ts::{VirtualTsOptions, generate_virtual_ts_with_offsets_options_api};

#[test]
fn data_template_bindings_emit_var_while_static_options_stay_const() {
    let script = r#"export default {
    props: { initial: Number },
    data() {
        return { open: false }
    },
    computed: {
        label() {
            return String(this.initial)
        },
    },
    methods: {
        close() {
            this.open = false
        },
    },
}
"#;
    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(
        &allocator,
        r#"<button @click="open = true">{{ initial }} {{ label }}</button>"#,
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
        &VirtualTsOptions::default(),
    );

    let declarations = output
        .code
        .lines()
        .filter(|line| line.contains("__VizeOptionsBinding"))
        .filter(|line| {
            let trimmed = line.trim_start();
            trimmed.starts_with("const ") || trimmed.starts_with("var ")
        })
        .collect::<Vec<_>>();
    assert_eq!(
        declarations,
        [
            "  const close: __VizeOptionsBinding<typeof __default__, \"close\"> = undefined as any;",
            "  const initial: __VizeOptionsBinding<typeof __default__, \"initial\"> = undefined as any;",
            "  const label: __VizeOptionsBinding<typeof __default__, \"label\"> = undefined as any;",
            "  var open: __VizeOptionsBinding<typeof __default__, \"open\"> = undefined as any;",
        ]
    );
}
