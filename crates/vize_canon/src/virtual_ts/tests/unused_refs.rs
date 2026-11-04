use crate::virtual_ts::{
    VirtualTsGenerationOptions, VirtualTsOptions, generate_virtual_ts_with_offsets_and_checks,
};
use vize_croquis::{Analyzer, AnalyzerOptions};

mod mixed_scripts;

#[test]
fn test_preserve_unused_diagnostics_does_not_mark_static_template_refs_used() {
    let script = r#"const activatorRef = null
const menuRef = null
const decoy = null
"#;
    let template = r#"<div ref="activatorRef"><div ref="menuRef" /></div>"#;

    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);

    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    let output = generate_virtual_ts_with_offsets_and_checks(
        &summary,
        Some(script),
        Some(&root),
        0,
        0,
        &VirtualTsOptions::default(),
        VirtualTsGenerationOptions {
            preserve_unused_diagnostics: true,
            ..Default::default()
        },
    );

    assert!(!output.code.contains("void activatorRef;"));
    assert!(!output.code.contains("void menuRef;"));
    assert!(!output.code.contains("void decoy;"));
}

#[test]
fn test_preserve_unused_diagnostics_does_not_unwrap_static_template_refs() {
    let script = r#"import { ref } from 'vue'
const root = ref(null)
const decoy = null
"#;
    let template = r#"<div ref="root" />"#;

    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);

    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    let output = generate_virtual_ts_with_offsets_and_checks(
        &summary,
        Some(script),
        Some(&root),
        0,
        0,
        &VirtualTsOptions::default(),
        VirtualTsGenerationOptions {
            preserve_unused_diagnostics: true,
            ..Default::default()
        },
    );

    // The inline renderer reads the raw ref. It must stay outside the
    // unwrapped template scope while genuinely unused locals remain unused.
    assert!(output.code.contains("void (() => root);"));
    assert!(!output.code.contains("type __R_root = typeof root;"));
    assert!(!output.code.contains("var root: __U<__R_root>"));
    assert!(!output.code.contains("void decoy;"));
}

#[test]
fn test_preserve_unused_diagnostics_keeps_expression_reads_for_static_template_refs() {
    let script = r#"const activatorRef = null
const decoy = null
"#;
    let template = r#"<div ref="activatorRef">{{ activatorRef }}</div>"#;

    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);

    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    let output = generate_virtual_ts_with_offsets_and_checks(
        &summary,
        Some(script),
        Some(&root),
        0,
        0,
        &VirtualTsOptions::default(),
        VirtualTsGenerationOptions {
            preserve_unused_diagnostics: true,
            ..Default::default()
        },
    );

    assert!(output.code.contains("void activatorRef;"));
    assert!(!output.code.contains("void decoy;"));
}

/// The setup-scope `props` anchor must stay narrowed to templates that spell
/// `props`, even when unused-local diagnostics are not preserved. The LSP
/// generates with `preserve_unused_diagnostics: false` yet still reports
/// TS6133 as a hint, so anchoring on binding kind alone silences a genuine
/// unused `props` (a template reading prop names directly never reads it).
#[test]
fn test_props_shadow_anchor_requires_a_template_reference_to_props() {
    let anchor = "// Anchor before the template scope shadows `props`";
    let script = r#"const props = defineProps<{ label: string }>()
const other = 1
"#;

    let generate = |template: &str| {
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
            VirtualTsGenerationOptions::default(),
        )
        .code
    };

    let forwarded = generate(r#"<button v-bind="props"></button>"#);
    assert!(
        forwarded.contains(anchor),
        "a template forwarding `props` reads the setup binding: {forwarded}"
    );

    let unreferenced = generate(r#"<span>{{ other }}</span>"#);
    assert!(
        !unreferenced.contains(anchor),
        "a template that never spells `props` must keep its unused report: {unreferenced}"
    );
}

fn static_ref_read_block(
    script: &str,
    template: &str,
    dialect: vize_l0::config::VueVersion,
) -> std::string::String {
    let allocator = vize_carton::Allocator::new();
    let (root, errors) = vize_armature::parse(&allocator, template);
    assert_eq!(errors.len(), 0);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let mut blocks = Vec::new();
    for (check_template_bindings, options_api) in
        [(true, false), (false, false), (true, true), (false, true)]
    {
        let output = generate_virtual_ts_with_offsets_and_checks(
            &summary,
            Some(script),
            Some(&root),
            0,
            0,
            &VirtualTsOptions::default(),
            VirtualTsGenerationOptions {
                preserve_unused_diagnostics: true,
                dialect,
                options_api,
                check_options: crate::virtual_ts::VirtualTsCheckOptions {
                    check_template_bindings,
                    ..Default::default()
                },
                ..Default::default()
            },
        );
        println!(
            "static ref projection ({check_template_bindings}, options support {options_api}): {output:?}"
        );
        let marker = "  // Runtime reads for static template refs\n";
        let block = if let Some((before, after)) = output.code.split_once(marker) {
            assert_eq!(
                before.matches("function __template").count(),
                0,
                "raw read must precede every template shadow"
            );
            after
                .split_once("\n\n")
                .expect("complete raw anchor block")
                .0
                .to_owned()
        } else {
            std::string::String::new()
        };
        blocks.push(block);
    }
    let first = blocks.remove(0);
    for block in blocks {
        assert_eq!(
            first, block,
            "actual setup reads survive template-check and Options API support policies"
        );
    }
    first
}

#[test]
fn static_ref_runtime_reads_keep_original_setup_bindings_and_deduplicate() {
    use vize_l0::config::VueVersion;
    let script = "import { ref as vueRef } from 'vue'\nconst plain = vueRef(null)\nconst [fromComposable] = useBox()\nlet props = null\nconst decoy = null\nconst unused = vueRef(null)\n";
    assert_eq!(
        static_ref_read_block(
            script,
            r#"<div ref="plain"></div><div ref="fromComposable"></div><div ref="props"></div><div ref="plain"></div>"#,
            VueVersion::V3
        ),
        "  void (() => fromComposable);\n  void (() => plain);\n  void (() => props);",
    );
    assert_eq!(
        static_ref_read_block(
            script,
            r#"<div :ref="'plain'"></div><div ref="decoy"></div><div ref="missing"></div><div ref=" plain"></div><div ref="plain-x"></div>"#,
            VueVersion::V3
        ),
        "",
    );
    assert_eq!(
        static_ref_read_block(script, r#"<div ref="plain"></div>"#, VueVersion::V2_7),
        ""
    );
}

#[test]
fn static_ref_reads_respect_template_shadow_and_do_not_parse_ref_strings() {
    use vize_l0::config::VueVersion;
    let script = "import { ref } from 'vue'\nconst root = ref(null)\nconst other = ref(null)\n";
    for template in [
        r#"<div v-for="root in []" ref="root">{{ root }}</div>"#,
        r#"<Comp v-slot="{ root }"><div ref="root">{{ root }}</div></Comp>"#,
    ] {
        assert_eq!(static_ref_read_block(script, template, VueVersion::V3), "");
    }
    assert_eq!(
        static_ref_read_block(
            script,
            r#"<Comp ref="root" v-slot="{ root }">{{ root }}</Comp><div ref="other"></div>"#,
            VueVersion::V3
        ),
        "  void (() => other);\n  void (() => root);",
    );
    assert_eq!(
        static_ref_read_block(
            script,
            r#"<div v-for="root in []">{{ root }}</div><div ref="root"></div>"#,
            VueVersion::V3
        ),
        "  void (() => root);",
    );
}

#[test]
fn static_ref_reads_use_import_provenance_and_allow_real_non_vue_factory_results() {
    use vize_l0::config::VueVersion;
    let script = "import { ref as factory, useTemplateRef } from 'vue'\nimport { candidate as external } from './other'\nimport Component from './Comp.vue'\nimport externalDefault from './other'\nimport * as namespace from './other'\nimport type { Shape } from './other'\nconst local = factory(null)\nconst opaque = fakeUseTemplateRef()\nfunction fakeUseTemplateRef() { return null }\nconst plain = null\n";
    assert_eq!(
        static_ref_read_block(
            script,
            r#"<div ref="local"></div><div ref="opaque"></div><div ref="factory"></div><div ref="useTemplateRef"></div><div ref="Component"></div><div ref="Shape"></div><div ref="fakeUseTemplateRef"></div><div ref="plain"></div>"#,
            VueVersion::V3
        ),
        "  void (() => local);\n  void (() => opaque);",
    );
    // A named non-Vue value import follows the inline compiler's maybe-ref law.
    assert_eq!(
        static_ref_read_block(
            script,
            r#"<div ref="external"></div><div ref="externalDefault"></div><div ref="namespace"></div>"#,
            VueVersion::V3
        ),
        "  void (() => external);\n  void (() => externalDefault);"
    );
}

#[test]
fn uninitialized_static_ref_reads_are_deferred_like_the_inline_renderer() {
    assert_eq!(
        static_ref_read_block(
            "let el: HTMLElement;",
            "<div ref=\"el\"></div>",
            vize_l0::config::VueVersion::V3
        ),
        "  void (() => el);",
    );
}

#[test]
fn memo_dependencies_preserve_loop_scope_and_authored_projection() {
    let script = "import { isHidden } from './util'; defineProps<{ rows: { id: number }[] }>();";
    let expression = "[row.id, isHidden(), row.missing]";
    let template = format!(
        r#"<div v-for="row in rows" :key="row.id" v-memo="{expression}">{{{{ row.id }}}}</div>"#
    );
    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, &template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let dependencies = summary
        .template_expressions
        .iter()
        .filter(|entry| entry.content.as_str() == expression)
        .map(|entry| {
            (
                entry.kind.as_str(),
                entry.start as usize,
                entry.end as usize,
            )
        })
        .collect::<Vec<_>>();
    let start = template.find(expression).unwrap();
    assert_eq!(dependencies, [("VMemo", start, start + expression.len())]);
    let output = generate_virtual_ts_with_offsets_and_checks(
        &summary,
        Some(script),
        Some(&root),
        0,
        100,
        &VirtualTsOptions::default(),
        VirtualTsGenerationOptions {
            preserve_unused_diagnostics: true,
            ..Default::default()
        },
    );
    let mapped = output
        .mapping
        .spans()
        .iter()
        .filter(|span| span.src_range == (100 + start..100 + start + expression.len()))
        .map(|span| output.code.get(span.gen_range.clone()).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(mapped, [expression], "{}", output.code);
    let checks = output
        .code
        .lines()
        .filter(|line| line.ends_with("// VMemo"))
        .map(str::trim)
        .collect::<Vec<_>>();
    assert_eq!(
        checks,
        ["void ([row.id, isHidden(), row.missing]); // VMemo"]
    );
}
