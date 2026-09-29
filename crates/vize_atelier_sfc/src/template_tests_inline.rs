use super::{
    CodegenOptions, CustomElementMatcher, SfcCompileOptions, SfcScriptOutputMode,
    TemplateCompileOptions, TemplateSyntaxMode, compile_sfc, compile_sfc_for_adapter, parse_sfc,
};

#[test]
fn production_inline_render_stays_an_es_module_without_setup_proxy() {
    let source = r#"<script setup lang="ts">
const { title } = defineProps<{ title: string }>()
function open(id: string) { return id }
</script>
<template>
  <div class="card" @click="open(title)">{{ title }}</div>
</template>
"#;
    let descriptor = parse_sfc(source, Default::default()).unwrap();
    let result = compile_sfc_for_adapter(
        &descriptor,
        SfcCompileOptions {
            template: TemplateCompileOptions {
                is_prod: true,
                ..Default::default()
            },
            ..Default::default()
        },
        TemplateSyntaxMode::Standard,
        CustomElementMatcher::default(),
        CodegenOptions::default(),
        SfcScriptOutputMode::InlineTemplate,
    )
    .unwrap();
    let code = result.code.as_str();
    assert!(
        code.contains("export default"),
        "production inline output stays an ES module:\n{code}"
    );
    assert!(
        !code.contains("$setup"),
        "inlined render must not read bindings through $setup:\n{code}"
    );
    assert!(
        !code.contains("function _sfc_render"),
        "inlined render must not keep a separate render function:\n{code}"
    );
    assert!(
        !code.contains("type: String"),
        "production output drops the string prop runtime type:\n{code}"
    );
    assert!(
        !code.contains("= Vue"),
        "production inline output must not rewrite the module onto a Vue global:\n{code}"
    );
}

pub(super) fn assert_separate_template_local_directive_output(source: &str) {
    let descriptor = parse_sfc(source, Default::default()).unwrap();
    let result = compile_sfc_for_adapter(
        &descriptor,
        SfcCompileOptions::default(),
        TemplateSyntaxMode::Standard,
        CustomElementMatcher::default(),
        CodegenOptions::default(),
        SfcScriptOutputMode::SeparateTemplate,
    )
    .unwrap();

    assert!(
        result
            .code
            .contains(r#"const _directive_flip = $setup["vFlip"]"#),
        "local script-setup directives must resolve from the setup state in separate-template mode:\n{}",
        result.code
    );
    assert!(
        !result.code.contains("const _directive_flip = vFlip"),
        "separate-template render functions cannot close over setup locals:\n{}",
        result.code
    );

    let returned = result
        .code
        .split("const __returned__ = {")
        .nth(1)
        .and_then(|tail| tail.split("Object.defineProperty").next())
        .unwrap_or("");
    assert!(
        returned.contains("vFlip"),
        "local directive binding must be returned to $setup:\n{}",
        result.code
    );
}

/// Options-API SFCs compile through the DOM lane with
/// `prefix_identifiers: true`. Compound dynamic keys must walk each
/// identifier; otherwise the render function throws `ReferenceError`.
#[test]
fn test_compile_sfc_compound_dynamic_bind_and_on_keys_prefix_identifiers() {
    let source = r#"
<template>
  <div :[prefix+suffix]="value" @[prefix+suffix]="handler"></div>
</template>

<script>
export default {
  data() {
    return { prefix: 'data-', suffix: 'id', value: 42 }
  },
  methods: {
    handler() {}
  }
}
</script>
"#;
    let descriptor = parse_sfc(source, Default::default()).unwrap();
    let result = compile_sfc(&descriptor, SfcCompileOptions::default()).unwrap();

    assert!(
        result.code.contains("[$data.prefix+$data.suffix || \"\"]"),
        "bind key was not prefixed:\n{}",
        result.code
    );
    assert!(
        result
            .code
            .contains("_toHandlerKey($data.prefix+$data.suffix)"),
        "on key was not prefixed:\n{}",
        result.code
    );
}
