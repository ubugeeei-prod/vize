//! Emission of `$`-prefixed template instance globals in both forms.

use crate::virtual_ts::{VirtualTsOptions, VirtualTsOutput, generate_virtual_ts_with_offsets};
use vize_croquis::{Analyzer, AnalyzerOptions};

const SCRIPT: &str = "const label = 'x'\n";
const TEMPLATE: &str =
    r#"<p :title="$missing('c')">{{ $missing('a') }}{{ $missing('b') }}{{ label }}</p>"#;

fn generate(strict: bool) -> VirtualTsOutput {
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
    generate_virtual_ts_with_offsets(&summary, Some(SCRIPT), Some(&root), 0, 0, &options)
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

#[test]
fn the_permissive_form_declares_the_name_once_with_an_any_fallback() {
    let output = generate(false);

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
fn the_strict_form_maps_each_access_back_to_its_authored_occurrence() {
    let output = generate(true);

    let mut mapped = output
        .mappings
        .iter()
        .filter(|mapping| {
            output.code.get(mapping.gen_range.clone()) == Some("$missing")
                && output.code[..mapping.gen_range.start]
                    .ends_with("__vize_strict_template_context.")
        })
        .map(|mapping| mapping.src_range.clone())
        .collect::<Vec<_>>();
    mapped.sort_by_key(|range| range.start);

    assert_eq!(
        mapped,
        authored_offsets()
            .into_iter()
            .map(|start| start..start + "$missing".len())
            .collect::<Vec<_>>()
    );
}

#[test]
fn the_permissive_form_maps_the_single_declaration_to_the_first_occurrence() {
    let output = generate(false);

    let mapped = output
        .mappings
        .iter()
        .filter(|mapping| output.code.get(mapping.gen_range.clone()) == Some("$missing"))
        .map(|mapping| mapping.src_range.clone())
        .collect::<Vec<_>>();

    let first = authored_offsets()[0];
    assert_eq!(mapped, vec![first..first + "$missing".len()]);
}
