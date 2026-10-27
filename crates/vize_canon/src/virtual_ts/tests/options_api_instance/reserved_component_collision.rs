use super::*;

#[test]
fn reserved_component_local_does_not_collide_with_authored_or_configured_names() {
    let script = r#"export default {
  props: { as: String, _as: String, _as_: String },
}"#;
    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, r#"<component :is="as" />"#);
    let mut analyzer = vize_croquis::Analyzer::with_options(vize_croquis::AnalyzerOptions::full())
        .with_options_api();
    analyzer.analyze_script_plain(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    for (globals, expected_local) in [
        (Vec::new(), "_as__"),
        (
            vec![TemplateGlobal {
                name: "_as__".into(),
                type_annotation: "string".into(),
                default_value: "\"\"".into(),
            }],
            "_as___",
        ),
    ] {
        let options = VirtualTsOptions {
            template_globals: globals,
            ..Default::default()
        };
        let output = generate_virtual_ts_with_offsets_options_api(
            &summary,
            Some(script),
            Some(&root),
            0,
            0,
            &options,
        );
        let actual = output
            .code
            .lines()
            .filter(|line| {
                (line.starts_with("  const _as") && line.contains("__VizeOptionsBinding"))
                    || line.starts_with("  type __as_Component_0")
            })
            .collect::<Vec<_>>();
        let local = vize_carton::cstr!(
            "  const {expected_local}: __VizeOptionsBinding<typeof __default__, \"as\"> = (void 0) as any; void {expected_local};"
        );
        let reference = vize_carton::cstr!("  type __as_Component_0 = typeof {expected_local};");
        assert_eq!(
            actual,
            vec![
                "  const _as: __VizeOptionsBinding<typeof __default__, \"_as\"> = (void 0) as any;",
                "  const _as_: __VizeOptionsBinding<typeof __default__, \"_as_\"> = (void 0) as any;",
                local.as_str(),
                reference.as_str(),
            ],
            "{}",
            output.code
        );
    }
}
