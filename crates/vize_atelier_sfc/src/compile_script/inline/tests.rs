//! Tests for inline script compilation.

#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use super::super::compiler::compile_script_setup_inline;
    use crate::compile_script::TemplateParts;
    use vize_carton::String;

    /// Helper to compile a minimal script setup and return the output code
    fn compile_setup(script_content: &str) -> String {
        let empty_template = TemplateParts {
            imports: "",
            hoisted: "",
            preamble: "",
            render_body: "null",
        };
        let result = compile_script_setup_inline(
            script_content,
            "TestComponent",
            false, // is_ts = false (JS output, strip TS)
            true,  // source_is_ts = true
            empty_template,
            None,
            &[], // no css_vars
            "",  // no scope_id
            None,
        )
        .expect("compilation should succeed");
        result.code
    }

    /// Helper to compile with is_ts=true (TypeScript output)
    fn compile_setup_ts(script_content: &str) -> String {
        let empty_template = TemplateParts {
            imports: "",
            hoisted: "",
            preamble: "",
            render_body: "null",
        };
        let result = compile_script_setup_inline(
            script_content,
            "TestComponent",
            true, // is_ts = true (TS output)
            true, // source_is_ts = true
            empty_template,
            None,
            &[], // no css_vars
            "",  // no scope_id
            None,
        )
        .expect("compilation should succeed");
        result.code
    }

    #[test]
    fn test_declare_global_not_in_setup_body_ts() {
        let content = r#"
import { ref } from 'vue'

const handleClick = () => {
  console.log('click')
}

declare global {
  interface Window {
    EyeDropper: any
  }
}

const x = ref(0)
"#;
        let output = compile_setup_ts(content);
        let setup_start = output.find("setup(").expect("should have setup function");
        let setup_body = &output[setup_start..];
        assert!(
            !setup_body.contains("declare global"),
            "declare global should NOT be inside setup function body. Got:\n{}",
            output
        );
    }

    #[test]
    fn test_export_type_reexport_stripped() {
        let content = r#"
import { ref } from 'vue'
import type { FilterType } from './types'

export type { FilterType }

const x = ref(0)
"#;
        let output = compile_setup(content);
        let setup_start = output.find("setup(").expect("should have setup");
        let setup_body = &output[setup_start..];
        assert!(
            !setup_body.contains("export type"),
            "export type re-export should not be inside setup body. Got:\n{}",
            output
        );
    }

    #[test]
    fn test_type_as_variable_at_line_start() {
        let content = r#"
import { ref } from 'vue'

const type = ref('material-symbols')
const identifier =
  type === 'material-symbols' ? 'name' : 'ligature'
"#;
        let output = compile_setup(content);
        assert!(
            output.contains("type ==="),
            "`type ===` continuation line should be preserved. Got:\n{}",
            output
        );
    }

    #[test]
    fn test_destructure_with_multiline_function_call() {
        let content = r#"
import { ref, toRef } from 'vue'
import { useSomething } from './useSomething'

const fileInputRef = ref()

const {
  handleSelect,
  handleChange,
} = useSomething(
  fileInputRef,
  {
    onError: (e) => console.log(e),
    onSuccess: () => console.log('ok'),
  },
  toRef(() => 'test'),
)

const other = ref(1)
"#;
        let output = compile_setup(content);
        // Function call args should be part of the destructure statement, not bare statements
        assert!(
            output.contains("useSomething("),
            "Function call should be present in destructure statement. Got:\n{}",
            output
        );
        assert!(
            output.contains("const other = ref(1)"),
            "Code after destructure should be present. Got:\n{}",
            output
        );
    }

    #[test]
    fn test_side_effect_import_without_semicolons() {
        let content = r#"
import { watch } from 'vue'
import '@/css/oldReset.scss'

const { dialogRef } = provideDialog()

watch(
  dialogRef,
  (val) => {
    console.log(val)
  },
  { immediate: true },
)
"#;
        let output = compile_setup_ts(content);
        assert!(
            output.contains("watch("),
            "watch() call should be in setup body. Got:\n{}",
            output
        );
    }

    #[test]
    fn test_multiline_standalone_await_preserves_object_literal() {
        let content = r#"
const client = useClient()

await client.reports.create({
  accountId: 'acc',
  message: 'hello',
})
"#;
        let output = compile_setup_ts(content);
        assert!(
            output.contains("_withAsyncContext(() => client.reports.create({"),
            "multiline await should keep the full call expression. Got:\n{}",
            output
        );
        assert!(
            output.contains("accountId: 'acc'") && output.contains("message: 'hello'"),
            "object literal fields should remain intact. Got:\n{}",
            output
        );
        assert!(
            !output.contains("create({))"),
            "await transform must not truncate the object literal. Got:\n{}",
            output
        );
    }

    #[test]
    fn test_multiline_await_assignment_preserves_initializer() {
        let content = r#"
const response = await fetch('/api/report', {
  method: 'POST',
  body: JSON.stringify({ ok: true }),
})
"#;
        let output = compile_setup_ts(content);
        assert!(
            output.contains("const response =")
                && output.contains("_withAsyncContext(() => fetch('/api/report', {"),
            "await assignment should wrap the whole initializer. Got:\n{}",
            output
        );
        assert!(
            output.contains("method: 'POST'")
                && output.contains("body: JSON.stringify({ ok: true })"),
            "initializer object literal should remain intact. Got:\n{}",
            output
        );
        assert!(
            !output.contains("fetch('/api/report', {))"),
            "await assignment must not truncate multiline initializers. Got:\n{}",
            output
        );
    }

    #[test]
    fn test_export_type_with_arrow_function_member() {
        let content = r#"
import { computed } from 'vue'
import { useRoute } from 'vue-router'

export type MenuSelectorOption = {
  label: string
  onClick: () => void
}

const route = useRoute()
const heading = computed(() => route.name)
"#;
        let output = compile_setup_ts(content);
        assert!(
            output.contains("export type MenuSelectorOption"),
            "export type should be at module level. Got:\n{}",
            output
        );
        let setup_start = output.find("setup(").expect("should have setup");
        let setup_body = &output[setup_start..];
        assert!(
            setup_body.contains("const route = useRoute()"),
            "const route should be inside setup body. Got:\n{}",
            output
        );
    }

    #[test]
    fn test_define_props_with_trailing_semicolon() {
        // Semicolons at end of defineProps() should not prevent macro detection
        let content = r#"
import { ref } from 'vue'

interface Props {
    msg: string
}

const { msg } = defineProps<Props>();
const count = ref(0)
"#;
        let output = compile_setup(content);
        assert!(
            output.contains("props: {"),
            "should generate props declaration even with trailing semicolon. Got:\n{}",
            output
        );
        assert!(
            output.contains("msg:"),
            "should include msg prop. Got:\n{}",
            output
        );
        assert!(
            output.contains("const count = ref(0)"),
            "code after defineProps should be present. Got:\n{}",
            output
        );
    }

    #[test]
    fn test_multiline_define_props_with_trailing_semicolon() {
        // Multi-line defineProps with trailing semicolon on closing line
        let content = r#"
import { ref } from 'vue'

const { label, disabled } = defineProps<{
    label: string
    disabled?: boolean
}>();
const x = ref(1)
"#;
        let output = compile_setup(content);
        assert!(
            output.contains("props: {"),
            "should generate props declaration for multiline defineProps with semicolon. Got:\n{}",
            output
        );
        assert!(
            output.contains("const x = ref(1)"),
            "code after defineProps should be present. Got:\n{}",
            output
        );
    }

    #[test]
    fn test_with_defaults_trailing_semicolon() {
        // withDefaults with trailing semicolon
        let content = r#"
import { ref } from 'vue'

interface Props {
    msg: string
    count?: number
}

const { msg, count } = withDefaults(defineProps<Props>(), {
    count: 0,
});
const x = ref(1)
"#;
        let output = compile_setup(content);
        assert!(
            output.contains("props: {"),
            "should generate props declaration for withDefaults with semicolon. Got:\n{}",
            output
        );
        assert!(
            output.contains("const x = ref(1)"),
            "code after withDefaults should be present. Got:\n{}",
            output
        );
    }

    /// Helper to compile with no template (empty render_body)
    fn compile_setup_no_template(script_content: &str) -> String {
        let empty_template = TemplateParts {
            imports: "",
            hoisted: "",
            preamble: "",
            render_body: "",
        };
        let result = compile_script_setup_inline(
            script_content,
            "TestComponent",
            false,
            true,
            empty_template,
            None,
            &[], // no css_vars
            "",  // no scope_id
            None,
        )
        .expect("compilation should succeed");
        result.code
    }

    #[test]
    fn test_no_template_returns_setup_bindings() {
        // When there's no template, setup bindings should be returned as an object
        let content = r#"
import { ref, computed } from 'vue'

const count = ref(0)
const doubled = computed(() => count.value * 2)
"#;
        let output = compile_setup_no_template(content);
        assert!(
            output.contains("return {"),
            "no-template case should return setup bindings. Got:\n{}",
            output
        );
        assert!(
            output.contains("count"),
            "should return count binding. Got:\n{}",
            output
        );
        assert!(
            output.contains("doubled"),
            "should return doubled binding. Got:\n{}",
            output
        );
    }

    #[test]
    fn test_no_template_returns_imported_bindings() {
        // Imported bindings should also be returned for runtime template compilation
        let content = r#"
import { onMounted } from 'vue'

onMounted(() => {
    console.log('mounted')
})
"#;
        let output = compile_setup_no_template(content);
        assert!(
            output.contains("return {") && output.contains("onMounted"),
            "no-template case should return imported bindings too (for runtime template access). Got:\n{}",
            output
        );
    }

    #[test]
    fn test_export_type_generates_props_declaration() {
        let content = r#"
export type MenuItemProps = {
    id: string
    label: string
    routeName: string
    disabled?: boolean
}
const { label, disabled, routeName } = defineProps<MenuItemProps>()
"#;
        let output = compile_setup(content);
        assert!(
            output.contains("props: {"),
            "should generate props declaration for export type. Got:\n{}",
            output
        );
        assert!(
            output.contains("label:") && output.contains("String"),
            "should include label prop. Got:\n{}",
            output
        );
        assert!(
            output.contains("routeName:") && output.contains("String"),
            "should include routeName prop. Got:\n{}",
            output
        );
        assert!(
            output.contains("disabled:"),
            "should include disabled prop. Got:\n{}",
            output
        );
    }

    #[test]
    fn test_define_props_destructure_value_on_next_line() {
        // Pattern: const { ... } =\n  defineProps<...>()
        // The destructure pattern is complete on line 1, but defineProps is on line 2.
        let content = r#"
import { computed } from 'vue'

interface TimetableCell {
    type: string
    title: string
    startTime: string
}

const { type, title, startTime } =
  defineProps<TimetableCell>();
const accentColor = computed(() => type === 'event' ? 'primary' : 'secondary')
"#;
        let output = compile_setup(content);
        assert!(
            output.contains("props: {"),
            "should generate props declaration for next-line defineProps. Got:\n{}",
            output
        );
        assert!(
            output.contains("type:") && output.contains("String"),
            "should include type prop. Got:\n{}",
            output
        );
        assert!(
            output.contains("title:") && output.contains("String"),
            "should include title prop. Got:\n{}",
            output
        );
        // Verify props destructure references are transformed correctly in setup body
        let setup_start = output.find("setup(").expect("should have setup");
        let setup_body = &output[setup_start..];
        assert!(
            setup_body.contains("__props.type"),
            "destructured prop 'type' should be rewritten to __props.type in setup body. Got:\n{}",
            output
        );
        assert!(
            !setup_body.contains("const { __props."),
            "destructure declaration should NOT appear in setup body. Got:\n{}",
            output
        );
    }

    #[test]
    fn test_define_props_destructure_value_on_next_line_with_semicolon() {
        // Same pattern with trailing semicolon
        let content = r#"
import { ref } from 'vue'

interface Props {
    msg: string
    count: number
}

const { msg, count } =
  defineProps<Props>();
const doubled = ref(count * 2)
"#;
        let output = compile_setup(content);
        assert!(
            output.contains("props: {"),
            "should generate props declaration. Got:\n{}",
            output
        );
        assert!(
            output.contains("msg:") && output.contains("String"),
            "should include msg prop. Got:\n{}",
            output
        );
    }

    #[test]
    fn test_multiline_conditional_type_ts() {
        // Multi-line conditional type with ? and : continuation markers
        let content = r#"
import { computed } from 'vue'

type KeyOfUnion<T> = T extends T ? keyof T : never;
type DistributiveOmit<T, K extends KeyOfUnion<T>> = T extends T
	? Omit<T, K>
	: never;

const x = computed(() => 1)
"#;
        let output = compile_setup_ts(content);
        // The full conditional type should be at module level, not in setup body
        assert!(
            output.contains("type DistributiveOmit"),
            "Conditional type should be in output. Got:\n{}",
            output
        );
        assert!(
            output.contains("? Omit<T, K>"),
            "Conditional type true branch should be in type declaration. Got:\n{}",
            output
        );
        assert!(
            output.contains(": never;"),
            "Conditional type false branch should be in type declaration. Got:\n{}",
            output
        );
        let setup_start = output.find("setup(").expect("should have setup");
        let setup_body = &output[setup_start..];
        assert!(
            !setup_body.contains("? Omit<T, K>"),
            "Conditional type branches should NOT be in setup body. Got:\n{}",
            output
        );
        assert!(
            setup_body.contains("const x = computed(() => 1)"),
            "Code after type should be in setup body. Got:\n{}",
            output
        );
    }

    #[test]
    fn test_non_props_destructure_value_on_next_line() {
        // Ensure regular (non-defineProps) destructures with value on next line
        // still work correctly
        let content = r#"
import { ref, toRefs } from 'vue'

const state = ref({ x: 1, y: 2 })
const { x, y } =
  toRefs(state.value)
const sum = ref(x.value + y.value)
"#;
        let output = compile_setup(content);
        assert!(
            output.contains("toRefs("),
            "non-props destructure should be preserved in setup body. Got:\n{}",
            output
        );
        assert!(
            output.contains("const sum = ref("),
            "code after destructure should be present. Got:\n{}",
            output
        );
    }
}
