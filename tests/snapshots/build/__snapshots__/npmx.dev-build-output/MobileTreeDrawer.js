import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, Transition as _Transition, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx } from "vue"


const _hoisted_1 = { class: "font-mono text-sm text-fg-muted" }
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("span", { "aria-hidden": "true", class: "flex-shrink-1 flex-grow-1" })
import type { PackageFileTree } from '#shared/types'
import type { RouteNamedMap } from 'vue-router/auto-routes'

export default /*@__PURE__*/_defineComponent({
  __name: 'MobileTreeDrawer',
  props: {
    tree: { type: Array, required: true },
    currentPath: { type: String, required: true },
    baseUrl: { type: String, required: true },
    baseRoute: { type: null, required: true }
  },
  setup(__props: any) {

const isOpen = shallowRef(false)
// Close drawer on navigation
const route = useRoute()
watch(
  () => route.fullPath,
  () => {
    isOpen.value = false
  },
)
const isLocked = useScrollLock(document)
// Prevent body scroll when drawer is open
watch(isOpen, open => (isLocked.value = open))

return (_ctx: any,_cache: any) => {
  const _component_ButtonBase = _resolveComponent("ButtonBase")
  const _component_CodeFileTree = _resolveComponent("CodeFileTree")

  return (_openBlock(), _createElementBlock(_Fragment, null, [ _createVNode(_component_ButtonBase, {
        variant: "primary",
        class: "md:hidden fixed bottom-4 inset-ie-4 z-45",
        "aria-label": _ctx.$t('code.toggle_tree'),
        onClick: _cache[0] || (_cache[0] = ($event: any) => (isOpen.value = !isOpen.value)),
        classicon: isOpen.value ? 'i-lucide:x' : 'i-lucide:folder'
      }, null, 8 /* PROPS */, ["aria-label", "classicon"]), _createVNode(_Transition, {
        "enter-active-class": "transition-opacity duration-200",
        "enter-from-class": "opacity-0",
        "enter-to-class": "opacity-100",
        "leave-active-class": "transition-opacity duration-200",
        "leave-from-class": "opacity-100",
        "leave-to-class": "opacity-0"
      }, {
        default: _withCtx(() => [
          (isOpen.value)
            ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: "md:hidden fixed inset-0 z-40 bg-black/50",
              onClick: _cache[1] || (_cache[1] = ($event: any) => (isOpen.value = false))
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
          (isOpen.value)
            ? (_openBlock(), _createElementBlock("aside", {
              key: 0,
              class: "md:hidden fixed inset-y-0 inset-is-0 z-50 w-72 bg-bg-subtle border-ie border-border overflow-y-auto"
            }, [
              _createElementVNode("div", { class: "sticky top-0 z-10 bg-bg-subtle border-b border-border px-4 py-3 flex items-center justify-start" }, [
                _createElementVNode("span", _hoisted_1, _toDisplayString(_ctx.$t('code.files_label')), 1 /* TEXT */),
                _hoisted_2,
                _createVNode(_component_ButtonBase, {
                  "aria-label": _ctx.$t('code.close_tree'),
                  onClick: _cache[2] || (_cache[2] = ($event: any) => (isOpen.value = false)),
                  classicon: "i-lucide:x"
                }, null, 8 /* PROPS */, ["aria-label"])
              ]),
              _createVNode(_component_CodeFileTree, {
                tree: __props.tree,
                "current-path": __props.currentPath,
                "base-url": __props.baseUrl,
                "base-route": __props.baseRoute
              }, null, 8 /* PROPS */, ["tree", "current-path", "base-url", "base-route"])
            ]))
            : _createCommentVNode("v-if", true)
        ]),
        _: 1 /* STABLE */
      }) ], 64 /* STABLE_FRAGMENT */))
}
}

})
