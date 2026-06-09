//! Static completion lists: Vue Composition API, compiler macros, and common
//! import suggestions surfaced inside script blocks.
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]

use tower_lsp::lsp_types::CompletionItem;

use crate::ide::completion::items;

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
            "defineArt",
            "defineArt(source, options)",
            "Declare Musea art metadata",
            "defineArt(\"$1\", {\n\ttitle: \"$2\",\n});",
        ),
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
pub(crate) fn import_completions() -> Vec<CompletionItem> {
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
