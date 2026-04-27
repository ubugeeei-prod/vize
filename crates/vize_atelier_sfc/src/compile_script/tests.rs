//! Tests for script compilation.

#[cfg(test)]
mod compile_script_tests {
    use crate::compile_script::compile_script;
    use crate::compile_script::function_mode::compile_script_setup;
    use crate::compile_script::props::{
        extract_prop_types_from_type, extract_with_defaults_defaults, is_valid_identifier,
    };
    use crate::compile_script::typescript::transform_typescript_to_js;
    use crate::types::SfcDescriptor;
    use vize_carton::ToCompactString;

    #[test]
    fn test_compile_empty_script() {
        let descriptor = SfcDescriptor::default();
        let result =
            compile_script(&descriptor, &Default::default(), "Test", false, false).unwrap();
        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_is_valid_identifier() {
        assert!(is_valid_identifier("foo"));
        assert!(is_valid_identifier("_bar"));
        assert!(is_valid_identifier("$baz"));
        assert!(is_valid_identifier("foo123"));
        assert!(!is_valid_identifier("123foo"));
        assert!(!is_valid_identifier(""));
        assert!(!is_valid_identifier("foo-bar"));
    }

    #[test]
    fn test_extract_with_defaults_defaults() {
        // Test simple case
        let input = r#"withDefaults(defineProps<{ msg?: string }>(), { msg: "hello" })"#;
        let defaults = extract_with_defaults_defaults(input);
        eprintln!("Defaults: {:?}", defaults);
        assert_eq!(defaults.get("msg"), Some(&r#""hello""#.to_compact_string()));

        // Test multiple defaults
        let input2 = r#"withDefaults(defineProps<{ msg?: string, count?: number }>(), { msg: "hello", count: 42 })"#;
        let defaults2 = extract_with_defaults_defaults(input2);
        assert_eq!(
            defaults2.get("msg"),
            Some(&r#""hello""#.to_compact_string())
        );
        assert_eq!(defaults2.get("count"), Some(&"42".to_compact_string()));

        // Test multiline input like AfCheckbox
        let input3 = r#"withDefaults(
  defineProps<{
    checked: boolean;
    label?: string;
    color?: "primary" | "secondary";
  }>(),
  {
    label: undefined,
    color: "primary",
  },
)"#;
        let defaults3 = extract_with_defaults_defaults(input3);
        eprintln!("Defaults3: {:?}", defaults3);
        assert_eq!(
            defaults3.get("label"),
            Some(&"undefined".to_compact_string())
        );
        assert_eq!(
            defaults3.get("color"),
            Some(&r#""primary""#.to_compact_string())
        );

        // Strings containing commas/markdown markers must stay intact
        let input4 = r#"withDefaults(defineProps<{ description?: string }>(), { description: 'a fast, modern browser for the **npm registry**' })"#;
        let defaults4 = extract_with_defaults_defaults(input4);
        assert_eq!(
            defaults4.get("description"),
            Some(&"'a fast, modern browser for the **npm registry**'".to_compact_string())
        );
    }

    #[test]
    fn test_compile_script_setup_with_define_props() {
        let content = r#"
import { ref } from 'vue'
const props = defineProps(['msg'])
const count = ref(0)
"#;
        let result = compile_script_setup(content, "Test", false, false, None).unwrap();

        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_type_only_imports_not_in_bindings() {
        let content = r#"
import type { AnalysisResult } from './wasm'
import type { Diagnostic } from './MonacoEditor.vue'
import { ref } from 'vue'

const analysisResult = ref<AnalysisResult | null>(null)
"#;
        let result = compile_script_setup(content, "Test", false, true, None).unwrap();
        let bindings = result.bindings.expect("bindings should be present");

        assert!(!bindings.bindings.contains_key("AnalysisResult"));
        assert!(!bindings.bindings.contains_key("Diagnostic"));
        assert!(bindings.bindings.contains_key("analysisResult"));
    }

    #[test]
    fn test_import_used_only_in_ts_positions_not_returned() {
        // With template: type-only import should NOT be in __returned__
        let content = r#"
import { SomeType } from './types'

interface Props {
  items: SomeType[]
}

const props = defineProps<Props>()
"#;
        let result = compile_script_setup(
            content,
            "Test",
            false,
            true,
            Some("<div>{{ props.items.length }}</div>"),
        )
        .unwrap();

        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_no_template_preserves_all_imports_in_returned() {
        // Without template: all imports should be conservatively included in __returned__
        // to match @vue/compiler-sfc behavior
        let content = r#"
import { SomeType } from './types'

interface Props {
  items: SomeType[]
}

const props = defineProps<Props>()
"#;
        let result = compile_script_setup(content, "Test", false, true, None).unwrap();

        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_mixed_import_type_and_runtime_with_template() {
        // Mixed import: SomeType used only in type positions, someHelper used at runtime.
        // With template, only runtime-used and template-used imports should be in __returned__.
        let content = r#"
import { SomeType, someHelper } from './mod'

interface Props {
  items: SomeType[]
}

const props = defineProps<Props>()
const result = someHelper()
"#;
        let result = compile_script_setup(
            content,
            "Test",
            false,
            true,
            Some("<div>{{ result }}</div>"),
        )
        .unwrap();

        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_import_used_both_type_and_runtime() {
        // Same symbol used in both type annotation and runtime (e.g., new SomeClass()).
        // Should be kept in __returned__.
        let content = r#"
import { SomeClass } from './mod'

const instance: SomeClass = new SomeClass()
"#;
        let result = compile_script_setup(
            content,
            "Test",
            false,
            true,
            Some("<div>{{ instance }}</div>"),
        )
        .unwrap();

        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_default_import_type_only_with_template() {
        // Default import used only in type position, with template present.
        let content = r#"
import Foo from './foo'

interface Props {
  value: Foo
}

const props = defineProps<Props>()
"#;
        let result = compile_script_setup(
            content,
            "Test",
            false,
            true,
            Some("<div>{{ props.value }}</div>"),
        )
        .unwrap();

        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_import_used_in_template_included() {
        // Import not used in setup runtime code, but used in template.
        // Should be included in __returned__.
        let content = r#"
import { formatter } from './utils'

const today = new Date()
"#;
        let result = compile_script_setup(
            content,
            "Test",
            false,
            false,
            Some("<div>{{ formatter }}</div>"),
        )
        .unwrap();

        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_import_type_syntax_always_excluded() {
        // Explicit `import type` syntax should never be in __returned__,
        // regardless of template presence.
        let content = r#"
import type { MyType } from './types'
import { ref } from 'vue'

const value = ref<MyType | null>(null)
"#;
        // Without template
        let result = compile_script_setup(content, "Test", false, true, None).unwrap();

        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_compile_script_setup_with_define_emits() {
        let content = r#"
const emit = defineEmits(['click', 'update'])
"#;
        let result = compile_script_setup(content, "Test", false, false, None).unwrap();

        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_compile_script_setup_with_define_emits_usage() {
        let content = r#"
import { ref } from 'vue'
const emit = defineEmits(['click', 'update'])
const count = ref(0)
function onClick() {
    emit('click', count.value)
}
"#;
        let result = compile_script_setup(content, "Test", false, false, None).unwrap();

        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_compile_script_setup_without_macros() {
        let content = r#"
import { ref } from 'vue'
const msg = ref('hello')
"#;
        let result = compile_script_setup(content, "Test", false, false, None).unwrap();

        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_compile_script_setup_strips_ecosystem_compile_time_macro() {
        let content = r#"
definePage({
  name: 'home',
  meta: {
    requiresAuth: true,
  },
})

const msg = 'ready'
"#;
        let result =
            compile_script_setup(content, "Test", false, false, Some("<div>{{ msg }}</div>"))
                .unwrap();

        assert!(
            !result.code.contains("definePage"),
            "definePage should be removed from runtime output:\n{}",
            result.code
        );
        assert!(result.code.contains("ready"));
    }

    #[test]
    fn test_compile_script_setup_strips_define_page_meta() {
        let content = r#"
definePageMeta({
  name: 'docs',
  meta: {
    scrollMargin: 180,
  },
})

const msg = 'ready'
"#;
        let result =
            compile_script_setup(content, "Test", false, false, Some("<div>{{ msg }}</div>"))
                .unwrap();

        assert!(
            !result.code.contains("definePageMeta"),
            "definePageMeta should be removed from runtime output:\n{}",
            result.code
        );
        assert!(result.code.contains("ready"));
    }

    #[test]
    fn test_compile_script_setup_strips_define_route_rules() {
        let content = r#"
defineRouteRules({
  prerender: true,
  cache: {
    maxAge: 60,
  },
})

const msg = 'ready'
"#;
        let result =
            compile_script_setup(content, "Test", false, false, Some("<div>{{ msg }}</div>"))
                .unwrap();

        assert!(
            !result.code.contains("defineRouteRules"),
            "defineRouteRules should be removed from runtime output:\n{}",
            result.code
        );
        assert!(result.code.contains("ready"));
    }

    #[test]
    fn test_compile_script_setup_expands_define_lazy_hydration_component() {
        let content = r#"
const LazyHydrationMyComponent = defineLazyHydrationComponent(
  'idle',
  () => import('./components/MyComponent.vue'),
)
"#;
        let result = compile_script_setup(
            content,
            "Test",
            false,
            false,
            Some("<LazyHydrationMyComponent />"),
        )
        .unwrap();

        assert!(
            !result.code.contains("defineLazyHydrationComponent"),
            "defineLazyHydrationComponent should be expanded from runtime output:\n{}",
            result.code
        );
        assert!(result.code.contains("__vizeCreateLazyIdleComponent"));
        assert!(result.code.contains("./components/MyComponent.vue"));
    }

    #[test]
    fn test_compile_script_setup_with_props_destructure() {
        let content = r#"
import { computed } from 'vue'
const { count } = defineProps({ count: Number })
const double = computed(() => count * 2)
"#;
        let result = compile_script_setup(content, "Test", false, false, None).unwrap();

        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_compiler_macros_not_in_returned() {
        let content = r#"
import { defineProps, ref } from 'vue'
const props = defineProps(['msg'])
const count = ref(0)
"#;
        let result = compile_script_setup(content, "Test", false, false, None).unwrap();

        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_props_destructure_with_defaults() {
        let content = r#"
import { computed, watch } from 'vue'

const {
  name,
  count = 0,
  disabled = false,
  items = () => []
} = defineProps<{
  name: string
  count?: number
  disabled?: boolean
  items?: string[]
}>()

const doubled = computed(() => count * 2)
const itemCount = computed(() => items.length)
"#;

        // First check context analysis
        let mut ctx = crate::script::ScriptCompileContext::new(content);
        ctx.analyze();

        let result = compile_script_setup(content, "Test", false, false, None).unwrap();

        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_extract_prop_types() {
        let type_args = r#"{
  name: string
  count?: number
  disabled?: boolean
  items?: string[]
}"#;
        let props = extract_prop_types_from_type(type_args);
        let find = |name: &str| props.iter().find(|(n, _)| n == name).map(|(_, v)| v);
        assert!(find("name").is_some(), "Should extract name");
        assert!(find("count").is_some(), "Should extract count");
        assert!(find("disabled").is_some(), "Should extract disabled");
        assert!(find("items").is_some(), "Should extract items");

        // Check types
        assert_eq!(find("name").unwrap().js_type, "String");
        assert_eq!(find("count").unwrap().js_type, "Number");
        assert_eq!(find("disabled").unwrap().js_type, "Boolean");
        assert_eq!(find("items").unwrap().js_type, "Array");

        // Check optionality
        assert!(!find("name").unwrap().optional);
        assert!(find("count").unwrap().optional);
        assert!(find("disabled").unwrap().optional);
        assert!(find("items").unwrap().optional);
    }

    #[test]
    fn test_compile_script_setup_with_multiline_define_emits() {
        let content = r#"
const emit = defineEmits<{
  (e: 'click', payload: MouseEvent): void
  (e: 'update', value: string): void
}>()

function handleClick(e: MouseEvent) {
    emit('click', e)
}
"#;
        let result = compile_script_setup(content, "Test", false, false, None).unwrap();

        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_compile_script_setup_with_typed_define_emits_single_line() {
        let content = r#"
const emit = defineEmits<(e: 'click') => void>()
"#;
        let result = compile_script_setup(content, "Test", false, false, None).unwrap();

        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_compile_script_setup_with_next_line_define_props_assignment() {
        let content = r#"
import { computed } from 'vue'

interface Props {
  name: string
}

const props =
  defineProps<Props>()

const greeting = computed(() => props.name)
"#;
        let result = compile_script_setup(content, "Test", false, false, None).unwrap();

        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_compile_script_setup_with_next_line_define_slots_assignment() {
        let content = r#"
const slots =
  defineSlots<{
    default?: () => string
  }>()

const hasDefault = !!slots.default
"#;
        let result = compile_script_setup(content, "Test", false, false, None).unwrap();

        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_compile_script_setup_with_define_expose() {
        let content = r#"
import { ref } from 'vue'
const count = ref(0)
const reset = () => count.value = 0
defineExpose({ count, reset })
"#;
        let result = compile_script_setup(content, "Test", false, false, None).unwrap();

        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_compile_script_setup_without_define_expose() {
        // Test that __expose() is always called, even without defineExpose.
        // This matches the official Vue compiler behavior, which is required for
        // proper component initialization with @vue/test-utils.
        let content = r#"
import { ref } from 'vue'
const count = ref(0)
"#;
        let result = compile_script_setup(content, "Test", false, false, None).unwrap();

        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_compile_script_setup_with_empty_define_expose() {
        // Test that defineExpose() (empty) is handled correctly
        let content = r#"
import { ref } from 'vue'
const count = ref(0)
defineExpose()
"#;
        let result = compile_script_setup(content, "Test", false, false, None).unwrap();

        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_transform_typescript_to_js_strips_types() {
        let ts_code = r#"const getNumber = (x: number): string => {
    return x.toString();
}
const foo: string = "bar";"#;
        let result = transform_typescript_to_js(ts_code);
        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_compile_script_setup_strips_typescript() {
        let content = r#"
const getNumberOfTeachers = (
  items: Item[]
): string => {
  return items.length.toString();
};
"#;
        // is_ts = false means we want JavaScript output (TypeScript should be stripped)
        let result = compile_script_setup(content, "Test", false, false, None).unwrap();
        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_compile_script_setup_preserves_typescript_when_is_ts() {
        let content = r#"
const count: number = 1;
const items: Array<string> = [];
"#;
        let result = compile_script_setup(content, "Test", false, true, None).unwrap();
        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_props_destructure_type_based_defaults() {
        let content = r#"
const { color = "primary" } = defineProps<{
  color?: "primary" | "secondary";
        }>();
"#;
        let result = compile_script_setup(content, "Test", false, false, None).unwrap();
        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_duplicate_imports_filtered() {
        let content = r#"
import { ref } from 'vue'
        import { ref } from 'vue'
const count = ref(0)
"#;
        let result = compile_script_setup(content, "Test", false, false, None).unwrap();
        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_async_setup_detection() {
        let content = r#"
const response = await fetch('/api/data')
        const data = await response.json()
"#;
        let result = compile_script_setup(content, "Test", false, false, None).unwrap();
        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_await_in_string_literal_does_not_trigger_async() {
        let content = r#"
const msg = "await should not trigger async"
"#;
        let result = compile_script_setup(content, "Test", false, false, None).unwrap();
        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_type_comparison_not_stripped() {
        let content = r#"
const props = defineProps(['type'])
const isButton = props.type === 'button'
"#;
        let result = compile_script_setup(content, "Test", false, false, None).unwrap();
        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_generic_function_call_stripped() {
        let content = r#"
const store = useStore<RootState>()
"#;
        let result = compile_script_setup(content, "Test", false, false, None).unwrap();
        insta::assert_snapshot!(result.code.as_str());
    }
}
