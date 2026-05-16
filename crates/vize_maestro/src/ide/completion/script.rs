//! Script completion provider.
//!
//! Handles completions within script blocks including Vue Composition API,
//! compiler macros, and import suggestions.
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]

use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, CompletionItemLabelDetails, Documentation, MarkupContent,
    MarkupKind,
};
use vize_croquis::{Analyzer, AnalyzerOptions};
use vize_relief::BindingType;

use super::items;
use crate::ide::IdeContext;

/// Get completions for script context.
pub(crate) fn complete_script(ctx: &IdeContext, is_setup: bool) -> Vec<CompletionItem> {
    let mut items_vec = Vec::new();

    // Add Vue Composition API
    items_vec.extend(composition_api_completions());

    // Add Vue macros (script setup only)
    if is_setup {
        items_vec.extend(macro_completions());
    }

    // Add common imports
    items_vec.extend(import_completions());

    // Use vize_croquis for accurate bindings in script
    let options = vize_atelier_sfc::SfcParseOptions {
        filename: ctx.uri.path().to_string().into(),
        ..Default::default()
    };

    if let Ok(descriptor) = vize_atelier_sfc::parse_sfc(&ctx.content, options) {
        let script = if is_setup {
            descriptor.script_setup.as_ref()
        } else {
            descriptor.script.as_ref()
        };

        if let Some(script) = script {
            let mut analyzer = Analyzer::with_options(AnalyzerOptions {
                analyze_script: true,
                ..Default::default()
            });

            if is_setup {
                analyzer.analyze_script_setup(&script.content);
            } else {
                analyzer.analyze_script_plain(&script.content);
            }

            let croquis = analyzer.finish();

            // Add bindings with type information
            for (name, binding_type) in croquis.bindings.iter() {
                let (kind, type_detail, doc) = items::binding_type_to_completion_info(binding_type);

                // For refs in script, add .value hint
                let needs_value = matches!(
                    binding_type,
                    BindingType::SetupRef | BindingType::SetupMaybeRef
                );

                #[allow(clippy::disallowed_macros)]
                items_vec.push(CompletionItem {
                    label: name.to_string(),
                    kind: Some(kind),
                    label_details: Some(CompletionItemLabelDetails {
                        detail: Some(type_detail.clone()),
                        description: if needs_value {
                            Some(".value".to_string())
                        } else {
                            None
                        },
                    }),
                    detail: Some(type_detail),
                    documentation: Some(Documentation::MarkupContent(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: doc,
                    })),
                    sort_text: Some(format!("0{}", name)),
                    ..Default::default()
                });
            }

            // Add reactive sources
            for source in croquis.reactivity.sources() {
                let needs_value = source.kind.needs_value_access();
                let kind_str = source.kind.to_display();

                #[allow(clippy::disallowed_macros)]
                items_vec.push(CompletionItem {
                    label: source.name.to_string(),
                    kind: Some(CompletionItemKind::VARIABLE),
                    label_details: Some(CompletionItemLabelDetails {
                        detail: Some(kind_str.to_string()),
                        description: if needs_value {
                            Some(".value".to_string())
                        } else {
                            None
                        },
                    }),
                    detail: Some(kind_str.to_string()),
                    documentation: Some(Documentation::MarkupContent(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: if needs_value {
                            "Needs `.value` access in script.".to_string()
                        } else {
                            "Direct access (no `.value` needed).".to_string()
                        },
                    })),
                    sort_text: Some(format!("0{}", source.name)),
                    ..Default::default()
                });
            }
        }
    }

    items_vec
}

/// Vue Composition API completions.
pub(crate) fn composition_api_completions() -> Vec<CompletionItem> {
    vec![
        items::api_item(
            "ref",
            "function ref<T>(value: T): Ref<T>",
            "Create a reactive reference",
        ),
        items::api_item(
            "reactive",
            "function reactive<T>(target: T): T",
            "Create a reactive object",
        ),
        items::api_item(
            "computed",
            "function computed<T>(getter: () => T): ComputedRef<T>",
            "Create a computed property",
        ),
        items::api_item(
            "watch",
            "function watch(source, callback, options?)",
            "Watch reactive sources",
        ),
        items::api_item(
            "watchEffect",
            "function watchEffect(effect: () => void)",
            "Run effect with auto-tracking",
        ),
        items::api_item(
            "onMounted",
            "function onMounted(callback: () => void)",
            "Lifecycle: after mount",
        ),
        items::api_item(
            "onUnmounted",
            "function onUnmounted(callback: () => void)",
            "Lifecycle: after unmount",
        ),
        items::api_item(
            "onBeforeMount",
            "function onBeforeMount(callback: () => void)",
            "Lifecycle: before mount",
        ),
        items::api_item(
            "onBeforeUnmount",
            "function onBeforeUnmount(callback: () => void)",
            "Lifecycle: before unmount",
        ),
        items::api_item(
            "onUpdated",
            "function onUpdated(callback: () => void)",
            "Lifecycle: after update",
        ),
        items::api_item(
            "onBeforeUpdate",
            "function onBeforeUpdate(callback: () => void)",
            "Lifecycle: before update",
        ),
        items::api_item(
            "toRef",
            "function toRef<T>(object: T, key: K): Ref<T[K]>",
            "Create ref from reactive property",
        ),
        items::api_item(
            "toRefs",
            "function toRefs<T>(object: T): ToRefs<T>",
            "Convert reactive to refs",
        ),
        items::api_item(
            "unref",
            "function unref<T>(ref: T | Ref<T>): T",
            "Unwrap a ref",
        ),
        items::api_item(
            "isRef",
            "function isRef(r): r is Ref",
            "Check if value is ref",
        ),
        items::api_item(
            "shallowRef",
            "function shallowRef<T>(value: T): ShallowRef<T>",
            "Shallow reactive reference",
        ),
        items::api_item(
            "shallowReactive",
            "function shallowReactive<T>(target: T): T",
            "Shallow reactive object",
        ),
        items::api_item(
            "readonly",
            "function readonly<T>(target: T): DeepReadonly<T>",
            "Create readonly proxy",
        ),
        items::api_item(
            "nextTick",
            "function nextTick(callback?): Promise<void>",
            "Wait for next DOM update",
        ),
        items::api_item(
            "provide",
            "function provide<T>(key, value: T)",
            "Provide value to descendants",
        ),
        items::api_item(
            "inject",
            "function inject<T>(key, defaultValue?): T",
            "Inject value from ancestor",
        ),
    ]
}

/// Vue macro completions (script setup only).
pub(crate) fn macro_completions() -> Vec<CompletionItem> {
    vec![
        items::macro_item(
            "defineProps",
            "defineProps<T>()",
            "Declare component props",
            "defineProps<{\n\t$1\n}>()",
        ),
        items::macro_item(
            "defineEmits",
            "defineEmits<T>()",
            "Declare component emits",
            "defineEmits<{\n\t$1\n}>()",
        ),
        items::macro_item(
            "defineExpose",
            "defineExpose(exposed)",
            "Expose properties via refs",
            "defineExpose({\n\t$1\n})",
        ),
        items::macro_item(
            "defineOptions",
            "defineOptions(options)",
            "Declare component options",
            "defineOptions({\n\tname: '$1',\n})",
        ),
        items::macro_item(
            "defineSlots",
            "defineSlots<T>()",
            "Declare typed slots",
            "defineSlots<{\n\t$1\n}>()",
        ),
        items::macro_item(
            "defineModel",
            "defineModel<T>(name?, options?)",
            "Declare two-way binding prop",
            "defineModel<$1>()",
        ),
        items::macro_item(
            "withDefaults",
            "withDefaults(props, defaults)",
            "Set prop defaults",
            "withDefaults(defineProps<{\n\t$1\n}>(), {\n\t$2\n})",
        ),
    ]
}

/// Common import completions.
fn import_completions() -> Vec<CompletionItem> {
    vec![
        items::import_item("import vue", "Import from Vue", "import { $1 } from 'vue'"),
        items::import_item(
            "import ref",
            "Import ref from Vue",
            "import { ref } from 'vue'",
        ),
        items::import_item(
            "import reactive",
            "Import reactive from Vue",
            "import { reactive } from 'vue'",
        ),
        items::import_item(
            "import computed",
            "Import computed from Vue",
            "import { computed } from 'vue'",
        ),
        items::import_item(
            "import watch",
            "Import watch from Vue",
            "import { watch, watchEffect } from 'vue'",
        ),
        items::import_item(
            "import lifecycle",
            "Import lifecycle hooks",
            "import { onMounted, onUnmounted } from 'vue'",
        ),
    ]
}
