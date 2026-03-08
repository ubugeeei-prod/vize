import { defineComponent as _defineComponent } from 'vue'
import { Transition as _Transition, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = { class: "text-xs text-fg-subtle mt-0.5" }
const _hoisted_2 = { class: "text-xs text-fg-subtle mt-0.5" }
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:external-link w-3 h-3", "aria-hidden": "true" })

export default /*@__PURE__*/_defineComponent({
  __name: 'SearchProviderToggle.client',
  setup(__props) {

const route = useRoute()
const router = useRouter()
const { searchProvider } = useSearchProvider()
const searchProviderValue = computed(() => {
  const p = normalizeSearchParam(route.query.p)
  if (p === 'npm' || searchProvider.value === 'npm') return 'npm'
  return 'algolia'
})
const isOpen = shallowRef(false)
const toggleRef = useTemplateRef('toggleRef')
onClickOutside(toggleRef, () => {
  isOpen.value = false
})
useEventListener('keydown', event => {
  if (event.key === 'Escape' && isOpen.value) {
    isOpen.value = false
  }
})

return (_ctx: any,_cache: any) => {
  const _component_ButtonBase = _resolveComponent("ButtonBase")

  return (_openBlock(), _createElementBlock("div", {
      ref_key: "toggleRef", ref: toggleRef,
      class: "relative"
    }, [ _createVNode(_component_ButtonBase, {
        "aria-label": _ctx.$t('settings.data_source.label'),
        "aria-expanded": isOpen.value,
        "aria-haspopup": "true",
        size: "small",
        class: "border-none w-8 h-8 !px-0 justify-center",
        classicon: "i-lucide:settings",
        onClick: _cache[0] || (_cache[0] = ($event: any) => (isOpen.value = !isOpen.value))
      }, null, 8 /* PROPS */, ["aria-label", "aria-expanded"]), _createVNode(_Transition, {
        "enter-active-class": "transition-all duration-150",
        "leave-active-class": "transition-all duration-100",
        "enter-from-class": "opacity-0 translate-y-1",
        "leave-to-class": "opacity-0 translate-y-1"
      }, {
        default: _withCtx(() => [
          (isOpen.value)
            ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: "absolute inset-ie-0 top-full pt-2 w-72 z-50",
              role: "menu",
              "aria-label": _ctx.$t('settings.data_source.label')
            }, [
              _createElementVNode("div", { class: "bg-bg-subtle/80 backdrop-blur-sm border border-border-subtle rounded-lg shadow-lg shadow-bg-elevated/50 overflow-hidden p-1" }, [
                _createElementVNode("button", {
                  type: "button",
                  role: "menuitem",
                  class: _normalizeClass(["w-full flex items-start gap-3 px-3 py-2.5 rounded-md text-start transition-colors hover:bg-bg-muted", [searchProviderValue.value !== 'algolia' ? 'bg-bg-muted' : '']]),
                  onClick: _cache[1] || (_cache[1] = () => {
  	searchProvider.value = "npm";
  	_unref(router).push({ query: {
  		..._unref(route).query,
  		p: "npm"
  	} });
  	isOpen.value = false;
  })
                }, [
                  _createElementVNode("span", {
                    class: _normalizeClass(["i-simple-icons:npm w-4 h-4 mt-0.5 shrink-0", searchProviderValue.value !== 'algolia' ? 'text-accent' : 'text-fg-muted']),
                    "aria-hidden": "true"
                  }, null, 2 /* CLASS */),
                  _createElementVNode("div", { class: "min-w-0 flex-1" }, [
                    _createElementVNode("div", {
                      class: _normalizeClass(["text-sm font-medium", searchProviderValue.value !== 'algolia' ? 'text-fg' : 'text-fg-muted'])
                    }, _toDisplayString(_ctx.$t('settings.data_source.npm')), 3 /* TEXT, CLASS */),
                    _createElementVNode("p", _hoisted_1, _toDisplayString(_ctx.$t('settings.data_source.npm_description')), 1 /* TEXT */)
                  ])
                ], 2 /* CLASS */),
                _createTextVNode("\n\n          " + "\n          "),
                _createElementVNode("button", {
                  type: "button",
                  role: "menuitem",
                  class: _normalizeClass(["w-full flex items-start gap-3 px-3 py-2.5 rounded-md text-start transition-colors hover:bg-bg-muted mt-1", [searchProviderValue.value === 'algolia' ? 'bg-bg-muted' : '']]),
                  onClick: _cache[2] || (_cache[2] = () => {
  	searchProvider.value = "algolia";
  	_unref(router).push({ query: {
  		..._unref(route).query,
  		p: undefined
  	} });
  	isOpen.value = false;
  })
                }, [
                  _createElementVNode("span", {
                    class: _normalizeClass(["i-simple-icons:algolia w-4 h-4 mt-0.5 shrink-0", searchProviderValue.value === 'algolia' ? 'text-accent' : 'text-fg-muted']),
                    "aria-hidden": "true"
                  }, null, 2 /* CLASS */),
                  _createElementVNode("div", { class: "min-w-0 flex-1" }, [
                    _createElementVNode("div", {
                      class: _normalizeClass(["text-sm font-medium", searchProviderValue.value === 'algolia' ? 'text-fg' : 'text-fg-muted'])
                    }, _toDisplayString(_ctx.$t('settings.data_source.algolia')), 3 /* TEXT, CLASS */),
                    _createElementVNode("p", _hoisted_2, _toDisplayString(_ctx.$t('settings.data_source.algolia_description')), 1 /* TEXT */)
                  ])
                ], 2 /* CLASS */),
                _createTextVNode("\n\n          " + "\n          "),
                (searchProviderValue.value === 'algolia')
                  ? (_openBlock(), _createElementBlock("div", {
                    key: 0,
                    class: "border-t border-border mx-1 mt-1 pt-2 pb-1"
                  }, [
                    _createElementVNode("a", {
                      href: "https://www.algolia.com/developers",
                      target: "_blank",
                      rel: "noopener noreferrer",
                      class: "text-xs text-fg-subtle hover:text-fg-muted transition-colors inline-flex items-center gap-1 px-2"
                    }, [
                      _createTextVNode(_toDisplayString(_ctx.$t('search.algolia_disclaimer')) + "\n              ", 1 /* TEXT */),
                      _hoisted_3
                    ])
                  ]))
                  : _createCommentVNode("v-if", true)
              ])
            ]))
            : _createCommentVNode("v-if", true)
        ]),
        _: 1 /* STABLE */
      }) ], 512 /* NEED_PATCH */))
}
}

})
