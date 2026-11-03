//! Where the props-independent emit-props gate applies, and where the direct
//! call stays. `tests/tooling/canon-emit-props-gate-equivalence.test.ts`
//! proves the two calls have the same type wherever the gate applies.
use super::{GATE_ALIASES, is_single_line};
use crate::virtual_ts::generic_parameters::GenericParameters;
use crate::virtual_ts::{
    VirtualTsGenerationOptions, VirtualTsOptions, generate_virtual_ts_with_offsets_and_checks,
};
use vize_croquis::{Analyzer, AnalyzerOptions};

const SCRIPT: &str = r#"import Child from './Child.vue'
import Other from './Other.vue'
const pick = true
const value = 'text'
const attrs = { value: 'text' }
const onSave = (value: number) => value
"#;

const GATED: &str = "const __vize_emit_factory_0 = (undefined as unknown as __vize_emit_resolver<";
const DIRECT: &str = "_0_emit_resolver)(";
const ALIASES: &str = "type __vize_emit_input<";
const COMPUTED: &str =
    r#"<component :is="pick ? Child : Other" :value="value" @save="onSave"></component>"#;

fn generate(template: &str, options: VirtualTsGenerationOptions<'_>) -> vize_carton::String {
    let allocator = vize_carton::Allocator::new();
    let (root, errors) = vize_armature::parse(&allocator, template);
    assert!(errors.is_empty(), "{errors:?}");
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(SCRIPT);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    generate_virtual_ts_with_offsets_and_checks(
        &summary,
        Some(SCRIPT),
        Some(&root),
        0,
        0,
        &VirtualTsOptions::default(),
        options,
    )
    .code
}

#[test]
fn a_computed_is_listener_takes_the_gated_call_in_both_modes() {
    for (hoist_shared_preamble, factory) in [
        (false, "$EmitPropsFactory<C>"),
        (true, "import(\"vize:virtual-ts\").$EmitPropsFactory<C>"),
    ] {
        let code = generate(
            COMPUTED,
            VirtualTsGenerationOptions {
                hoist_shared_preamble,
                ..Default::default()
            },
        );
        assert_eq!(code.matches(GATED).count(), 1, "{code}");
        assert_eq!(code.matches(DIRECT).count(), 0, "{code}");
        assert_eq!(code.matches(ALIASES).count(), 1, "{code}");
        assert_eq!(
            code.matches(vize_carton::cstr!("(component: C) => C : {factory};").as_str())
                .count(),
            1,
            "{code}"
        );
        assert!(
            code.contains(
                "const __vize_emit_props_0 = (undefined as unknown as __vize_emit_input<"
            ),
            "{code}"
        );
    }
}

#[test]
fn other_usages_keep_the_direct_call() {
    let generic = GenericParameters::parse("T");
    let multi_line = "<component :is=\"pick ? Child : Other\" :value=\"[\n  value,\n]\" @save=\"onSave\"></component>";
    for (name, template, options, aliases) in [
        (
            "static tag",
            r#"<Child :value="value" @save="onSave" />"#,
            VirtualTsGenerationOptions::default(),
            0,
        ),
        (
            "named :is",
            r#"<component :is="Child" :value="value" @save="onSave"></component>"#,
            VirtualTsGenerationOptions::default(),
            0,
        ),
        (
            "generic template",
            COMPUTED,
            VirtualTsGenerationOptions {
                generic_parameters: Some(&generic),
                ..Default::default()
            },
            0,
        ),
        (
            "legacy Vue 2",
            COMPUTED,
            VirtualTsGenerationOptions {
                legacy_vue2: true,
                ..Default::default()
            },
            0,
        ),
        (
            "multi-line value",
            multi_line,
            VirtualTsGenerationOptions::default(),
            1,
        ),
    ] {
        let code = generate(template, options);
        assert_eq!(code.matches(GATED).count(), 0, "{name}\n{code}");
        assert_eq!(code.matches(ALIASES).count(), aliases, "{name}\n{code}");
        assert_eq!(code.matches(DIRECT).count(), 1, "{name}\n{code}");
    }
}

#[test]
fn modules_without_a_gated_candidate_declare_no_aliases() {
    for template in [
        r#"<component :is="pick ? Child : Other" v-bind="attrs" @save="onSave"></component>"#,
        r#"<component :is="pick ? Child : Other" @save="onSave"></component>"#,
        r#"<component :is="pick ? Child : Other" :value="value"></component>"#,
    ] {
        let code = generate(template, VirtualTsGenerationOptions::default());
        assert_eq!(code.matches(ALIASES).count(), 0, "{template}\n{code}");
        assert_eq!(code.matches(GATED).count(), 0, "{template}\n{code}");
    }
}

#[test]
fn line_terminators_follow_typescript() {
    assert!(is_single_line("columns.find(c => c.id === id)!"));
    for value in ["a\nb", "a\rb", "a\u{2028}b", "a\u{2029}b"] {
        assert!(!is_single_line(value), "{value:?}");
    }
}

#[test]
fn aliases_name_the_mode_factory() {
    for (hoisted, factory) in [
        (false, "$EmitPropsFactory<C>"),
        (true, "import(\"vize:virtual-ts\").$EmitPropsFactory<C>"),
    ] {
        let text = GATE_ALIASES.replace(
            "{emit_props_factory}",
            crate::virtual_ts::helpers::shared_types::EMIT_PROPS_FACTORY.select(hoisted),
        );
        assert_eq!(text.matches(factory).count(), 1, "{text}");
    }
}
