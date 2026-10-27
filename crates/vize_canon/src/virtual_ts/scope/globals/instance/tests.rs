//! Emission of `$`-prefixed template instance globals in every form.
#![expect(clippy::string_slice, reason = "tests assert by panicking")]

use crate::virtual_ts::{
    VirtualTsGenerationOptions, VirtualTsOptions, VirtualTsOutput,
    generate_virtual_ts_with_offsets, generate_virtual_ts_with_offsets_and_checks,
    generate_virtual_ts_with_offsets_options_api,
};
use vize_carton::config::VueVersion;
use vize_croquis::{Analyzer, AnalyzerOptions};

const SCRIPT: &str = "const label = 'x'\n";
const TEMPLATE: &str =
    r#"<p :title="$missing('c')">{{ $missing('a') }}{{ $missing('b') }}{{ label }}</p>"#;

fn generate(strict: bool) -> VirtualTsOutput {
    generate_for(strict, VueVersion::V3)
}

fn generate_for(strict: bool, dialect: VueVersion) -> VirtualTsOutput {
    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, TEMPLATE);

    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(SCRIPT);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    let options = VirtualTsOptions {
        strict_instance_globals: strict,
        ..Default::default()
    };
    if dialect == VueVersion::V3 {
        return generate_virtual_ts_with_offsets(
            &summary,
            Some(SCRIPT),
            Some(&root),
            0,
            0,
            &options,
        );
    }
    generate_virtual_ts_with_offsets_and_checks(
        &summary,
        Some(SCRIPT),
        Some(&root),
        0,
        0,
        &options,
        VirtualTsGenerationOptions {
            dialect,
            ..Default::default()
        },
    )
}

/// Authored offsets of the three `$missing` occurrences in `TEMPLATE`.
fn authored_offsets() -> Vec<usize> {
    let mut offsets = Vec::new();
    let mut from = 0;
    while let Some(found) = TEMPLATE[from..].find("$missing") {
        offsets.push(from + found);
        from += found + "$missing".len();
    }
    offsets
}

/// `vue-tsc` reports `TS2339` (`TS2551` with a spelling suggestion) at every
/// read of a `$` name the public instance does not declare (#10617).
#[test]
fn vue3_reads_every_occurrence_off_the_public_instance() {
    let output = generate(false);

    assert_eq!(
        output.code.matches("__VizeInstanceGlobal").count(),
        0,
        "{}",
        output.code
    );
    assert_eq!(
        output
            .code
            .matches("const $missing = __ctx.$missing;")
            .count(),
        1,
        "{}",
        output.code
    );
    assert_eq!(
        output.code.matches("void (__ctx.$missing);").count(),
        2,
        "{}",
        output.code
    );
}

#[test]
fn vue2_declares_the_name_once_with_an_any_fallback() {
    let output = generate_for(false, VueVersion::V2_7);

    assert!(
        output.code.contains(
            "type __VizeInstanceGlobal<K extends string> = K extends keyof __Ctx ? __Ctx[K] : any;"
        ),
        "{}",
        output.code
    );
    assert_eq!(
        output
            .code
            .matches("const $missing: __VizeInstanceGlobal<'$missing'> = undefined as any;")
            .count(),
        1,
        "{}",
        output.code
    );
    assert!(!output.code.contains("__ctx.$missing"), "{}", output.code);
}

#[test]
fn the_strict_form_reads_every_occurrence_off_the_template_context() {
    let output = generate(true);

    assert!(
        !output.code.contains("__VizeInstanceGlobal"),
        "{}",
        output.code
    );
    assert_eq!(
        output
            .code
            .matches("const $missing = __vize_strict_template_context.$missing;")
            .count(),
        1,
        "{}",
        output.code
    );
    assert_eq!(
        output
            .code
            .matches("void (__vize_strict_template_context.$missing);")
            .count(),
        2,
        "{}",
        output.code
    );
}

#[test]
fn typed_receivers_map_each_access_back_to_its_authored_occurrence() {
    for (strict, receiver) in [(true, "__vize_strict_template_context."), (false, "__ctx.")] {
        let output = generate(strict);

        let mut mapped = output
            .mapping
            .spans()
            .iter()
            .filter(|mapping| {
                output.code.get(mapping.gen_range.clone()) == Some("$missing")
                    && output.code[..mapping.gen_range.start].ends_with(receiver)
            })
            .map(|mapping| mapping.src_range.clone())
            .collect::<Vec<_>>();
        mapped.sort_by_key(|range| range.start);

        assert_eq!(
            mapped,
            authored_offsets()
                .into_iter()
                .map(|start| start..start + "$missing".len())
                .collect::<Vec<_>>(),
            "{}",
            output.code
        );
    }
}

#[test]
fn vue2_maps_the_single_declaration_to_the_first_occurrence() {
    let output = generate_for(false, VueVersion::V2_7);

    let mapped = output
        .mapping
        .spans()
        .iter()
        .filter(|mapping| output.code.get(mapping.gen_range.clone()) == Some("$missing"))
        .map(|mapping| mapping.src_range.clone())
        .collect::<Vec<_>>();

    let first = authored_offsets()[0];
    assert_eq!(mapped, vec![first..first + "$missing".len()]);
}

/// An Options API template resolves on its authored instance, where mixins
/// and `$`-named options members exist, as they do in `vue-tsc`'s context.
#[test]
fn options_api_reads_the_authored_instance() {
    let script = "import { defineComponent } from 'vue'\nexport default defineComponent({})\n";
    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, TEMPLATE);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full()).with_options_api();
    analyzer.analyze_script_plain(script);
    analyzer.analyze_template(&root);
    let output = generate_virtual_ts_with_offsets_options_api(
        &analyzer.finish(),
        Some(script),
        Some(&root),
        0,
        0,
        &VirtualTsOptions::default(),
    );

    assert_eq!(
        output
            .code
            .matches("  const __vize_instance_globals = undefined as unknown as (typeof __default__ extends abstract new (...args: any) => infer __I ? __I : __Ctx);\n")
            .count(),
        1,
        "{}",
        output.code
    );
    assert_eq!(
        output
            .code
            .matches("const $missing = __vize_instance_globals.$missing;")
            .count(),
        1,
        "{}",
        output.code
    );
    assert_eq!(
        output
            .code
            .matches("void (__vize_instance_globals.$missing);")
            .count(),
        2,
        "{}",
        output.code
    );
}

mod lexical_scope;
