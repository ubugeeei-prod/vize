use super::{VirtualTsOptions, VizeSemanticLinkKind, generate_virtual_ts_with_offsets};
use vize_croquis::{Analyzer, AnalyzerOptions};

#[test]
fn defaults_key_bridge_projects_only_its_exact_ast_owner_with_script_offsets() {
    for newline in ["\n", "\r\n"] {
        let script = [
            "type Props = { tone?: 'dark' | 'light' };",
            "const unrelated = { tone: 'dark' };",
            "void unrelated;",
            "withDefaults(defineProps<Props>(), { tone: 'light' });",
        ]
        .join(newline);
        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_script_setup(&script);
        let summary = analyzer.finish();
        let offset = 37_u32;
        let output = generate_virtual_ts_with_offsets(
            &summary,
            Some(&script),
            None,
            offset,
            0,
            &VirtualTsOptions::default(),
        );
        let key_start = script.rfind("tone:").unwrap() + offset as usize;
        let unrelated_start = script.find("tone:").unwrap() + offset as usize;
        let binding_links: Vec<_> = output
            .mapping
            .semantic_links()
            .iter()
            .filter(|link| link.kind == VizeSemanticLinkKind::VueTemplatePropBinding)
            .collect();
        assert_eq!(binding_links.len(), 2);
        assert_eq!(binding_links[0].source_range, binding_links[1].source_range);
        let key = binding_links[1];
        assert_eq!(output.code.get(key.source_range.clone()), Some("tone"));
        assert_eq!(output.code.get(key.target_range.clone()), Some("tone"));
        let mapping = output
            .mapping
            .rows()
            .filter(|row| row.span.gen_range.contains(&key.target_range.start))
            .min_by_key(|row| row.span.gen_range.len())
            .expect("copied default-key mapping");
        let projected =
            super::mapping::map_generated_offset_to_source(mapping.span, key.target_range.start);
        assert_eq!(projected, key_start);
        assert_ne!(projected, unrelated_start);
        assert_eq!(
            super::mapping::map_generated_offset_to_source(mapping.span, key.target_range.end),
            key_start + "tone".len(),
        );
    }
}
