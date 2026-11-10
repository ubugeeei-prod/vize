#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
use crate::virtual_ts::{
    VirtualTsGenerationOptions, VirtualTsOptions, generate_virtual_ts_with_offsets_and_checks,
    helpers::SHARED_PREAMBLE_DTS,
};
use vize_croquis::{Analyzer, AnalyzerOptions};

fn generate_template_code(
    script: &str,
    template: &str,
    hoist_shared_preamble: bool,
) -> std::string::String {
    generate_template_code_with(script, template, hoist_shared_preamble, false)
}

fn generate_template_code_with(
    script: &str,
    template: &str,
    hoist_shared_preamble: bool,
    local_template_ref_unwrap: bool,
) -> std::string::String {
    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);

    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    generate_virtual_ts_with_offsets_and_checks(
        &summary,
        Some(script),
        Some(&root),
        0,
        0,
        &VirtualTsOptions::default(),
        VirtualTsGenerationOptions {
            hoist_shared_preamble,
            local_template_ref_unwrap,
            ..Default::default()
        },
    )
    .code
    .into()
}

fn exact_line_count(source: &str, expected: &str) -> usize {
    source.lines().filter(|line| *line == expected).count()
}

#[test]
fn explicit_ref_literal_types_survive_template_unwrap() {
    let script = r#"import { ref } from 'vue'
const direction = ref<'rtl'>('rtl')
"#;
    let template = r#"<Child :direction="direction" />"#;
    let hoisted_code = generate_template_code(script, template, true);
    let embedded_code = generate_template_code(script, template, false);
    let editor_code = generate_template_code_with(script, template, true, true);

    const WIDEN_HELPER: &str = "type $WidenTemplateRef<T> = $IsAny<T> extends true ? T : $IsUnion<T> extends true ? T : T extends string ? string extends T ? string : T : T extends number ? number extends T ? number : T : T extends boolean ? boolean extends T ? boolean : T : T;";
    assert_eq!(
        exact_line_count(SHARED_PREAMBLE_DTS, &format!("export {WIDEN_HELPER}")),
        1,
        "the shared template-ref helper must preserve the exact widening oracle:\n{}",
        SHARED_PREAMBLE_DTS,
    );
    assert_eq!(
        exact_line_count(&embedded_code, &format!("    {WIDEN_HELPER}")),
        1,
        "the embedded template-ref helper must preserve the exact widening oracle:\n{embedded_code}",
    );
    // Hoisted shadows name the shared alias; embedded and editor ones keep the
    // local one, so an editor answer never depends on the shared helpers.
    assert_eq!(
        exact_line_count(
            &editor_code,
            "    type __U<T> = T extends { value: unknown } ? T extends import('vue').Ref ? import(\"vize:virtual-ts\").$WidenTemplateRef<T['value']> : T : T;",
        ),
        1,
        "editor modules keep the per-file unwrap alias:\n{editor_code}",
    );
    for (code, shadow) in [
        (
            editor_code.clone(),
            "    var direction: __U<__R_direction> = (void 0) as any;",
        ),
        (
            hoisted_code,
            "    var direction: import(\"vize:virtual-ts\").$TemplateRefUnwrap<__R_direction> = (void 0) as any;",
        ),
        (
            embedded_code,
            "    var direction: __U<__R_direction> = (void 0) as any;",
        ),
    ] {
        assert_eq!(
            exact_line_count(&code, "  type __R_direction = typeof direction;"),
            1,
            "template expressions should capture the exact setup ref type:\n{code}",
        );
        assert_eq!(
            exact_line_count(&code, shadow),
            1,
            "template expressions should emit the exact unwrapped ref shadow:\n{code}",
        );
    }
}

#[test]
fn authored_props_binding_does_not_collide_with_template_props_object() {
    let script = r#"defineProps<{ label?: string }>()
const { props, size } = useField()
"#;
    let code = generate_template_code(
        script,
        r#"<div>{{ props.title }} {{ size }} {{ label }}</div>"#,
        true,
    );

    assert_eq!(
        exact_line_count(&code, "  type __R_props = typeof props;"),
        1,
        "the authored maybe-ref binding must retain its exact template capture:\n{code}",
    );
    assert_eq!(
        exact_line_count(
            &code,
            "    var props: import(\"vize:virtual-ts\").$TemplateRefUnwrapNarrowed<__R_props> = (void 0) as any;",
        ),
        1,
        "the authored binding must remain the exact template-visible `props`:\n{code}",
    );
    assert_eq!(
        exact_line_count(
            &code,
            "  const __vize_props: __DefineProps<Props> = {} as __DefineProps<Props>;",
        ),
        1,
        "declared props must use the exact collision-free internal object:\n{code}",
    );
    assert_eq!(
        exact_line_count(&code, r#"  const label = __vize_props["label"];"#),
        1,
        "declared prop locals must resolve through the internal object:\n{code}",
    );
    assert_eq!(
        exact_line_count(
            &code,
            "  const props: __DefineProps<Props> = {} as __DefineProps<Props>;",
        ),
        0,
        "the generated object must not redeclare the authored binding:\n{code}",
    );
}

/// The template-ref registry targets: hoisted modules name the shared SDK
/// declarations, whose bodies are byte-for-byte the per-file ones that the
/// embedded and editor modules keep.
#[test]
fn template_ref_registry_targets_name_the_shared_declarations() {
    let script = r#"import { useTemplateRef } from 'vue'
import Child from './Child.vue'
const box = useTemplateRef('box')
const child = useTemplateRef('child')
const probe = useTemplateRef('probe')
"#;
    let template = r#"<div ref="box" /><Child ref="child" /><Child ref="probe" :label="'x'" />"#;
    let hoisted = generate_template_code(script, template, true);
    let embedded = generate_template_code(script, template, false);
    let editor = generate_template_code_with(script, template, true, true);
    let names = [
        "$DomElement",
        "$TemplateComponentRef",
        "$RefInstanceFactory",
        "$TemplateRefInstance",
    ];
    for name in names {
        let declaration = |code: &str| {
            let prefix = format!("  type {name}<");
            let lines: Vec<_> = code
                .lines()
                .filter(|line| line.starts_with(&prefix))
                .collect();
            assert_eq!(lines.len(), 1, "{name} per-file declaration:\n{code}");
            lines[0].trim_start().to_owned()
        };
        let local = declaration(&embedded);
        assert_eq!(declaration(&editor), local, "editor modules keep {name}");
        assert_eq!(
            exact_line_count(SHARED_PREAMBLE_DTS, &format!("export {local}")),
            1,
            "the shared {name} must be the exact per-file declaration:\n{SHARED_PREAMBLE_DTS}",
        );
        let redeclared = hoisted
            .lines()
            .filter(|line| line.trim_start().starts_with(&format!("type {name}<")))
            .count();
        assert_eq!(
            redeclared, 0,
            "hoisted modules must not redeclare {name}:\n{hoisted}"
        );
    }
    for (code, prefix) in [(&hoisted, "import(\"vize:virtual-ts\")."), (&embedded, "")] {
        assert_eq!(
            exact_line_count(
                code,
                &format!(
                    "  type $TemplateRefs = {{ \"box\": {prefix}$DomElement<\"div\">; \"child\": {prefix}$TemplateComponentRef<typeof Child>; \"probe\": {prefix}$TemplateRefInstance<typeof __vize_template.__vizeRefs, \"38\", typeof Child>; }};"
                ),
            ),
            1,
            "registry targets:\n{code}",
        );
        assert_eq!(
            exact_line_count(
                code,
                &format!(
                    "  var __vize_ref_38 = (undefined as unknown as {prefix}$RefInstanceFactory<typeof Child>)(Child)({{"
                ),
            ),
            1,
            "ref instance probe:\n{code}",
        );
    }
}
