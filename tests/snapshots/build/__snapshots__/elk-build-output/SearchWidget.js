import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, withDirectives as _withDirectives, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, unref as _unref, vModelText as _vModelText, withModifiers as _withModifiers, withKeys as _withKeys } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("div", { "i-ri:search-2-line": "true", "pointer-events-none": "true", "text-secondary": "true", mt: "1px", class: "rtl-flip" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("span", { "aria-hidden": "true", class: "i-ri:close-line" })

export default /*@__PURE__*/_defineComponent({
  __name: 'SearchWidget',
  setup(__props, { expose: __expose }) {

const query = ref('')
const { accounts, hashtags, loading, statuses } = useSearch(query)
const index = ref(0)
const { t } = useI18n()
const el = ref<HTMLElement>()
const input = ref<HTMLInputElement>()
const router = useRouter()
const { focused } = useFocusWithin(el)
const results = computed(() => {
  if (query.value.length === 0)
    return []

  const results = [
    ...hashtags.value.slice(0, 3),
    ...accounts.value,
    ...statuses.value,

    // Disable until search page is implemented
    // {
    //   type: 'action',
    //   to: `/search?q=${query.value}`,
    //   action: {
    //     label: `Search for ${query.value}`,
    //   },
    // },
  ]

  return results
})
// Reset index when results change
watch([results, focused], () => index.value = -1)
function shift(delta: number) {
  return index.value = (index.value + delta % results.value.length + results.value.length) % results.value.length
}
function activate() {
  const currentIndex = index.value
  if (query.value.length === 0)
    return
  // Disable redirection until search page is implemented
  if (currentIndex === -1) {
    index.value = 0
    // router.push(`/search?q=${query.value}`)
    return
  }
  (document.activeElement as HTMLElement).blur()
  index.value = -1
  router.push(results.value[currentIndex].to)
}
__expose({
  input,
})

return (_ctx: any,_cache: any) => {
  const _component_SearchResult = _resolveComponent("SearchResult")
  const _component_SearchResultSkeleton = _resolveComponent("SearchResultSkeleton")

  return (_openBlock(), _createElementBlock("div", {
      ref_key: "el", ref: el,
      relative: "",
      group: ""
    }, [ _createElementVNode("div", {
        "bg-base": "",
        border: "~ base",
        h10: "",
        "ps-4": "",
        "pe-1": "",
        "rounded-3": "",
        flex: "~ row",
        "items-center": "",
        relative: "",
        "focus-within:box-shadow-outline": ""
      }, [ _hoisted_1, _withDirectives(_createElementVNode("input", {
          ref_key: "input", ref: input,
          "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((query).value = $event)),
          "h-full": "",
          "rounded-3": "",
          "w-full": "",
          "bg-transparent": "",
          outline: "focus:none",
          "ps-3": "",
          "pe-1": "",
          "ml-1": "",
          placeholder: _unref(t)('nav.search'),
          pb: "1px",
          "placeholder-text-secondary": "",
          onKeydown: [_withKeys(_withModifiers(($event: any) => (shift(1)), ["prevent"]), ["down"]), _withKeys(_withModifiers(($event: any) => (shift(-1)), ["prevent"]), ["up"]), _withKeys(_withModifiers(($event: any) => (input.value?.blur()), ["prevent"]), ["esc"])],
          onKeypress: _withKeys(activate, ["enter"])
        }, null, 40 /* PROPS, NEED_HYDRATION */, ["placeholder"]), [ [_vModelText, query.value] ]), (query.value.length) ? (_openBlock(), _createElementBlock("button", {
            key: 0,
            "btn-action-icon": "",
            "text-secondary": "",
            onClick: _cache[1] || (_cache[1] = ($event: any) => { query.value = ''; input.value?.focus() })
          }, [ _hoisted_2 ])) : _createCommentVNode("v-if", true) ]), _createTextVNode("\n    " + "\n    "), _createElementVNode("div", {
        "left-0": "",
        "top-11": "",
        absolute: "",
        "w-full": "",
        "z-10": "",
        "group-focus-within": "pointer-events-auto visible",
        invisible: "",
        "pointer-events-none": ""
      }, [ _createElementVNode("div", {
          "w-full": "",
          "bg-base": "",
          border: "~ base",
          "rounded-3": "",
          "max-h-100": "",
          "overflow-auto": "",
          class: _normalizeClass(results.value.length === 0 ? 'py2' : null)
        }, [ (query.value.trim().length === 0) ? (_openBlock(), _createElementBlock("span", {
              key: 0,
              block: "",
              "text-center": "",
              "text-sm": "",
              "text-secondary": ""
            }, _toDisplayString(_unref(t)('search.search_desc')), 1 /* TEXT */)) : (!_unref(loading)) ? (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [ (results.value.length > 0) ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(results.value, (result, i) => {
                      return (_openBlock(), _createBlock(_component_SearchResult, {
                        key: result.id,
                        active: index.value === parseInt(i.toString()),
                        result: result,
                        tabindex: _unref(focused) ? 0 : -1
                      }, null, 8 /* PROPS */, ["active", "result", "tabindex"]))
                    }), 128 /* KEYED_FRAGMENT */)) ], 64 /* STABLE_FRAGMENT */)) : (_openBlock(), _createElementBlock("span", {
                    key: 1,
                    block: "",
                    "text-center": "",
                    "text-sm": "",
                    "text-secondary": ""
                  }, _toDisplayString(_unref(t)('search.search_empty')), 1 /* TEXT */)) ], 64 /* STABLE_FRAGMENT */)) : (_openBlock(), _createElementBlock("div", { key: 2 }, [ _createVNode(_component_SearchResultSkeleton), _createVNode(_component_SearchResultSkeleton), _createVNode(_component_SearchResultSkeleton) ])) ], 2 /* CLASS */) ]) ], 512 /* NEED_PATCH */))
}
}

})
