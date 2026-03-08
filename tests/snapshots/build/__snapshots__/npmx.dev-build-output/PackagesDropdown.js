import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, Transition as _Transition, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx } from "vue"


const _hoisted_1 = { class: "font-mono text-xs text-fg-subtle" }
const _hoisted_2 = { class: "text-fg-muted text-sm" }
const _hoisted_3 = { class: "text-fg-muted text-sm" }
const _hoisted_4 = { class: "text-fg-muted text-sm" }
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:arrow-right rtl-flip w-3 h-3", "aria-hidden": "true" })

export default /*@__PURE__*/_defineComponent({
  __name: 'PackagesDropdown',
  props: {
    username: { type: String, required: true }
  },
  setup(__props: any) {

const props = __props
const { listUserPackages } = useConnector()
const isOpen = shallowRef(false)
const isLoading = shallowRef(false)
const packages = shallowRef<string[]>([])
const hasLoaded = shallowRef(false)
const error = shallowRef<string | null>(null)
async function loadPackages() {
  if (hasLoaded.value || isLoading.value) return
  isLoading.value = true
  error.value = null
  try {
    const pkgMap = await listUserPackages()
    if (pkgMap) {
      // Sort alphabetically and take top 10
      packages.value = Object.keys(pkgMap).sort().slice(0, 10)
    } else {
      error.value = $t('header.packages_dropdown.error')
    }
    hasLoaded.value = true
  } catch {
    error.value = $t('header.packages_dropdown.error')
  } finally {
    isLoading.value = false
  }
}
function handleMouseEnter() {
  isOpen.value = true
  if (!hasLoaded.value) {
    loadPackages()
  }
}
function handleMouseLeave() {
  isOpen.value = false
}
function handleKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape' && isOpen.value) {
    isOpen.value = false
  }
}

return (_ctx: any,_cache: any) => {
  const _component_NuxtLink = _resolveComponent("NuxtLink")

  return (_openBlock(), _createElementBlock("div", {
      class: "relative",
      onMouseenter: handleMouseEnter,
      onMouseleave: handleMouseLeave,
      onKeydown: handleKeydown
    }, [ _createVNode(_component_NuxtLink, {
        to: { name: '~username', params: { username: __props.username } },
        class: "link-subtle font-mono text-sm inline-flex items-center gap-1"
      }, {
        default: _withCtx(() => [
          _createTextVNode(_toDisplayString(_ctx.$t('header.packages')), 1 /* TEXT */),
          _createTextVNode("\n      "),
          _createElementVNode("span", {
            class: _normalizeClass(["i-lucide:chevron-down w-3 h-3 transition-transform duration-200", { 'rotate-180': isOpen.value }]),
            "aria-hidden": "true"
          }, null, 2 /* CLASS */)
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["to"]), _createVNode(_Transition, {
        "enter-active-class": "transition-all duration-150",
        "leave-active-class": "transition-all duration-100",
        "enter-from-class": "opacity-0 translate-y-1",
        "leave-to-class": "opacity-0 translate-y-1"
      }, {
        default: _withCtx(() => [
          (isOpen.value)
            ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: "absolute inset-ie-0 top-full pt-2 w-64 z-50"
            }, [
              _createElementVNode("div", { class: "bg-bg-elevated border border-border rounded-lg shadow-lg overflow-hidden" }, [
                _createElementVNode("div", { class: "px-3 py-2 border-b border-border" }, [
                  _createElementVNode("span", _hoisted_1, _toDisplayString(_ctx.$t('header.packages_dropdown.title')), 1 /* TEXT */)
                ]),
                (isLoading.value)
                  ? (_openBlock(), _createElementBlock("div", {
                    key: 0,
                    class: "px-3 py-4 text-center"
                  }, [
                    _createElementVNode("span", _hoisted_2, _toDisplayString(_ctx.$t('header.packages_dropdown.loading')), 1 /* TEXT */)
                  ]))
                  : (error.value)
                    ? (_openBlock(), _createElementBlock("div", {
                      key: 1,
                      class: "px-3 py-4 text-center"
                    }, [
                      _createElementVNode("span", _hoisted_3, _toDisplayString(_ctx.$t('header.packages_dropdown.error')), 1 /* TEXT */)
                    ]))
                  : (packages.value.length > 0)
                    ? (_openBlock(), _createElementBlock("ul", {
                      key: 2,
                      class: "py-1 max-h-80 overflow-y-auto"
                    }, [
                      (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(packages.value, (pkg) => {
                        return (_openBlock(), _createElementBlock("li", { key: pkg }, [
                          _createVNode(_component_NuxtLink, {
                            to: _ctx.packageRoute(pkg),
                            class: "block px-3 py-2 font-mono text-sm text-fg hover:bg-bg-subtle transition-colors truncate"
                          }, {
                            default: _withCtx(() => [
                              _createTextVNode(_toDisplayString(pkg), 1 /* TEXT */)
                            ]),
                            _: 2 /* DYNAMIC */
                          }, 8 /* PROPS */, ["to"])
                        ]))
                      }), 128 /* KEYED_FRAGMENT */))
                    ]))
                  : (_openBlock(), _createElementBlock("div", {
                    key: 3,
                    class: "px-3 py-4 text-center"
                  }, [
                    _createElementVNode("span", _hoisted_4, _toDisplayString(_ctx.$t('header.packages_dropdown.empty')), 1 /* TEXT */)
                  ])),
                _createElementVNode("div", { class: "px-3 py-2 border-t border-border" }, [
                  _createVNode(_component_NuxtLink, {
                    to: { name: '~username', params: { username: __props.username } },
                    class: "link-subtle font-mono text-xs inline-flex items-center gap-1"
                  }, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_ctx.$t('header.packages_dropdown.view_all')), 1 /* TEXT */),
                      _createTextVNode("\n              "),
                      _hoisted_5
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["to"])
                ])
              ])
            ]))
            : _createCommentVNode("v-if", true)
        ]),
        _: 1 /* STABLE */
      }) ], 32 /* NEED_HYDRATION */))
}
}

})
