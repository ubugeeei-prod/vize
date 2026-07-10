use crate::virtual_ts::{
    VirtualTsCheckOptions, VirtualTsGenerationOptions, VirtualTsOptions,
    generate_virtual_ts_with_offsets_and_checks,
};
use vize_croquis::{Analyzer, AnalyzerOptions};

#[test]
fn skips_template_scope_but_keeps_script_checks() {
    let script = r#"import { ref } from 'vue'
import Child from './Child.vue'

const scriptOnly: string = 1
const isLoading = ref(false)
"#;
    let template = r#"<div>
  <Child v-for="item in missingItems" :key="item.id" :busy="isLoading" @save="confirmDialog" />
  {{ confirmDialog }}
</div>"#;

    let allocator = vize_carton::Bump::new();
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
            check_options: VirtualTsCheckOptions {
                check_template_bindings: false,
                ..Default::default()
            },
            ..Default::default()
        },
    );

    assert!(output.code.contains("const scriptOnly: string = 1"));
    assert!(output.code.contains("__setup();"));
    assert!(!output.code.contains("Template Scope"));
    assert!(!output.code.contains("ComponentPublicInstance"));
    assert!(!output.code.contains("type __R_isLoading"));
    assert!(!output.code.contains("var isLoading:"));
    assert!(!output.code.contains("void isLoading;"));
    assert!(!output.code.contains("missingItems"));
    assert!(!output.code.contains("confirmDialog"));
    assert!(!output.code.contains("__vize_prop_check"));
    assert!(!output.code.contains("@save handler"));
}

#[test]
fn preserves_template_usage_anchors_when_template_checks_are_disabled() {
    let script = r#"import { ref } from 'vue'
import Child from './Child.vue'

const scriptOnly: string = 1
const isLoading = ref(false)
function confirmDialog() {}
"#;
    let template = r#"<div>
  <Child v-for="item in missingItems" :key="item.id" :busy="isLoading" @save="confirmDialog" />
</div>"#;

    let allocator = vize_carton::Bump::new();
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
            check_options: VirtualTsCheckOptions {
                check_template_bindings: false,
                ..Default::default()
            },
            preserve_unused_diagnostics: true,
            ..Default::default()
        },
    );

    assert!(output.code.contains("const scriptOnly: string = 1"));
    assert!(output.code.contains("__setup();"));
    assert!(!output.code.contains("Template Scope"));
    assert!(!output.code.contains("ComponentPublicInstance"));
    assert!(!output.code.contains("type __R_isLoading"));
    assert!(!output.code.contains("var isLoading:"));
    assert!(!output.code.contains("__vize_prop_check"));
    assert!(!output.code.contains("@save handler"));
    assert!(!output.code.contains("void missingItems;"));
    assert!(output.code.contains("void Child;"));
    assert!(output.code.contains("void confirmDialog;"));
    assert!(output.code.contains("void isLoading;"));
}
