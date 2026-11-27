//! Template `ref` attributes as reads of setup bindings.

use crate::{Analyzer, AnalyzerOptions};
use vize_carton::Allocator;

fn assert_unused(script: &str, template: &str, expected: &[&str]) {
    let allocator = Allocator::new();
    let (root, errors) = vize_armature::parse(&allocator, template);
    assert!(errors.is_empty());
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full()).with_unused_bindings();
    analyzer.draw_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let mut names: Vec<&str> = summary
        .unused_bindings
        .iter()
        .map(|name| name.as_str())
        .collect();
    names.sort_unstable();
    assert_eq!(names, expected);
}

#[test]
fn a_ref_attribute_reads_the_binding_it_names() {
    assert_unused(
        "import { ref } from 'vue'; const el = ref(null);",
        "<div ref=\"el\"></div>",
        &[],
    );
}

#[test]
fn a_ref_attribute_does_not_read_a_template_ref_handle_of_the_same_name() {
    assert_unused(
        "import { useTemplateRef } from 'vue'; const el = useTemplateRef('el'); \
         const other = useTemplateRef('field');",
        "<div><div ref=\"el\"></div><div :ref=\"'other'\"></div><div ref=\"field\"></div></div>",
        &["el", "other"],
    );
}

#[test]
fn a_template_ref_handle_read_by_an_expression_stays_used() {
    assert_unused(
        "import { useTemplateRef } from 'vue'; const el = useTemplateRef('el');",
        "<div ref=\"el\">{{ el?.tagName }}</div>",
        &[],
    );
}

#[test]
fn template_ref_handles_resolve_by_binding_origin() {
    assert_unused(
        "import { useTemplateRef as utr } from 'vue'; const el = utr('el');",
        "<div ref=\"el\"></div>",
        &["el"],
    );
    assert_unused(
        "import { ref } from 'vue'; const useTemplateRef = ref; const el = useTemplateRef(null);",
        "<div ref=\"el\"></div>",
        &[],
    );
    assert_unused(
        "import { useTemplateRef } from './helpers'; const el = useTemplateRef('el');",
        "<div ref=\"el\"></div>",
        &[],
    );
}
