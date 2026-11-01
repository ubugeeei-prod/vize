//! Typed inherited bindings and unchanged parse-error fallback.
use super::*;

/// A member inherited from an imported `extends:` base is declared from the
/// authored instance with its type, like every other unknown template name of
/// a Vue 3 Options API component; only a contextual keyword the undefined
/// reference pass cannot declare (`as`, which component APIs expose) keeps the
/// untyped inherited fallback.
#[test]
fn test_options_api_unresolved_extends_declares_template_refs_from_the_instance() {
    use vize_croquis::{Analyzer, AnalyzerOptions};

    let script = r#"import BaseButton from './BaseButton.vue'

export default {
    name: 'Button',
    extends: BaseButton
}
"#;
    let template = r#"<component v-if="!asChild" :is="as" :class="cx('root')">
  <span v-if="label">{{ label }}</span>
</component>"#;

    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);

    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full()).with_options_api();
    analyzer.analyze_script_plain(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let output = generate_virtual_ts_with_offsets_options_api(
        &summary,
        Some(script),
        Some(&root),
        0,
        0,
        &Default::default(),
    );

    for name in ["asChild", "cx", "label"] {
        assert_eq!(
            output
                .code
                .matches(&format!(
                    "  var {name}!: $AuthoredInstanceMember<\"{name}\">; void ({name});\n"
                ))
                .count(),
            1,
            "inherited `{name}` is declared once from the authored instance:\n{}",
            output.code
        );
        assert_eq!(
            output
                .code
                .matches(&format!("const {name}: any = undefined as any;"))
                .count(),
            0,
            "inherited `{name}` must not keep the untyped fallback:\n{}",
            output.code
        );
    }
    assert_eq!(
        output
            .code
            .matches("  const as: any = undefined as any;\n")
            .count(),
        1,
        "the contextual keyword `as` keeps the untyped inherited fallback:\n{}",
        output.code
    );
    assert_eq!(
        output
            .code
            .matches("$AuthoredInstanceMember<\"as\">")
            .count(),
        0,
        "the contextual keyword `as` is not declared by the instance pass:\n{}",
        output.code
    );
}

/// The instance pass classifies names against the script's top-level bindings,
/// which it can read only from a cleanly parsed script; with parse diagnostics
/// it keeps the free-name emission, so the inherited `any` fallback must stay
/// for every name instead of leaving them undeclared.
#[test]
fn test_options_api_unresolved_extends_keeps_the_fallback_when_the_script_has_parse_errors() {
    use crate::virtual_ts::scope::authored_instance_declares_unknown_names;
    use vize_croquis::{Analyzer, AnalyzerOptions};

    assert!(authored_instance_declares_unknown_names(None));
    assert!(authored_instance_declares_unknown_names(Some(
        "import BaseButton from './BaseButton.vue'\nexport default { extends: BaseButton }\n"
    )));

    // `await` outside an async function is an early error oxc reports without
    // abandoning the parse, so the options analysis still sees the imported
    // `extends:` while the instance pass cannot read the script's top-level
    // bindings.
    let script = r#"import BaseButton from './BaseButton.vue'
function warm() { await 1 }

export default {
    name: 'Button',
    extends: BaseButton
}
"#;
    assert!(!authored_instance_declares_unknown_names(Some(script)));
    let template = r#"<span v-if="label">{{ label }}</span>"#;

    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);

    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full()).with_options_api();
    analyzer.analyze_script_plain(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let output = generate_virtual_ts_with_offsets_options_api(
        &summary,
        Some(script),
        Some(&root),
        0,
        0,
        &Default::default(),
    );

    assert_eq!(
        output
            .code
            .matches("  const label: any = undefined as any;\n")
            .count(),
        1,
        "a script with parse diagnostics keeps the inherited fallback for `label`:\n{}",
        output.code
    );
    assert_eq!(
        output
            .code
            .matches("extends keyof typeof __vize_template_instance")
            .count(),
        0,
        "the instance pass does not declare names for a script with parse diagnostics:\n{}",
        output.code
    );
}
