use vize_atelier_sfc::{SfcCompileOptions, SfcParseOptions, compile_sfc, parse_sfc};

#[test]
fn module_mode_keeps_runtime_types_for_indexed_access_props() {
    let source = r#"<script setup lang="ts">
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

<template><div /></template>"#;

    let descriptor = parse_sfc(
        source,
        SfcParseOptions {
            filename: "IndexedAccessProps.vue".into(),
            ..Default::default()
        },
    )
    .expect("parse SFC");
    let result = compile_sfc(&descriptor, SfcCompileOptions::default()).expect("compile SFC");
    let code = result.code;
    assert_eq!(
        code.as_str(),
        r#"import { openBlock as _openBlock, createElementBlock as _createElementBlock } from "vue";
export default {
  __name: "anonymous",
  props: {
    showOverflowTooltip: {
      type: [Boolean, Object],
      required: false,
      default: undefined
    },
    tooltipFormatter: {
      type: Function,
      required: false
    }
  },
  setup(__props) {
    return (_ctx, _cache) => {
      return _openBlock(), _createElementBlock("div");
    };
  }
};"#
    );
}
