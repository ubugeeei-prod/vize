use vize_croquis::{Analyzer, AnalyzerOptions, Croquis};
use vize_l0::{Allocator, CompactString, FxHashSet};

use super::super::super::template_record::TemplateRecord;
use crate::virtual_ts::types::{VirtualTsCheckOptions, VirtualTsOptions};

fn registry_body(script: &str, template: &str) -> vize_l0::String {
    let allocator = Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary: Croquis = analyzer.finish();
    super::template_ref_registry(
        &summary,
        &VirtualTsOptions::default(),
        true,
        Some(&root),
        &FxHashSet::<CompactString>::default(),
        (VirtualTsCheckOptions::default(), &TemplateRecord::default()),
    )
    .expect("registry")
    .body
}

#[test]
fn a_ref_valued_dynamic_target_is_read_unwrapped_like_the_template() {
    let script = r#"import { ref, useTemplateRef } from "vue";
const tag = ref<"div" | "span">("div");
const inner = useTemplateRef("inner");
"#;
    assert_eq!(
        registry_body(script, r#"<component :is="tag" ref="inner" />"#).as_str(),
        r#" "inner": $TemplateComponentRef<import('vue').UnwrapRef<typeof tag>>; "#
    );
}

#[test]
fn a_static_is_names_its_target_directly() {
    let script = r#"import { useTemplateRef } from "vue";
const inner = useTemplateRef("inner");
"#;
    assert_eq!(
        registry_body(script, r#"<component is="div" ref="inner" />"#).as_str(),
        r#" "inner": $DomElement<"div">; "#
    );
    // `vue:` forces a component even for a native spelling.
    assert_ne!(
        registry_body(script, r#"<component is="vue:div" ref="inner" />"#).as_str(),
        r#" "inner": $DomElement<"div">; "#
    );
}
