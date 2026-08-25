use super::NoExportInScriptSetup;
use crate::rules::script::ScriptLinter;

fn create_linter() -> ScriptLinter {
    let mut linter = ScriptLinter::new();
    linter.add_rule(Box::new(NoExportInScriptSetup));
    linter
}

// --- Invalid: runtime exports inside a recognizable <script setup> ---

#[test]
fn test_invalid_named_export_with_macro() {
    let source = r#"
const props = defineProps<{ count: number }>()
export const helper = () => props.count
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
    insta::with_settings!({ snapshot_path => "../snapshots" }, {
        insta::assert_debug_snapshot!(result.diagnostics);
    });
}

#[test]
fn test_invalid_default_export_with_macro() {
    let source = r#"
defineProps<{ count: number }>()
export default {}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_export_all_with_macro() {
    let source = r#"
defineEmits(['change'])
export * from './helpers'
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_export_named_specifier_with_macro() {
    let source = r#"
defineExpose({})
const a = 1
export { a }
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_multiple_exports_all_reported() {
    let source = r#"
const props = defineProps<{ count: number }>()
export const a = 1
export function b() {}
export default {}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 3);
}

#[test]
fn test_invalid_export_detected_via_top_level_await() {
    // No compiler macro, but a top-level await proves this is <script setup>.
    let source = r#"
const data = await fetch('/api')
export const cached = data
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_export_enum_with_macro() {
    // Enums produce a runtime object; they are not erased like type aliases.
    let source = r#"
defineProps<{ count: number }>()
export enum Color { Red, Green }
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_mixed_type_and_value_specifiers() {
    // One runtime specifier makes the whole export a runtime export.
    let source = r#"
defineProps<{ count: number }>()
type Foo = { a: number }
const b = 1
export { type Foo, b }
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_empty_export_braces() {
    // `export {}` is a runtime module marker, not a type-only export.
    let source = r#"
defineProps<{ count: number }>()
export {}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_export_namespace_with_macro() {
    // A non-ambient namespace emits a runtime object, so it must be flagged.
    let source = r#"
defineProps<{ count: number }>()
export namespace Config { export const value = 1 }
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

// --- Valid: type-only exports are erased and stay legal in <script setup> ---

#[test]
fn test_valid_export_type_alias() {
    // Regression for #3208: `export type` is erased at compile time and
    // `@vue/compiler-sfc` accepts it, so the rule must not flag it.
    let source = r#"
export type Foo = { value: string }
defineProps<{ foo: Foo }>()
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_export_interface() {
    let source = r#"
export interface Props { count: number }
defineProps<Props>()
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_export_type_specifier_list() {
    let source = r#"
defineProps<{ count: number }>()
type Foo = { a: number }
export type { Foo }
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_export_inline_type_specifiers() {
    let source = r#"
defineProps<{ count: number }>()
type Foo = { a: number }
type Bar = { b: string }
export { type Foo, type Bar }
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_export_type_star() {
    let source = r#"
defineProps<{ count: number }>()
export type * from './types'
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_export_ambient_declare() {
    // Ambient declarations live in type space and are erased.
    let source = r#"
defineProps<{ count: number }>()
export declare const version: string
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_export_declare_namespace() {
    // An ambient namespace is erased at compile time; only non-ambient
    // `export namespace` emits a runtime object.
    let source = r#"
defineProps<{ count: number }>()
export declare namespace Config { const value: number }
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

// --- Valid: normal <script> (no setup markers) is never flagged ---

#[test]
fn test_valid_export_default_in_normal_script() {
    // A normal <script> exports its component options; not a <script setup>.
    let source = r#"
export default {
  name: 'MyComponent',
  data() {
    return { count: 0 }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_named_export_in_normal_script() {
    let source = r#"
export const API_URL = 'https://example.com'
export function helper() {
  return 1
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_export_all_in_normal_script() {
    let source = "export * from './helpers'\n";
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

// --- Valid: <script setup> without any export ---

#[test]
fn test_valid_script_setup_no_export() {
    let source = r#"
import { ref } from 'vue'
const props = defineProps<{ count: number }>()
const doubled = ref(props.count * 2)
defineExpose({ doubled })
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_import_is_not_export() {
    // Imports are allowed in <script setup>; only exports are flagged.
    let source = r#"
import Foo from './Foo.vue'
import { bar } from './bar'
const props = defineProps<{ count: number }>()
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_macro_substring_in_string_no_export() {
    // The macro byte-prefilter may trip on a string literal, but with no
    // actual export there is nothing to report.
    let source = r#"
const label = 'defineProps demo'
const x = 1
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_nested_export_keyword_not_top_level() {
    // `export` only inside a string is not a real export statement.
    let source = r#"
defineProps<{ count: number }>()
const code = 'export default {}'
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}
