import { useModel as _useModel } from 'vue'
import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, Transition as _Transition, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx } from "vue"


const _hoisted_1 = { class: "text-green-500" }
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("span", { class: "text-fg-subtle" }, "/")
const _hoisted_3 = { class: "text-red-500" }
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("span", { class: "text-fg-subtle" }, "/")
const _hoisted_5 = { class: "text-yellow-500" }
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("span", { class: "text-fg-subtle" }, "•")
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:x w-5 h-5" })
import type { CompareResponse, FileChange } from '#shared/types'

export default /*@__PURE__*/_defineComponent({
  __name: 'MobileSidebarDrawer',
  props: {
    compare: { type: null, required: true },
    groupedDeps: { type: Map, required: true },
    allChanges: { type: Array, required: true },
    "selectedFile": { default: null },
    "fileFilter": {
  default: 'all',
},
    "open": { default: false }
  },
  emits: ["update:selectedFile", "update:fileFilter", "update:open"],
  setup(__props: any) {

const props = __props
const selectedFile = _useModel(__props, "selectedFile")
const fileFilter = _useModel(__props, "fileFilter")
const open = _useModel(__props, "open")
const route = useRoute()
watch(
  () => route.fullPath,
  () => {
    open.value = false
  },
)
const isLocked = useScrollLock(import.meta.client ? document : null)
watch(open, value => {
  isLocked.value = value
})

return (_ctx: any,_cache: any) => {
  const _component_DiffSidebarPanel = _resolveComponent("DiffSidebarPanel")

  return (_openBlock(), _createElementBlock(_Fragment, null, [ _createVNode(_Transition, {
        "enter-active-class": "transition-opacity duration-200",
        "enter-from-class": "opacity-0",
        "enter-to-class": "opacity-100",
        "leave-active-class": "transition-opacity duration-200",
        "leave-from-class": "opacity-100",
        "leave-to-class": "opacity-0"
      }, {
        default: _withCtx(() => [
          (open.value)
            ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: "md:hidden fixed inset-0 z-40 bg-black/50",
              onClick: _cache[0] || (_cache[0] = ($event: any) => (open.value = false))
            }))
            : _createCommentVNode("v-if", true)
        ]),
        _: 1 /* STABLE */
      }), _createVNode(_Transition, {
        "enter-active-class": "transition-transform duration-200",
        "enter-from-class": "-translate-x-full",
        "enter-to-class": "translate-x-0",
        "leave-active-class": "transition-transform duration-200",
        "leave-from-class": "translate-x-0",
        "leave-to-class": "-translate-x-full"
      }, {
        default: _withCtx(() => [
          (open.value)
            ? (_openBlock(), _createElementBlock("aside", {
              key: 0,
              class: "md:hidden fixed inset-y-0 inset-is-0 z-50 w-72 max-w-[85vw] bg-bg-subtle border-ie border-border overflow-y-auto flex flex-col"
            }, [
              _createElementVNode("div", { class: "sticky top-0 bg-bg-subtle border-b border-border px-4 py-3 flex items-center justify-between gap-2" }, [
                _createElementVNode("div", { class: "text-xs font-mono text-fg-muted flex items-center gap-2" }, [
                  _createElementVNode("span", { class: "flex items-center gap-1" }, [
                    _createElementVNode("span", _hoisted_1, "+" + _toDisplayString(props.compare.stats.filesAdded), 1 /* TEXT */),
                    _hoisted_2,
                    _createElementVNode("span", _hoisted_3, "-" + _toDisplayString(props.compare.stats.filesRemoved), 1 /* TEXT */),
                    _hoisted_4,
                    _createElementVNode("span", _hoisted_5, "~" + _toDisplayString(props.compare.stats.filesModified), 1 /* TEXT */)
                  ]),
                  _hoisted_6,
                  _createElementVNode("span", null, _toDisplayString(_ctx.$t('compare.files_count', { count: props.allChanges.length })), 1 /* TEXT */)
                ]),
                _createElementVNode("button", {
                  type: "button",
                  class: "text-fg-muted hover:text-fg transition-colors",
                  "aria-label": _ctx.$t('compare.close_files_panel'),
                  onClick: _cache[1] || (_cache[1] = ($event: any) => (open.value = false))
                }, [
                  _hoisted_7
                ], 8 /* PROPS */, ["aria-label"])
              ]),
              _createVNode(_component_DiffSidebarPanel, {
                compare: props.compare,
                "grouped-deps": props.groupedDeps,
                "all-changes": props.allChanges,
                "selected-file": selectedFile.value,
                "onUpdate:selected-file": _cache[2] || (_cache[2] = ($event: any) => ((selectedFile).value = $event)),
                "file-filter": fileFilter.value,
                "onUpdate:file-filter": _cache[3] || (_cache[3] = ($event: any) => ((fileFilter).value = $event))
              }, null, 8 /* PROPS */, ["compare", "grouped-deps", "all-changes", "selected-file", "file-filter"])
            ]))
            : _createCommentVNode("v-if", true)
        ]),
        _: 1 /* STABLE */
      }) ], 64 /* STABLE_FRAGMENT */))
}
}

})
