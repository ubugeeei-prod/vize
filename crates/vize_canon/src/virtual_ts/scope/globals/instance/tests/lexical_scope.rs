//! Dollar spelling does not override an authored lexical template owner.

use super::*;

const TEMPLATE: &str = r#"<Panel v-slot="$form">{{ $form.label }}
<Panel><template #default="$field">{{ $form.label }}{{ $field.label }}</template></Panel>
<i v-for="$item in $form.items">{{ $item.label }}{{ $form.label }}</i>
<button @click="($value) => $value.label">{{ $form.label }}</button></Panel>
<Panel :title="$before.label" v-slot="$before">{{ $before.label }}</Panel>
<Panel v-slot="$after" :title="$after.label">{{ $after.label }}</Panel>
{{ $form.label }}{{ $field.label }}{{ $unknown.label }}
<i v-for="$self in $self.items">{{ $self.label }}</i>"#;

fn output(strict: bool) -> VirtualTsOutput {
    let allocator = vize_carton::Allocator::new();
    let (root, errors) = vize_armature::parse(&allocator, TEMPLATE);
    assert!(errors.is_empty(), "{errors:?}");
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(SCRIPT);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let options = VirtualTsOptions {
        strict_instance_globals: strict,
        ..Default::default()
    };
    if let Some(dir) = std::env::var_os("VIZE_LEXICAL_DOLLAR_CAPTURE_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(std::path::Path::new(&dir).join(vize_l0::cstr!("producer-scopes-{strict}.json").as_str()), serde_json::to_vec_pretty(&serde_json::json!({
            "script": SCRIPT, "template": TEMPLATE, "templateOffset": 1000,
            "strictInstanceGlobals": strict, "completeSummary": vize_l0::cstr!("{summary:#?}"),
            "expressions": summary.template_expressions.iter().map(|expr| serde_json::json!({
                "source":expr.content.as_str(),"start":expr.start,"end":expr.end,"scopeId":expr.scope_id.as_u32(),
            })).collect::<Vec<_>>(),
            "scopes": summary.scopes.iter().map(|scope|serde_json::json!({
                "id":scope.id.as_u32(),"parent":scope.parent().map(|parent|parent.as_u32()),
                "kind":vize_l0::cstr!("{:?}",scope.kind),"span":vize_l0::cstr!("{:?}",scope.span),
                "data":vize_l0::cstr!("{:?}",scope.data()),"bindings":scope.bindings().map(|(name,data)|serde_json::json!({"name":name,"data":vize_l0::cstr!("{data:?}")})).collect::<Vec<_>>(),
            })).collect::<Vec<_>>(),
        })).unwrap()).unwrap();
    }
    generate_virtual_ts_with_offsets(&summary, Some(SCRIPT), Some(&root), 0, 1000, &options)
}

#[test]
fn lexical_dollar_scopes_keep_only_real_instance_reads_and_exact_authored_ranges() {
    let output = output(true);
    let mut reads: Vec<_> = output
        .mapping
        .spans()
        .iter()
        .filter_map(|mapping| {
            output
                .code
                .get(..mapping.gen_range.start)
                .filter(|head| head.ends_with("__vize_strict_template_context."))?;
            Some((
                output.code.get(mapping.gen_range.clone()).unwrap(),
                mapping.src_range.clone(),
            ))
        })
        .collect();
    reads.sort_by_key(|(_, range)| range.start);
    assert_eq!(
        reads,
        vec![
            ("$before", 1293..1300),
            ("$after", 1384..1390),
            ("$form", 1428..1433),
            ("$field", 1445..1451),
            ("$unknown", 1463..1471),
            ("$self", 1500..1505),
        ],
        "{}",
        output.code
    );
}

#[test]
fn permissive_dollar_scopes_do_not_hoist_lexical_slot_loop_or_handler_names() {
    let output = output(false);
    let declarations: Vec<_> = output
        .code
        .lines()
        .filter(|line| {
            line.trim_start().starts_with("const $") && line.ends_with("= undefined as any;")
        })
        .collect();
    assert_eq!(
        declarations,
        vec![
            "  const $before: __VizeInstanceGlobal<'$before'> = undefined as any;",
            "  const $after: __VizeInstanceGlobal<'$after'> = undefined as any;",
            "  const $form: __VizeInstanceGlobal<'$form'> = undefined as any;",
            "  const $field: __VizeInstanceGlobal<'$field'> = undefined as any;",
            "  const $unknown: __VizeInstanceGlobal<'$unknown'> = undefined as any;",
            "  const $self: __VizeInstanceGlobal<'$self'> = undefined as any;",
        ]
    );
}
