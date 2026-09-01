//! Template lowering that reaches the emitted render function:
//! ref access, custom directives, and `v-model` alongside them.
//!
//! Split out of `lib.rs` so that module stays inside the per-file
//! source-length budget.

use super::{SfcCompileOptions, compile_sfc, compile_sfc_with_template_syntax, parse_sfc};
use vize_atelier_core::TemplateSyntaxMode;
use vize_carton::config::VueVersion;

#[test]
fn test_compile_sfc_ts_ref_condition_and_handler_keep_value_access() {
    use vize_carton::ToCompactString;

    let source = r#"
<template>
  <div>
    <template v-if="folder == null">
      <MkButton @click="isRootSelected = true" />
    </template>
    <template v-else>
      <MkButton
        v-if="!selectedFolders.some(f => f.id === folder!.id)"
        @click="selectedFolders.push(folder)"
      />
      <MkButton
        v-else
        @click="selectedFolders = selectedFolders.filter(f => f.id !== folder!.id)"
      />
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

const folder = ref<{ id: string } | null>(null)
const selectedFolders = ref<{ id: string }[]>([])
const isRootSelected = ref(false)
</script>
"#;
    let descriptor = parse_sfc(source, Default::default()).unwrap();
    let script_setup = descriptor.script_setup.as_ref().unwrap();
    let template = descriptor.template.as_ref().unwrap();
    let croquis = crate::script::analyze_script_setup_to_summary(&script_setup.content);
    let mut binding_metadata = crate::BindingMetadata::default();
    binding_metadata.is_script_setup = croquis.bindings.is_script_setup;
    for (name, binding_type) in croquis.bindings.iter() {
        binding_metadata
            .bindings
            .insert(name.to_compact_string(), binding_type);
    }
    for (local, key) in &croquis.bindings.props_aliases {
        binding_metadata
            .props_aliases
            .insert(local.to_compact_string(), key.to_compact_string());
    }

    let template_allocator = vize_carton::Allocator::new();
    let template_output = crate::compile_template::compile_template_block(
        &template_allocator,
        template,
        &crate::TemplateCompileOptions::default(),
        &vize_atelier_core::options::CustomElementMatcher::default(),
        crate::compile_template::TemplateBlockCompileContext {
            scope_id: "",
            apply_scope_id: false,
            has_scoped: false,
            is_ts: true,
            inline: true,
            component_name: None,
            bindings: Some(&binding_metadata),
            croquis: Some(croquis),
        },
        vize_atelier_core::TemplateSyntaxMode::Standard,
        &vize_atelier_core::CodegenOptions::default(),
    )
    .expect("template compile should succeed");
    let template_code = template_output.code;
    assert!(
        template_code.contains("!selectedFolders.value.some((f) => f.id === folder.value.id)"),
        "unexpected template code:\n{}",
        template_code
    );
    assert!(
            template_code.contains("$event => (selectedFolders.value = selectedFolders.value.filter((f) => f.id !== folder.value.id))"),
            "unexpected template code:\n{}",
            template_code
        );
    let (_imports, _hoisted, _preamble, render_body, _render_fn_name) =
        crate::compile_template::extract_template_parts(&template_code);
    assert!(
        render_body.contains("!selectedFolders.value.some((f) => f.id === folder.value.id)"),
        "unexpected render body:\n{}",
        render_body
    );
    assert!(
            render_body.contains("$event => (selectedFolders.value = selectedFolders.value.filter((f) => f.id !== folder.value.id))"),
            "unexpected render body:\n{}",
            render_body
        );

    let result = compile_sfc(&descriptor, SfcCompileOptions::default()).unwrap();

    assert!(
        result
            .code
            .contains("!selectedFolders.value.some((f) => f.id === folder.value.id)"),
        "unexpected code:\n{}",
        result.code
    );
    assert!(
        result.code.contains(
            "selectedFolders.value = selectedFolders.value.filter((f) => f.id !== folder.value.id)"
        ),
        "unexpected code:\n{}",
        result.code
    );
    assert!(!result.code.contains("($event) => (($event) =>"));
}

#[test]
fn test_compile_sfc_nested_custom_directive_keeps_inline_with_directives() {
    let source = r#"
<template>
  <div>
    <button
      v-show="ok"
      v-appear="shouldEnableInfiniteScroll ? fetchOlder : null"
      @click="fetchOlder"
    >
      Load more
    </button>
  </div>
</template>

<script setup lang="ts">
const ok = true
const shouldEnableInfiniteScroll = true
const fetchOlder = () => {}
</script>
"#;
    let descriptor = parse_sfc(source, Default::default()).unwrap();
    let result = compile_sfc(&descriptor, SfcCompileOptions::default()).unwrap();
    let normalized = result.code.replace('\n', " ");

    assert!(
        result
            .code
            .contains(r#"const _directive_appear = _resolveDirective("appear")"#),
        "unexpected code:\n{}",
        result.code
    );
    assert!(
        normalized.contains(
            r#"[_directive_appear, shouldEnableInfiniteScroll ? fetchOlder : null], [_vShow, ok]"#
        ),
        "unexpected code:\n{}",
        result.code
    );
    assert!(
        result.code.contains("_withDirectives(_createElementVNode(")
            && result.code.contains("\"button\""),
        "unexpected code:\n{}",
        result.code
    );
}

#[test]
fn test_compile_sfc_native_v_model_keeps_custom_directive() {
    let source = r#"
<template>
  <input v-model="local" v-example @input="touches++">
</template>

<script setup lang="ts">
import { ref } from 'vue'

const local = ref('initial')
const touches = ref(0)
</script>
"#;
    let descriptor = parse_sfc(source, Default::default()).unwrap();
    let result = compile_sfc(&descriptor, SfcCompileOptions::default()).unwrap();
    let normalized = result.code.replace('\n', " ");

    assert!(
        result
            .code
            .contains(r#"const _directive_example = _resolveDirective("example")"#),
        "unexpected code:\n{}",
        result.code
    );
    assert!(
        normalized.contains(r#"[_vModelText, local.value], [_directive_example]"#),
        "unexpected code:\n{}",
        result.code
    );
    assert!(
        result.code.contains(r#""onUpdate:modelValue":"#)
            && result.code.contains("local.value = $event"),
        "unexpected code:\n{}",
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
