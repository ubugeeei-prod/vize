use vize_atelier_sfc::{SfcCompileOptions, SfcParseOptions, compile_sfc, parse_sfc};
use vize_carton::String;

#[test]
fn module_mode_keeps_runtime_types_for_indexed_access_props() {
    let source = r#"
<script setup lang="ts">
interface TableColumnCtx<T> {
  showOverflowTooltip?: boolean | TableOverflowTooltipOptions
  tooltipFormatter?: TableOverflowTooltipFormatter<T>
}
interface TableColumnProps<T> {
  showOverflowTooltip?: TableColumnCtx<T>['showOverflowTooltip']
  tooltipFormatter?: TableColumnCtx<T>['tooltipFormatter']
}
type TableOverflowTooltipOptions = Partial<Omit<UseTooltipProps, 'content'>>
type TableOverflowTooltipFormatter<T> = (data: { row: T }) => string

withDefaults(defineProps<TableColumnProps<Row>>(), {
  showOverflowTooltip: undefined,
})
</script>
<template><div /></template>
"#;
    let descriptor = parse_sfc(
        source,
        SfcParseOptions {
            filename: "IndexedAccessProps.vue".into(),
            ..SfcParseOptions::default()
        },
    )
    .expect("parse SFC");
    let result = compile_sfc(&descriptor, SfcCompileOptions::default()).expect("compile SFC");
    let code = result.code;
    let normalized = code.split_whitespace().collect::<Vec<_>>().join(" ");

    assert_eq!(
        prop_entry(&normalized, "showOverflowTooltip"),
        "showOverflowTooltip: { type: [Boolean, Object], required: false, default: undefined }",
        "expected indexed access to preserve Boolean/Object runtime type, got:\n{code}",
    );
    assert_eq!(
        prop_entry(&normalized, "tooltipFormatter"),
        "tooltipFormatter: { type: Function, required: false }",
        "expected indexed access to preserve Function runtime type, got:\n{code}",
    );
}

fn prop_entry(code: &str, name: &str) -> String {
    let mut prefix = String::from(name);
    prefix.push_str(": ");
    let start = code.find(prefix.as_str()).expect("prop entry should exist");
    let entry = &code[start..];
    let mut depth = 0i32;
    let mut saw_value = false;
    for (idx, ch) in entry[prefix.len()..].char_indices() {
        match ch {
            '{' | '[' | '(' => {
                saw_value = true;
                depth += 1;
            }
            '}' | ']' | ')' => {
                depth -= 1;
                if saw_value && depth == 0 {
                    return String::from(&entry[..prefix.len() + idx + ch.len_utf8()]);
                }
            }
            _ => {}
        }
    }
    panic!("prop entry should have a complete value: {name}");
}
