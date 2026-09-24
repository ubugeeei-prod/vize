//! Template scopes must survive the strict Nuxt template-global path.
//!
//! `generate_strict_expression_refs` resolves each template identifier through
//! `Croquis::bindings_visible_at`, which starts at the smallest-span scope
//! containing the offset. Script scope spans are measured over the script text
//! and template scope spans over the template text, so for any template offset
//! below the script's length both ranges match and the script scope wins.
//!
//! Every case below therefore pairs a `<script setup>` long enough to cover the
//! template offsets under test with the corresponding `v-for` / `v-slot` read,
//! which is exactly elk's shape (#4423). A short script never reproduces the
//! collision, so these scripts are load-bearing and must not be trimmed.
#![expect(clippy::string_slice, reason = "tests assert by panicking")]
#![expect(clippy::disallowed_methods, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]

use super::{VirtualTsOptions, generate_virtual_ts_with_offsets};
use vize_croquis::{Analyzer, AnalyzerOptions};

/// elk `CommonRouteTabs.vue` is 139 script characters with the `:key` read at
/// template offset 99; both numbers matter, so assert the collision is real
/// rather than trusting the literals to stay in range.
const SCRIPT: &str = "interface Option { name: string, to: string, hide?: boolean, disabled?: boolean }\nconst { options } = defineProps<{ options: Option[] }>()\n";

fn strict_virtual_ts(template: &str, script: Option<&str>) -> String {
    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    if let Some(script) = script {
        analyzer.analyze_script_setup(script);
    }
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    generate_virtual_ts_with_offsets(
        &summary,
        script,
        Some(&root),
        0,
        0,
        &VirtualTsOptions {
            strict_instance_globals: true,
            ..Default::default()
        },
    )
    .code
    .to_string()
}

fn strict_context_reads(template: &str, script: Option<&str>) -> Vec<String> {
    let code = strict_virtual_ts(template, script);
    let mut reads = Vec::new();
    for (index, _) in code.match_indices("__vize_strict_template_context.") {
        let rest = &code[index + "__vize_strict_template_context.".len()..];
        let end = rest
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '$'))
            .unwrap_or(rest.len());
        reads.push(rest[..end].to_string());
    }
    reads.sort();
    reads.dedup();
    reads
}

#[test]
fn v_for_aliases_read_before_the_script_length_are_not_strict_context_reads() {
    // `:key` sits at template offset 99, inside the 139-character script span.
    let template = "<div>\n    <template\n      v-for=\"(option, index) in options.filter(item => !item.hide)\"\n      :key=\"option?.name || index\"\n    >\n      <a :href=\"option.to\">{{ option.name }}</a>\n    </template>\n  </div>";
    assert!(
        template.find(":key=\"option").unwrap() + 6 < SCRIPT.len(),
        "the `:key` read must fall inside the script span or the case is vacuous"
    );

    assert_eq!(
        strict_context_reads(template, Some(SCRIPT)),
        Vec::<String>::new()
    );
}

/// A nested `v-for` puts a second template scope over the same offsets, so the
/// outer alias has to stay visible from inside the inner one as well.
#[test]
fn nested_v_for_aliases_are_not_strict_context_reads() {
    let template = "<div>\n    <template v-for=\"(option, index) in options\" :key=\"index\">\n      <b v-for=\"tag in option.tags\" :key=\"tag + index\">{{ option.name }}</b>\n    </template>\n  </div>";
    assert!(
        template.find(":key=\"tag + index\"").unwrap() < SCRIPT.len(),
        "the inner read must fall inside the script span or the case is vacuous"
    );

    assert_eq!(
        strict_context_reads(template, Some(SCRIPT)),
        Vec::<String>::new()
    );
}

/// The negative control for both cases above: closing the false positives must
/// not silence a name nothing declares, including one read at the very same
/// offsets from inside the same `v-for`.
#[test]
fn undeclared_names_inside_a_template_scope_still_read_the_strict_context() {
    let template = "<div>\n    <template\n      v-for=\"(option, index) in options.filter(item => !item.hide)\"\n      :key=\"missingKey || index\"\n    >\n      <a :href=\"option.to\">{{ missingBody }}</a>\n    </template>\n  </div>";

    assert_eq!(
        strict_context_reads(template, Some(SCRIPT)),
        vec!["missingBody".to_string(), "missingKey".to_string()]
    );
}

#[test]
fn dynamic_is_before_or_after_same_element_v_for_sees_its_alias() {
    let script = "const parts = [{ type: 'text', key: '0', value: 'a' }];";
    for template in [
        "<component :is=\"part.type === 'text' ? 'span' : 'a'\" v-for=\"part in parts\" :key=\"part.key\">{{ part.value }}</component>",
        "<component v-for=\"part in parts\" :is=\"part.type === 'text' ? 'span' : 'a'\" :key=\"part.key\">{{ part.value }}</component>",
    ] {
        assert_eq!(
            strict_context_reads(template, Some(script)),
            Vec::<String>::new()
        );
        let code = strict_virtual_ts(template, Some(script));
        assert!(
            !code.contains("const __vize_dynamic_is_"),
            "the dynamic target must not escape its v-for scope:\n{code}"
        );
    }
}

#[test]
fn dynamic_is_in_same_element_v_for_still_reports_unknown_name() {
    let script = "const parts = [{ type: 'text' }];";
    let template = "<component :is=\"missing.type === 'text' ? 'span' : 'a'\" v-for=\"part in parts\">{{ part.type }}</component>";
    assert_eq!(
        strict_context_reads(template, Some(script)),
        vec!["missing"]
    );
}

#[test]
fn dynamic_is_independent_of_same_element_v_for_keeps_component_inference() {
    let script = "const parts = [1]; const flag = true;";
    let template =
        "<component :is=\"flag ? 'span' : 'a'\" v-for=\"part in parts\">{{ part }}</component>";
    let code = strict_virtual_ts(template, Some(script));
    assert!(
        code.contains("const __vize_dynamic_is_"),
        "an independent :is expression should keep its inferred component target:\n{code}"
    );
}

#[test]
fn same_element_v_if_still_cannot_read_v_for_alias() {
    let script = "const parts = [{ type: 'text' }];";
    for template in [
        "<component v-if=\"part.type === 'text'\" v-for=\"part in parts\" :is=\"'span'\" />",
        "<component v-for=\"part in parts\" v-if=\"part.type === 'text'\" :is=\"'span'\" />",
    ] {
        assert_eq!(strict_context_reads(template, Some(script)), vec!["part"]);
    }
}
