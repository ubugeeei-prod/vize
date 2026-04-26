//! Vue Single File Component (.vue) compiler.
//!
//! This module provides parsing and compilation of Vue SFCs, following the
//! Vue.js core structure:
//!
//! - `parse` - SFC parsing into descriptor blocks
//! - `compile_script` - Script/script setup compilation
//! - `compile_template` - Template block compilation (DOM and Vapor)
//! - `compile` - Main SFC compilation orchestration
//! - `style` - Style block compilation with scoped CSS
//! - `css` - Low-level CSS compilation with LightningCSS
//!
//! # Example
//!
//! ```ignore
//! use vize_atelier_sfc::{parse_sfc, compile_sfc, SfcParseOptions, SfcCompileOptions};
//!
//! let source = r#"
//! <script setup>
//! import { ref } from 'vue'
//! const count = ref(0)
//! </script>
//! <template>
//!   <button @click="count++">{{ count }}</button>
//! </template>
//! "#;
//!
//! let descriptor = parse_sfc(source, SfcParseOptions::default()).unwrap();
//! let result = compile_sfc(&descriptor, SfcCompileOptions::default()).unwrap();
//! println!("{}", result.code);
//! ```

#![allow(clippy::collapsible_match)]
#![allow(clippy::type_complexity)]
#![allow(clippy::redundant_field_names)]
#![allow(clippy::unnecessary_lazy_evaluations)]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::only_used_in_recursion)]

// Core modules - following Vue.js compiler-sfc structure
pub mod compile;
pub mod compile_script;
pub mod compile_template;
pub mod css;
pub mod parse;
pub mod rewrite_default;
pub mod script;
pub mod style;
pub mod types;

// Re-exports for public API
pub use compile::{compile_sfc, ScriptCompileResult};
pub use css::{
    bundle_css, compile_css, compile_style_block, CssCompileOptions, CssCompileResult, CssTargets,
};
pub use parse::parse_sfc;
pub use types::{
    BindingMetadata, BindingType, BlockLocation, PadOption, PropsDestructure, ScriptCompileOptions,
    SfcCompileOptions, SfcCompileResult, SfcCustomBlock, SfcDescriptor, SfcError, SfcParseOptions,
    SfcScriptBlock, SfcStyleBlock, SfcTemplateBlock, StyleCompileOptions, TemplateCompileOptions,
};

// Re-export key types from dependencies
pub use vize_atelier_core::CompilerError;
pub use vize_atelier_dom::compile_template;

#[cfg(test)]
mod snapshot_tests;

#[cfg(test)]
mod tests {
    use super::{compile_sfc, parse_sfc, SfcCompileOptions};

    #[test]
    fn test_parse_simple_sfc() {
        let source = r#"
<template>
  <div>Hello World</div>
</template>

<script>
export default {
  name: 'HelloWorld'
}
</script>

<style>
.hello { color: red; }
</style>
"#;
        let descriptor = parse_sfc(source, Default::default()).unwrap();

        assert!(descriptor.template.is_some());
        assert!(descriptor.script.is_some());
        assert_eq!(descriptor.styles.len(), 1);
    }

    #[test]
    fn test_parse_script_setup() {
        let source = r#"
<template>
  <div>{{ msg }}</div>
</template>

<script setup>
import { ref } from 'vue'
const msg = ref('Hello')
</script>
"#;
        let descriptor = parse_sfc(source, Default::default()).unwrap();

        assert!(descriptor.template.is_some());
        assert!(descriptor.script_setup.is_some());
    }

    #[test]
    fn test_parse_scoped_style() {
        let source = r#"
<template>
  <div class="container">Scoped</div>
</template>

<style scoped>
.container { background: blue; }
</style>
"#;
        let descriptor = parse_sfc(source, Default::default()).unwrap();

        assert_eq!(descriptor.styles.len(), 1);
        assert!(descriptor.styles[0].scoped);
    }

    #[test]
    fn test_compile_sfc_with_define_emits() {
        let source = r#"
<template>
  <button @click="onClick">{{ count }}</button>
</template>

<script setup>
import { ref } from 'vue'
const emit = defineEmits(['update'])
const count = ref(0)
function onClick() {
    emit('update', count.value)
}
</script>
"#;
        let descriptor = parse_sfc(source, Default::default()).unwrap();
        let result = compile_sfc(&descriptor, SfcCompileOptions::default()).unwrap();
        assert!(
            result.code.contains(r#"emits: ["update"]"#),
            "unexpected code:\n{}",
            result.code
        );
        assert!(
            result.code.contains("const emit = __emit"),
            "unexpected code:\n{}",
            result.code
        );
        assert!(
            result.code.contains("emit('update', count.value)")
                || result.code.contains("emit(\"update\", count.value)"),
            "unexpected code:\n{}",
            result.code
        );

        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_compile_sfc_define_model_with_type_args_preserves_body() {
        // Regression test: defineModel<Type>('name', { opts }); was wrongly detected
        // as a multi-line macro call because the line ends with `;` not `)`.
        // This caused all subsequent setup code to be swallowed by the macro tracker.
        let source = r#"
<template>
  <div>{{ fx }}</div>
</template>

<script setup lang="ts">
interface Layer { fxId: string }
const layer = defineModel<Layer>('layer', { required: true });

const fx = layer.value.fxId;
if (fx == null) {
  throw new Error('not found');
}
</script>
"#;
        let descriptor = parse_sfc(source, Default::default()).unwrap();
        let result = compile_sfc(&descriptor, SfcCompileOptions::default()).unwrap();

        insta::assert_snapshot!(result.code.as_str());
    }

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
        let script_setup = descriptor
            .script_setup
            .as_ref()
            .expect("expected script setup block");
        let template = descriptor
            .template
            .as_ref()
            .expect("expected template block");
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

        let template_code = crate::compile_template::compile_template_block(
            template,
            &crate::TemplateCompileOptions::default(),
            crate::compile_template::TemplateBlockCompileContext {
                scope_id: "",
                apply_scope_id: false,
                is_ts: true,
                component_name: None,
                bindings: Some(&binding_metadata),
                croquis: Some(croquis),
            },
        )
        .expect("template compile should succeed");
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
            result
                .code
                .contains("selectedFolders.value = selectedFolders.value.filter((f) => f.id !== folder.value.id)"),
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
}
