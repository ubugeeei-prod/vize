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
mod props;
mod scope;
mod types;

pub use generator::{generate_virtual_ts, generate_virtual_ts_with_offsets};
pub use types::{TemplateGlobal, VirtualTsOptions, VirtualTsOutput, VizeMapping};

#[cfg(test)]
mod tests {
    use super::helpers::{
        generate_template_context, get_dom_event_type, VUE_SETUP_COMPILER_MACROS,
    };
    use super::{
        generate_virtual_ts, generate_virtual_ts_with_offsets, TemplateGlobal, VirtualTsOptions,
    };

    fn assert_virtual_ts_snapshot(name: &str, value: &str) {
        insta::with_settings!({
            snapshot_path => "../snapshots"
        }, {
            insta::assert_snapshot!(name, value);
        });
    }

    #[test]
    fn test_vue_setup_compiler_macros_are_actual_functions() {
        assert_virtual_ts_snapshot(
            "virtual_ts_vue_setup_compiler_macros",
            VUE_SETUP_COMPILER_MACROS,
        );
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
