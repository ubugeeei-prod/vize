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
            let mut actual = output
                .mapping
                .spans()
                .iter()
                .filter_map(|mapping| {
                    let before = output.code.get(..mapping.gen_range.start)?;
                    let line_start = before.rfind('\n').map_or(0, |at| at + 1);
                    let rest = output.code.get(mapping.gen_range.end..)?;
                    let line_end = mapping.gen_range.end + rest.find('\n').unwrap_or(rest.len());
                    let statement = output.code.get(line_start..line_end)?;
                    if statement != "  void (__vize_keyed_template_context.titel);" {
                        return None;
                    }
                    Some((
                        mapping.src_range.start,
                        mapping.src_range.end,
                        output.code.get(mapping.gen_range.clone())?,
                    ))
                })
                .collect::<Vec<_>>();
            actual.sort_unstable();
            let expected = template
                .match_indices("titel")
                .map(|(at, _)| (100 + at, 100 + at + 5, "titel"))
                .collect::<Vec<_>>();
            assert_eq!(actual, expected, "{}", output.code);
        }
    }
}
