//! Vue runtime API name checks used while classifying import bindings.

pub(super) fn is_vue_runtime_api(name: &str) -> bool {
    matches!(
        name,
        "inject"
            | "provide"
            | "ref"
            | "shallowRef"
            | "reactive"
            | "shallowReactive"
            | "computed"
            | "readonly"
            | "shallowReadonly"
            | "toRef"
            | "toRefs"
            | "watch"
            | "watchEffect"
            | "watchPostEffect"
            | "watchSyncEffect"
            | "onMounted"
            | "onUnmounted"
            | "onBeforeMount"
            | "onBeforeUnmount"
            | "onUpdated"
            | "onBeforeUpdate"
            | "onActivated"
            | "onDeactivated"
            | "onWatcherCleanup"
    )
}
