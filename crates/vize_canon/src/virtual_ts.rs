//! Virtual TypeScript generation for Vue SFC type checking.
//!
//! This module generates TypeScript code that represents a Vue SFC's
//! runtime behavior, enabling type checking of template expressions
//! and script setup bindings.
//!
//! Key design: Uses closures from Croquis scope information instead of
//! `declare const` to properly model Vue's template scoping.

mod expressions;
mod generator;
mod helpers;
pub mod incremental;
mod props;
mod scope;
mod types;

#[cfg(any(test, feature = "native"))]
pub(crate) use generator::generate_virtual_ts_with_offsets_and_checks;
pub use generator::{
    generate_virtual_ts, generate_virtual_ts_with_offsets,
    generate_virtual_ts_with_offsets_legacy_vue2,
};
pub use types::{TemplateGlobal, VirtualTsOptions, VirtualTsOutput, VizeMapping};
#[cfg(any(test, feature = "native"))]
pub(crate) use types::{VirtualTsCheckOptions, VirtualTsGenerationOptions};

#[cfg(test)]
mod tests {
    use super::helpers::{VUE_SETUP_HELPERS, generate_template_context, get_dom_event_type};
    use super::{
        TemplateGlobal, VirtualTsCheckOptions, VirtualTsGenerationOptions, VirtualTsOptions,
        generate_virtual_ts, generate_virtual_ts_with_offsets,
        generate_virtual_ts_with_offsets_and_checks,
    };

    fn assert_virtual_ts_snapshot(name: &str, value: &str) {
        insta::with_settings!({
            snapshot_path => "../snapshots"
        }, {
            insta::assert_snapshot!(name, value);
        });
    }

    #[test]
    fn test_vue_setup_helpers_are_actual_functions() {
        assert_virtual_ts_snapshot("virtual_ts_vue_setup_helpers", VUE_SETUP_HELPERS);
    }

    #[test]
    fn test_vue_template_context() {
        // Template context should contain Vue instance properties
        let ctx = generate_template_context(&VirtualTsOptions::default());
        assert_virtual_ts_snapshot("virtual_ts_vue_template_context", ctx.as_str());
    }

    #[test]
    fn test_vue_template_context_with_globals() {
        // Plugin globals should appear when configured
        let options = VirtualTsOptions {
            template_globals: vec![
                TemplateGlobal {
                    name: "$t".into(),
                    type_annotation: "(...args: any[]) => string".into(),
                    default_value: "(() => '') as any".into(),
                },
                TemplateGlobal {
                    name: "$route".into(),
                    type_annotation: "any".into(),
                    default_value: "{} as any".into(),
                },
            ],
            ..Default::default()
        };
        let ctx = generate_template_context(&options);
        assert_virtual_ts_snapshot("virtual_ts_vue_template_context_with_globals", ctx.as_str());
    }

    #[test]
    fn test_const_auto_import_stubs_skip_imported_names() {
        use vize_croquis::{Analyzer, AnalyzerOptions};

        let script = r#"import { currentUser } from './users'
const count = 1
"#;

        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_script_setup(script);
        let summary = analyzer.finish();

        let options = VirtualTsOptions {
            auto_import_stubs: vec![
                "declare const currentUser: any;".into(),
                "declare const useHydratedHead: any;".into(),
            ],
            ..Default::default()
        };

        let output = generate_virtual_ts_with_offsets(&summary, Some(script), None, 0, 0, &options);

        assert_virtual_ts_snapshot(
            "virtual_ts_auto_import_stubs_skip_imported_names",
            output.code.as_str(),
        );
    }

    #[test]
    fn test_external_template_bindings_do_not_shadow_auto_imported_components() {
        use vize_croquis::{Analyzer, AnalyzerOptions};

        let script = "const count = 'oops'\n";
        let template = r#"<AutoCard :count="count" />"#;

        let allocator = vize_carton::Bump::new();
        let (root, _) = vize_armature::parse(&allocator, template);

        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_script_setup(script);
        analyzer.analyze_template(&root);
        let summary = analyzer.finish();

        let options = VirtualTsOptions {
            auto_import_stubs: vec![
                "declare const AutoCard: typeof import('./components/AutoCard.vue.ts')['default'];"
                    .into(),
            ],
            external_template_bindings: vec!["AutoCard".into()],
            ..Default::default()
        };
        let output =
            generate_virtual_ts_with_offsets(&summary, Some(script), Some(&root), 0, 0, &options);

        assert!(
            output
                .code
                .contains("declare const AutoCard: typeof import")
        );
        assert!(!output.code.contains("const AutoCard: any"));
        assert!(
            output
                .code
                .contains("type __AutoCard_Props_0 = typeof AutoCard")
        );
    }

    #[test]
    fn test_template_instance_globals_delegate_to_component_public_instance() {
        use vize_croquis::{Analyzer, AnalyzerOptions};

        let template = r#"<button :title="$t('hello')">{{ missing }}</button>"#;

        let allocator = vize_carton::Bump::new();
        let (root, _) = vize_armature::parse(&allocator, template);

        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_template(&root);
        let summary = analyzer.finish();

        let output = generate_virtual_ts_with_offsets(
            &summary,
            None,
            Some(&root),
            0,
            0,
            &VirtualTsOptions::default(),
        );

        assert!(
            output
                .code
                .contains("const $t: __VizeInstanceGlobal<'$t'> = undefined as any;"),
            "{}",
            output.code
        );
        assert!(output.code.contains("void ($t('hello'));"));
        assert!(output.code.contains("void (missing);"));
        assert!(!output.code.contains("void ($t);"));

        let configured_output = generate_virtual_ts_with_offsets(
            &summary,
            None,
            Some(&root),
            0,
            0,
            &VirtualTsOptions {
                template_globals: vec![TemplateGlobal {
                    name: "$t".into(),
                    type_annotation: "(key: string) => string".into(),
                    default_value: "(() => '') as any".into(),
                }],
                ..Default::default()
            },
        );

        assert!(
            !configured_output
                .code
                .contains("__VizeInstanceGlobal<'$t'>")
        );
        assert!(
            configured_output
                .code
                .contains("const $t: __Global<'$t', (key: string) => string>")
        );
    }

    #[test]
    fn test_kebab_case_component_names_are_sanitized_in_type_helpers() {
        use vize_croquis::{Analyzer, AnalyzerOptions};

        let script = r#"const value = 'hello'
function handleUpdate(value: string) {
  void value
}
"#;
        let template = r#"<my-widget :label="value" @update:model-value="handleUpdate" />"#;

        let allocator = vize_carton::Bump::new();
        let (root, _) = vize_armature::parse(&allocator, template);

        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_script_setup(script);
        analyzer.analyze_template(&root);
        let summary = analyzer.finish();

        let output = generate_virtual_ts_with_offsets(
            &summary,
            Some(script),
            Some(&root),
            0,
            0,
            &Default::default(),
        );

        assert_virtual_ts_snapshot(
            "virtual_ts_kebab_case_component_names",
            output.code.as_str(),
        );
    }

    #[test]
    fn test_check_props_option_disables_component_prop_checks() {
        use vize_croquis::{Analyzer, AnalyzerOptions};

        let script = r#"import Child from './Child.vue'
const wrong = 'not a number'
"#;
        let template = r#"<Child :count="wrong" />"#;

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
                    check_props: false,
                    ..Default::default()
                },
                ..Default::default()
            },
        );

        assert!(!output.code.contains("__vize_prop_check"));
        assert!(!output.code.contains("type __Child_Props_0"));
    }

    #[test]
    fn test_check_template_bindings_option_disables_template_expressions() {
        use vize_croquis::{Analyzer, AnalyzerOptions};

        let script = "const message = 'hello'\n";
        let template = r#"<div>{{ message }}</div>"#;

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

        assert!(!output.code.contains("void (message);"));
    }

    #[test]
    fn test_dom_event_type_mapping() {
        // Mouse events
        assert_eq!(get_dom_event_type("click"), "MouseEvent");
        assert_eq!(get_dom_event_type("dblclick"), "MouseEvent");
        assert_eq!(get_dom_event_type("mousedown"), "MouseEvent");
        assert_eq!(get_dom_event_type("mouseup"), "MouseEvent");
        assert_eq!(get_dom_event_type("mousemove"), "MouseEvent");
        assert_eq!(get_dom_event_type("contextmenu"), "MouseEvent");

        // Pointer events
        assert_eq!(get_dom_event_type("pointerdown"), "PointerEvent");
        assert_eq!(get_dom_event_type("pointerup"), "PointerEvent");

        // Touch events
        assert_eq!(get_dom_event_type("touchstart"), "TouchEvent");
        assert_eq!(get_dom_event_type("touchend"), "TouchEvent");

        // Keyboard events
        assert_eq!(get_dom_event_type("keydown"), "KeyboardEvent");
        assert_eq!(get_dom_event_type("keyup"), "KeyboardEvent");
        assert_eq!(get_dom_event_type("keypress"), "KeyboardEvent");

        // Focus events
        assert_eq!(get_dom_event_type("focus"), "FocusEvent");
        assert_eq!(get_dom_event_type("blur"), "FocusEvent");

        // Input events
        assert_eq!(get_dom_event_type("input"), "InputEvent");
        assert_eq!(get_dom_event_type("beforeinput"), "InputEvent");

        // Form events
        assert_eq!(get_dom_event_type("submit"), "SubmitEvent");
        assert_eq!(get_dom_event_type("change"), "Event");

        // Drag events
        assert_eq!(get_dom_event_type("drag"), "DragEvent");
        assert_eq!(get_dom_event_type("drop"), "DragEvent");

        // Clipboard events
        assert_eq!(get_dom_event_type("copy"), "ClipboardEvent");
        assert_eq!(get_dom_event_type("paste"), "ClipboardEvent");

        // Wheel events
        assert_eq!(get_dom_event_type("wheel"), "WheelEvent");

        // Animation events
        assert_eq!(get_dom_event_type("animationstart"), "AnimationEvent");
        assert_eq!(get_dom_event_type("animationend"), "AnimationEvent");

        // Transition events
        assert_eq!(get_dom_event_type("transitionend"), "TransitionEvent");

        // Unknown/custom events fallback to Event
        assert_eq!(get_dom_event_type("customEvent"), "Event");
        assert_eq!(get_dom_event_type("unknown"), "Event");
    }

    #[test]
    fn test_vfor_destructuring_scope() {
        use vize_croquis::{Analyzer, AnalyzerOptions};

        let script = r#"import { ref } from 'vue'
const items = ref([{ id: 1, name: 'Hello' }])
"#;
        let template = r#"<ul>
  <li v-for="{ id, name } in items" :key="id">
    {{ id }}: {{ name }}
  </li>
</ul>"#;

        let allocator = vize_carton::Bump::new();
        let (root, _) = vize_armature::parse(&allocator, template);

        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_script_setup(script);
        analyzer.analyze_template(&root);
        let summary = analyzer.finish();

        let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);

        assert_virtual_ts_snapshot("virtual_ts_vfor_destructuring_scope", output.code.as_str());
    }

    #[test]
    fn test_nested_vif_velse_chain() {
        use vize_croquis::{Analyzer, AnalyzerOptions};

        let script = r#"import { ref } from 'vue'
const status = ref('loading')
const message = ref('')
"#;
        let template = r#"<div>
  <div v-if="status === 'loading'">Loading</div>
  <div v-else-if="status === 'error'">{{ message }}</div>
  <div v-else>Done</div>
</div>"#;

        let allocator = vize_carton::Bump::new();
        let (root, _) = vize_armature::parse(&allocator, template);

        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_script_setup(script);
        analyzer.analyze_template(&root);
        let summary = analyzer.finish();

        let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);

        assert_virtual_ts_snapshot("virtual_ts_nested_vif_velse_chain", output.code.as_str());
    }

    #[test]
    fn test_v_else_if_chain_uses_linear_control_flow() {
        use vize_croquis::{Analyzer, AnalyzerOptions};

        let script = r#"type Log =
  | { type: 't0'; info: { value0: string } }
  | { type: 't1'; info: { value1: string } }
  | { type: 't2'; info: { value2: string } }

defineProps<{ log: Log }>()
"#;
        let template = r#"<div>
  <span v-if="log.type === 't0'">{{ log.info.value0 }}</span>
  <span v-else-if="log.type === 't1'">{{ log.info.value1 }}</span>
  <span v-else-if="log.type === 't2'">{{ log.info.value2 }}</span>
</div>"#;

        let allocator = vize_carton::Bump::new();
        let (root, _) = vize_armature::parse(&allocator, template);

        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_script_setup(script);
        analyzer.analyze_template(&root);
        let summary = analyzer.finish();

        let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);

        assert!(
            output.code.contains("if (log.type === 't0') {"),
            "expected first branch to use native control flow:\n{}",
            output.code
        );
        assert!(
            output.code.contains("} else if (log.type === 't1') {")
                && output.code.contains("} else if (log.type === 't2') {"),
            "expected else-if branches to use native control flow:\n{}",
            output.code
        );
        assert!(
            !output.code.contains("!(log.type === 't0') &&"),
            "virtual TS should not repeat cumulative negated branch guards:\n{}",
            output.code
        );
        assert!(
            !output.code.contains("void (log.type === 't1'); // VIf"),
            "branch conditions should not be emitted again inside the branch body:\n{}",
            output.code
        );
    }

    #[test]
    fn test_scoped_slot_expressions() {
        use vize_croquis::{Analyzer, AnalyzerOptions};

        let script = r#"import MyList from './MyList.vue'
const items = ['a', 'b']
"#;
        let template = r#"<MyList :items="items">
  <template #default="{ item }">
    {{ item }}
  </template>
</MyList>"#;

        let allocator = vize_carton::Bump::new();
        let (root, _) = vize_armature::parse(&allocator, template);

        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_script_setup(script);
        analyzer.analyze_template(&root);
        let summary = analyzer.finish();

        let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);

        assert_virtual_ts_snapshot("virtual_ts_scoped_slot_expressions", output.code.as_str());
    }

    #[test]
    fn test_v_if_narrows_nullable_binding() {
        // `<div v-if="user">{{ user.name }}</div>` must produce a virtual TS
        // closure that opens an `if (user) { … }` block so TypeScript narrows
        // `user` from `User | null` to `User` for the inner expression. See
        // #693. The snapshot captures the generated narrowing structure.
        use vize_croquis::{Analyzer, AnalyzerOptions};

        let script = r#"interface User { name: string }
const user: User | null = null as any
"#;
        let template = r#"<div v-if="user">
  <p>{{ user.name }}</p>
</div>"#;

        let allocator = vize_carton::Bump::new();
        let (root, _) = vize_armature::parse(&allocator, template);

        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_script_setup(script);
        analyzer.analyze_template(&root);
        let summary = analyzer.finish();

        let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);

        // The narrowing wrapper must appear in the generated TS so TS can
        // narrow `user` for the inner property access.
        assert!(
            output.code.contains("if ((user))"),
            "expected `if ((user))` narrowing wrapper in virtual TS, got:\n{}",
            output.code
        );
    }

    #[test]
    fn test_reserved_prop_and_hyphenated_slot_names() {
        use vize_croquis::{Analyzer, AnalyzerOptions};

        let script = r#"import TrendChart from './TrendChart.vue'
defineProps<{
  class?: string
}>()
"#;
        let template = r#"<TrendChart :class="class">
  <template #area-gradient="{ id }">
    {{ id }}
  </template>
</TrendChart>"#;

        let allocator = vize_carton::Bump::new();
        let (root, _) = vize_armature::parse(&allocator, template);

        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_script_setup(script);
        analyzer.analyze_template(&root);
        let summary = analyzer.finish();

        let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);

        let expression_start = template.find("\"class\"").unwrap() + 1;
        let expression_end = expression_start + "class".len();
        let mapping = output
            .mappings
            .iter()
            .find(|mapping| mapping.src_range == (expression_start..expression_end))
            .expect("should map the rewritten class prop expression");
        assert_eq!(&output.code[mapping.gen_range.clone()], "props[\"class\"]");

        assert_virtual_ts_snapshot(
            "virtual_ts_reserved_prop_and_hyphenated_slot_names",
            output.code.as_str(),
        );
    }

    #[test]
    fn test_multiple_event_handlers() {
        use vize_croquis::{Analyzer, AnalyzerOptions};

        let script = r#"import { ref } from 'vue'
const count = ref(0)
function handleClick() { count.value++ }
function handleHover() {}
"#;
        let template = r#"<div>
  <button @click="handleClick" @mouseenter="handleHover">{{ count }}</button>
</div>"#;

        let allocator = vize_carton::Bump::new();
        let (root, _) = vize_armature::parse(&allocator, template);

        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_script_setup(script);
        analyzer.analyze_template(&root);
        let summary = analyzer.finish();

        let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);

        assert_virtual_ts_snapshot("virtual_ts_multiple_event_handlers", output.code.as_str());
    }

    #[test]
    fn test_object_form_v_on_is_preserved_as_expression() {
        use vize_croquis::{Analyzer, AnalyzerOptions};

        let script = r#"const props = defineProps<{
  handlers?: {
    'update:modelValue'?: () => void
  }
}>()
"#;
        let template = r#"<button v-on="{ 'update:modelValue': props.handlers?.['update:modelValue'] }">Click</button>"#;

        let allocator = vize_carton::Bump::new();
        let (root, _) = vize_armature::parse(&allocator, template);

        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_script_setup(script);
        analyzer.analyze_template(&root);
        let summary = analyzer.finish();

        let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);

        assert!(
            output.code.contains(
                "void ({ 'update:modelValue': props.handlers?.['update:modelValue'] }); // VOn"
            ),
            "object-form v-on should be emitted as an expression:\n{}",
            output.code
        );
        assert!(
            !output.code.contains("@unknown handler"),
            "object-form v-on must not create a synthetic event handler:\n{}",
            output.code
        );
    }

    #[test]
    fn test_source_mappings_generated() {
        use vize_croquis::{Analyzer, AnalyzerOptions};

        let script = r#"import { ref } from 'vue'
const msg = ref('Hello')
"#;
        let template = r#"<div>{{ msg }}</div>"#;

        let allocator = vize_carton::Bump::new();
        let (root, _) = vize_armature::parse(&allocator, template);

        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_script_setup(script);
        analyzer.analyze_template(&root);
        let summary = analyzer.finish();

        let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);

        // Should have at least one mapping for the template expression
        assert!(
            !output.mappings.is_empty(),
            "Should generate source mappings for template expressions"
        );
        // All mappings should have valid ranges
        for mapping in &output.mappings {
            assert!(
                mapping.gen_range.start < mapping.gen_range.end,
                "Generated range should be non-empty"
            );
            assert!(
                mapping.src_range.start < mapping.src_range.end,
                "Source range should be non-empty"
            );
        }
    }

    #[test]
    fn test_source_mappings_target_expression_text() {
        use vize_croquis::{Analyzer, AnalyzerOptions};

        let script = r#"import { useTemplateRef } from 'vue'
const inputRef = useTemplateRef<HTMLInputElement>('input')
"#;
        let template = r#"<div :data-active="inputRef && inputRef.focus()"></div>"#;

        let allocator = vize_carton::Bump::new();
        let (root, _) = vize_armature::parse(&allocator, template);

        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_script_setup(script);
        analyzer.analyze_template(&root);
        let summary = analyzer.finish();

        let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);

        let expression = "inputRef && inputRef.focus()";
        let source_start = template.find(expression).unwrap();
        let source_end = source_start + expression.len();
        let mapping = output
            .mappings
            .iter()
            .find(|mapping| mapping.src_range == (source_start..source_end))
            .expect("should map the template expression");

        assert_eq!(&output.code[mapping.gen_range.clone()], expression);
    }

    #[test]
    fn test_template_shadow_bindings_only_unwrap_vue_refs() {
        use vize_croquis::{Analyzer, AnalyzerOptions};

        let script = r#"import { ref, useTemplateRef } from 'vue'
const users = ref([{ id: 1 }])
const inputRef = useTemplateRef<HTMLInputElement>('input')
"#;
        let template = r#"<div>{{ users.length }} {{ inputRef && inputRef.focus() }}</div>"#;

        let allocator = vize_carton::Bump::new();
        let (root, _) = vize_armature::parse(&allocator, template);

        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_script_setup(script);
        analyzer.analyze_template(&root);
        let summary = analyzer.finish();

        let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);

        assert_virtual_ts_snapshot("virtual_ts_template_binding_unwraps", output.code.as_str());
    }

    #[test]
    fn test_virtual_ts_generation_survives_unicode_script_comments() {
        use vize_croquis::{Analyzer, AnalyzerOptions};

        let script = r#"const reasgnSubMenuOpen = debounce(() => {
  console.log(1222222222222222222222222222222);
}, 100);

// あいうえおかきくけこさしすせそたちつてとなにぬねの
const heightLimit = "65vh";
// はひふへほまみむめもやいゆえよらりるれろわをん
"#;
        let template = r#"<div>{{ heightLimit }}</div>"#;

        let allocator = vize_carton::Bump::new();
        let (root, _) = vize_armature::parse(&allocator, template);

        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_script_setup(script);
        analyzer.analyze_template(&root);
        let summary = analyzer.finish();

        let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);

        assert!(output.code.contains("heightLimit"));
    }

    #[test]
    fn test_script_setup_generic_param_injected_into_hoisted_type() {
        // A type declared in `<script setup generic="T">` that references the
        // generic parameter is lifted to module scope; the generic must be
        // re-declared on it so `T` resolves there (a residual of the repro-8
        // hoisting fix). Bare uses like `Option[]` still resolve via `= any`.
        use vize_croquis::{Analyzer, AnalyzerOptions};

        let script = r#"type Option = { key: T; label: string }

defineProps<{
  options: Option[]
  current: T | undefined
}>()
"#;

        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_script_setup_with_generic(script, Some("T extends string"));
        let summary = analyzer.finish();

        let output = generate_virtual_ts_with_offsets(
            &summary,
            Some(script),
            None,
            0,
            0,
            &Default::default(),
        );

        let (module_scope, _setup_scope) = output
            .code
            .split_once("// ========== Setup Scope ==========")
            .expect("setup scope marker present");

        assert!(
            module_scope
                .contains("type Option<T extends string = any> = { key: T; label: string }"),
            "hoisted type should gain the SFC generic parameter so `T` resolves at module scope:\n{}",
            output.code
        );
    }

    #[test]
    fn test_script_setup_type_reexport_lifted_to_module_scope() {
        // `export type { X }` re-exports must be emitted at module top level,
        // not inside `__setup()` where `export` is a syntax error (TS1233).
        use vize_croquis::{Analyzer, AnalyzerOptions};

        let script = r#"import { type FilterType } from './ReExportType'

export type { FilterType }

defineProps<{ kind?: FilterType }>()
"#;

        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_script_setup(script);
        let summary = analyzer.finish();

        let output = generate_virtual_ts_with_offsets(
            &summary,
            Some(script),
            None,
            0,
            0,
            &Default::default(),
        );

        let (module_scope, setup_scope) = output
            .code
            .split_once("// ========== Setup Scope ==========")
            .expect("setup scope marker present");

        assert!(
            module_scope.contains("export type { FilterType }"),
            "re-export should be lifted to module scope:\n{}",
            output.code
        );
        assert!(
            !setup_scope.contains("export type { FilterType }"),
            "re-export must not be trapped inside __setup():\n{}",
            output.code
        );
    }

    #[test]
    fn test_vfor_component_props_in_scope() {
        // Component inside v-for should have prop checks inside the forEach closure
        use vize_croquis::{Analyzer, AnalyzerOptions};

        let script = r#"import { ref } from 'vue'
import TodoItem from './TodoItem.vue'

const todos = ref([{ id: 1, text: 'Hello' }])
"#;
        let template = r#"<div>
  <TodoItem v-for="todo in todos" :key="todo.id" :item="todo" />
</div>"#;

        let allocator = vize_carton::Bump::new();
        let (root, _) = vize_armature::parse(&allocator, template);

        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_script_setup(script);
        analyzer.analyze_template(&root);
        let summary = analyzer.finish();

        let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);

        assert_virtual_ts_snapshot(
            "virtual_ts_vfor_component_props_in_scope",
            output.code.as_str(),
        );
    }

    #[test]
    fn test_component_prop_checks_respect_same_element_vif_guard() {
        use vize_croquis::{Analyzer, AnalyzerOptions};

        let script = r#"import { ref } from 'vue'
import LinkComp from './LinkComp.vue'

const item = ref<{ name: string } | undefined>()
"#;
        let template = r#"<LinkComp v-if="item" :to="item.name" />"#;

        let allocator = vize_carton::Bump::new();
        let (root, _) = vize_armature::parse(&allocator, template);

        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_script_setup(script);
        analyzer.analyze_template(&root);
        let summary = analyzer.finish();

        let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);

        assert_virtual_ts_snapshot(
            "virtual_ts_component_prop_checks_respect_same_element_vif_guard",
            output.code.as_str(),
        );
    }
}
