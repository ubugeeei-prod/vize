use super::*;
use vize_croquis::{Analyzer, AnalyzerOptions};

#[test]
fn opaque_props_have_mapped_checks_for_every_bare_template_occurrence() {
    let template = "<h1>{{ titel }}</h1><p :title=\"titel\">{{ title }}</p>";
    for type_arg in [
        "ReturnType<typeof loader>",
        "Pick<{ title: string }, 'title'>",
    ] {
        let script = format!(
            "const loader = () => ({{ title: 'x' }}); type Props = {type_arg}; defineProps<Props>();"
        );
        for strict_instance_globals in [false, true] {
            let allocator = vize_carton::Allocator::new();
            let (root, _) = vize_armature::parse(&allocator, template);
            let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
            analyzer.analyze_script_setup(&script);
            analyzer.analyze_template(&root);
            let summary = analyzer.finish();
            let output = generate_virtual_ts_with_offsets(
                &summary,
                Some(&script),
                Some(&root),
                0,
                100,
                &VirtualTsOptions {
                    strict_instance_globals,
                    ..Default::default()
                },
            );
            assert!(
                output.code.contains("const __vize_keyed_template_context"),
                "{}",
                output.code
            );
            for (at, _) in template.match_indices("titel") {
                assert!(
                    output.mappings.iter().any(|mapping| {
                        mapping.src_range == (100 + at..100 + at + 5)
                            && output.code.get(mapping.gen_range.clone()) == Some("titel")
                            && output
                                .code
                                .get(..mapping.gen_range.start)
                                .is_some_and(|prefix| {
                                    prefix.ends_with("__vize_keyed_template_context.")
                                })
                    }),
                    "missing mapped keyed occurrence at {at}: {}",
                    output.code
                );
            }
        }
    }
}
