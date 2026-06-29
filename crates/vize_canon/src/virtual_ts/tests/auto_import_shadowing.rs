use crate::virtual_ts::{VirtualTsOptions, generate_virtual_ts_with_offsets};
use vize_croquis::{Analyzer, AnalyzerOptions};

#[test]
fn test_type_only_imports_do_not_shadow_auto_imported_components() {
    let script = r#"/// <reference types="nuxt" />
import type { Differences } from './types'
const count = 1
"#;
    let template = r#"<Differences :count="count" />"#;

    let allocator = vize_carton::Bump::new();
    let (root, _) = vize_armature::parse(&allocator, template);

    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    let options = VirtualTsOptions {
        auto_import_stubs: vec![
            "declare const Differences: typeof import('./components/Differences.vue')['default'];"
                .into(),
        ],
        ..Default::default()
    };

    let output =
        generate_virtual_ts_with_offsets(&summary, Some(script), Some(&root), 0, 0, &options);

    assert!(
        output
            .code
            .contains("declare const Differences: typeof import"),
        "{}",
        output.code
    );
    assert!(!output.code.contains("const Differences: any"));
    assert!(
        output
            .code
            .contains("type __Differences_Props_0 = typeof Differences"),
        "{}",
        output.code
    );
}
