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
    assert_instance_reads(true, "__vize_strict_template_context.");
    assert_instance_reads(false, "__ctx.");
}

fn assert_instance_reads(strict: bool, receiver: &str) {
    let output = output(strict);
    let mut reads: Vec<_> = output
        .mapping
        .spans()
        .iter()
        .filter_map(|mapping| {
            output
                .code
                .get(..mapping.gen_range.start)
                .filter(|head| head.ends_with(receiver))?;
            Some((
                output.code.get(mapping.gen_range.clone()).unwrap(),
                mapping.src_range.clone(),
            ))
        })
        .collect();
    reads.sort_by_key(|(_, range)| range.start);
    let mut expected = vec![
        ("$before", 1293..1300),
        ("$after", 1384..1390),
        ("$form", 1428..1433),
        ("$field", 1445..1451),
        ("$unknown", 1463..1471),
        ("$self", 1500..1505),
    ];
    if !strict {
        // The bare expression, canonical prop owner and second loop body are
        // genuine generated emissions. Relative-cache reuse never reuses their
        // generated coordinates or drops the corresponding authored mappings.
        expected.extend([
            ("$before", 1293..1300),
            ("$after", 1384..1390),
            ("$self", 1500..1505),
        ]);
        expected.sort_by_key(|(_, range)| range.start);
    }
    let generated: Vec<_> = output
        .mapping
        .spans()
        .iter()
        .filter(|mapping| {
            output
                .code
                .get(..mapping.gen_range.start)
                .is_some_and(|head| head.ends_with(receiver))
        })
        .map(|mapping| mapping.gen_range.clone())
        .collect();
    let mut unique_generated = generated.clone();
    unique_generated.sort_by_key(|range| (range.start, range.end));
    unique_generated.dedup();
    assert_eq!(
        generated.len(),
        unique_generated.len(),
        "one mapping per actual emitted property read"
    );
    assert_eq!(reads, expected, "{}", output.code);
}

#[test]
fn default_dollar_scopes_do_not_hoist_lexical_slot_loop_or_handler_names() {
    let output = output(false);
    let declarations: Vec<_> = output
        .code
        .lines()
        .filter(|line| line.trim_start().starts_with("const $") && line.contains(" = __ctx.$"))
        .collect();
    assert_eq!(
        declarations,
        vec![
            "    const $attrs = __ctx.$attrs;",
            "    const $slots = __ctx.$slots;",
            "    const $refs = __ctx.$refs;",
            "    const $emit = __ctx.$emit;",
        ]
    );
}
