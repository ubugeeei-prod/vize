use super::super::compile_sfc;
use crate::types::{SfcCompileOptions, TemplateCompileOptions};
use crate::{SfcParseOptions, parse_sfc};

#[test]
fn test_options_api_method_handlers_use_live_instance_lookup() {
    let source = r#"<script>
export default {
  methods: {
    onFocus(event) {
      this.$emit("focus", event)
    },
    onBlur(event) {
      this.$emit("blur", event)
    }
  }
}
</script>

<template>
  <input id="test" @focus="onFocus" @blur.stop="onBlur" />
</template>"#;

    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("Failed to parse SFC");
    let result =
        compile_sfc(&descriptor, SfcCompileOptions::default()).expect("Failed to compile SFC");

    assert!(
        result
            .code
            .contains("onFocus: (...args) => (_ctx.onFocus && _ctx.onFocus(...args))"),
        "Options API method refs must read from the public instance proxy at event time:\n{}",
        result.code
    );
    assert!(
        result
            .code
            .contains("_withModifiers((...args) => (_ctx.onBlur && _ctx.onBlur(...args))"),
        "Options API method refs wrapped with modifiers must still use live instance lookup:\n{}",
        result.code
    );
    assert!(
        !result.code.contains("$options.onFocus"),
        "Options API v-on refs must not capture the original $options method:\n{}",
        result.code
    );
}

#[test]
fn test_cached_options_api_method_handlers_keep_live_instance_lookup() {
    let source = r#"<script>
export default {
  methods: {
    onClick(event) {
      this.$emit("click", event)
    }
  }
}
</script>

<template>
  <button @click="onClick">click</button>
</template>"#;

    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("Failed to parse SFC");
    let mut compiler_options = vize_atelier_dom::DomCompilerOptions::default();
    compiler_options.cache_handlers = true;
    let result = compile_sfc(
        &descriptor,
        SfcCompileOptions {
            template: TemplateCompileOptions {
                compiler_options: Some(compiler_options),
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .expect("Failed to compile SFC");

    assert!(
        result.code.contains(
            "onClick: _cache[0] || (_cache[0] = (...args) => (_ctx.onClick && _ctx.onClick(...args)))",
        ),
        "Cached Options API handlers must keep a stable wrapper that performs a live lookup:\n{}",
        result.code
    );
    assert!(
        !result.code.contains("$options.onClick"),
        "Cached Options API v-on refs must not capture the original $options method:\n{}",
        result.code
    );
}
