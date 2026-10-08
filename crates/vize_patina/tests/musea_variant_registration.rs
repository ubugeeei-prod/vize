//! Whole original registration results across real Art script/variant frames.
#![expect(
    clippy::disallowed_macros,
    reason = "complete fixture result comparisons"
)]

use vize_patina::{LintDiagnostic, LintPreset, LintResult, Linter};

const BUTTON: &str = include_str!("fixtures/musea-variant-bindings/MyButton.art.vue.txt");
const CARD: &str = include_str!("fixtures/musea-variant-bindings/Card.art.vue.txt");
const RULE: &str = "vue/require-component-registration";

fn linter() -> Linter {
    Linter::with_preset(LintPreset::Incremental).with_enabled_rules(Some(vec![RULE.into()]))
}

fn expected(filename: &str, starts: &[usize], tag: &str) -> LintResult {
    LintResult {
        filename: filename.into(),
        diagnostics: starts
            .iter()
            .map(|&start| {
                LintDiagnostic::warn(
                    RULE,
                    "Component is used but not explicitly imported",
                    (start + 1) as u32,
                    (start + 1 + tag.len()) as u32,
                )
                .with_help(
                    "Import the component in <script setup> or register it in components option",
                )
            })
            .collect(),
        error_count: 0,
        warning_count: starts.len(),
    }
}

fn compare(actual: &LintResult, expected: &LintResult) {
    assert_eq!(format!("{actual:#?}"), format!("{expected:#?}"));
}

#[test]
fn original_imports_and_macro_target_are_registered_in_every_original_variant() {
    for filename in [
        "MyButton.art.vue",
        "/元/Anonymous.art.vue",
        r"C:\元\Anonymous.art.vue",
    ] {
        compare(
            &linter().lint_sfc(BUTTON, filename),
            &expected(filename, &[], ""),
        );
    }
    compare(
        &linter().lint_sfc(CARD, "Unrelated.art.vue"),
        &expected("Unrelated.art.vue", &[], ""),
    );
}

#[test]
fn removing_the_actual_import_preserves_the_entire_physical_missing_component_finding() {
    let source = BUTTON.replace("import MyIcon from \"./MyIcon.vue\";\n", "");
    let start = source.find("<MyIcon").unwrap();
    compare(
        &linter().lint_sfc(&source, "Anonymous.art.vue"),
        &expected("Anonymous.art.vue", &[start], "MyIcon"),
    );
}

#[test]
fn art_file_stem_cannot_register_an_unrelated_component() {
    let source = "<script setup>defineArt('./RealCard.vue', {title:'Real'});</script>\n<art><variant name=\"One\"><RealCard/><Unrelated/></variant></art>";
    let start = source.find("<Unrelated").unwrap();
    compare(
        &linter().lint_sfc(source, "Unrelated.art.vue"),
        &expected("Unrelated.art.vue", &[start], "Unrelated"),
    );
}

#[test]
fn each_missing_component_keeps_its_original_utf8_crlf_variant_coordinate() {
    let source = "<!--界-->\r\n<script setup>import MyIcon from './MyIcon.vue';</script>\r\n<art title=\"Icon\"><variant name=\"One\"><MyIcon/><MissingPanel/></variant>\r\n<variant name=\"Two\"><MyIcon/><MissingPanel/></variant></art>";
    let starts = source
        .match_indices("<MissingPanel")
        .map(|(start, _)| start)
        .collect::<Vec<_>>();
    assert_eq!(starts.len(), 2);
    compare(
        &linter().lint_sfc(source, "Gallery.art.vue"),
        &expected("Gallery.art.vue", &starts, "MissingPanel"),
    );
}

#[test]
fn different_art_blocks_do_not_share_their_demonstrated_component_attribute() {
    let source = "<art title=\"One\" component=\"./FirstCard.vue\"><variant name=\"One\"><FirstCard/></variant></art>\n<art title=\"Two\" component=\"./SecondCard.vue\"><variant name=\"Two\"><SecondCard/><FirstCard/></variant></art>";
    let start = source.rfind("<FirstCard").unwrap();
    compare(
        &linter().lint_sfc(source, "Gallery.art.vue"),
        &expected("Gallery.art.vue", &[start], "FirstCard"),
    );
}

#[test]
fn genuine_local_setup_bindings_are_visible_but_type_only_imports_are_not_values() {
    let source = "<script setup lang=\"ts\">import MyIcon from './MyIcon.vue';\nimport type TypeOnly from './TypeOnly.vue';\nconst LocalPanel = MyIcon;</script>\n<art title=\"Local\"><variant name=\"One\"><LocalPanel/><TypeOnly/></variant></art>";
    let start = source.find("<TypeOnly").unwrap();
    compare(
        &linter().lint_sfc(source, "Gallery.art.vue"),
        &expected("Gallery.art.vue", &[start], "TypeOnly"),
    );
}

#[test]
fn named_setup_values_keep_their_complete_registration_with_both_script_frames() {
    let source = "<script>const PlainPanel = {};</script>\n<script setup lang=\"ts\">const LocalPanel = {};let MutablePanel = {};function RenderPanel() {}class ClassPanel {};</script>\n<art><variant name=\"One\"><LocalPanel/><MutablePanel/><RenderPanel/><ClassPanel/><PlainPanel/></variant></art>";
    let start = source.find("<PlainPanel").unwrap();
    compare(
        &linter().lint_sfc(source, "Gallery.art.vue"),
        &expected("Gallery.art.vue", &[start], "PlainPanel"),
    );
}

#[test]
fn nested_and_erased_script_names_cannot_register_original_variants() {
    for declaration in [
        "{ const HiddenPanel = {}; }",
        "function outer() { const HiddenPanel = {}; }",
        "type HiddenPanel = {};",
        "interface HiddenPanel {}",
        "import { type HiddenPanel } from './Types';",
    ] {
        let source = format!(
            "<script setup lang=\"ts\">{declaration}</script>\n<art><variant name=\"One\"><HiddenPanel/></variant></art>"
        );
        let start = source.find("<HiddenPanel").unwrap();
        compare(
            &linter().lint_sfc(&source, "Gallery.art.vue"),
            &expected("Gallery.art.vue", &[start], "HiddenPanel"),
        );
    }
}

#[test]
fn ordinary_vue_import_registration_and_recursive_filename_reference_are_unchanged() {
    let source = "<script setup>import MyIcon from './MyIcon.vue';</script>\n<template><MyIcon/><Card/><MissingPanel/></template>";
    let start = source.find("<MissingPanel").unwrap();
    compare(
        &linter().lint_sfc(source, "Card.vue"),
        &expected("Card.vue", &[start], "MissingPanel"),
    );
}

#[test]
fn each_fragment_retains_its_own_template_facts_and_shared_original_registration() {
    use std::sync::Mutex;
    use vize_croquis::ScopeKind;
    use vize_patina::{LintContext, Rule, RuleCategory, RuleMeta, RuleRegistry, Severity};
    use vize_relief::RootNode;

    static AUDIT: RuleMeta = RuleMeta {
        name: "test/original-art-fragment-facts",
        description: "Retain complete original fragment facts",
        category: RuleCategory::Recommended,
        fixable: false,
        default_severity: Severity::Warning,
    };
    type FragmentFacts = Vec<(Vec<vize_l0::String>, usize)>;
    static FACTS: Mutex<FragmentFacts> = Mutex::new(Vec::new());
    struct Audit;
    impl Rule for Audit {
        fn meta(&self) -> &'static RuleMeta {
            &AUDIT
        }
        fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, _root: &RootNode<'a>) {
            let analysis = ctx.analysis().expect("actual original fragment analysis");
            let mut components = analysis.used_components.iter().cloned().collect::<Vec<_>>();
            components.sort();
            let locals = analysis
                .scopes
                .iter()
                .filter(|scope| scope.kind == ScopeKind::VFor)
                .count();
            FACTS.lock().unwrap().push((components, locals));
        }
    }
    let mut registry = RuleRegistry::with_all();
    registry.register(Box::new(Audit));
    let linter = Linter::with_registry(registry)
        .with_enabled_rules(Some(vec![RULE.into(), AUDIT.name.into()]));
    let source = "<script setup>import MyIcon from './MyIcon.vue';const rows = [1];</script>\n<art><variant name=\"One\"><MyIcon v-for=\"row in rows\" :key=\"row\"/></variant><variant name=\"Two\"><MyIcon/></variant></art>";
    compare(
        &linter.lint_sfc(source, "Gallery.art.vue"),
        &expected("Gallery.art.vue", &[], ""),
    );
    assert_eq!(
        *FACTS.lock().unwrap(),
        vec![(vec!["MyIcon".into()], 1), (vec!["MyIcon".into()], 0)]
    );
}
