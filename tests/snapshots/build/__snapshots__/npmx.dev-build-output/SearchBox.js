import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = { for: "header-search", class: "sr-only" }
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("span", { class: "absolute inset-is-3 text-fg-subtle font-mono text-sm pointer-events-none transition-colors duration-200 motion-reduce:transition-none [.group:hover:not(:focus-within)_&]:text-fg/80 group-focus-within:text-accent z-1" }, "\n            /\n          ")
const _hoisted_3 = { type: "submit", class: "sr-only" }

export default /*@__PURE__*/_defineComponent({
  __name: 'SearchBox',
  props: {
    inputClass: { type: String, required: false, default: 'inline sm:block' }
  },
  emits: ["blur", "focus"],
  setup(__props: any, { expose: __expose, emit: __emit }) {

const emit = __emit
const route = useRoute()
const isSearchFocused = shallowRef(false)
const showSearchBar = computed(() => {
  return route.name !== 'index'
})
const { model: searchQuery, flushUpdateUrlQuery } = useGlobalSearch('header')
function handleSubmit() {
  flushUpdateUrlQuery()
}
// Expose focus method for parent components
const inputRef = useTemplateRef('inputRef')
function focus() {
  inputRef.value?.focus()
}
__expose({ focus })

return (_ctx: any,_cache: any) => {
  const _component_InputBase = _resolveComponent("InputBase")
  const _component_search = _resolveComponent("search")

  return (showSearchBar.value)
      ? (_openBlock(), _createBlock(_component_search, {
        key: 0,
        class: _normalizeClass('flex-1 sm:max-w-md ' + __props.inputClass)
      }, {
        default: _withCtx(() => [
          _createElementVNode("form", {
            method: "GET",
            action: "/search",
            class: "relative",
            onSubmit: _withModifiers(handleSubmit, ["prevent"])
          }, [
            _createElementVNode("label", _hoisted_1, _toDisplayString(_ctx.$t('search.label')), 1 /* TEXT */),
            _createElementVNode("div", {
              class: _normalizeClass(["relative group", { 'is-focused': isSearchFocused.value }])
            }, [
              _createElementVNode("div", { class: "search-box relative flex items-center" }, [
                _hoisted_2,
                _createVNode(_component_InputBase, {
                  id: "header-search",
                  ref_key: "inputRef", ref: inputRef,
                  type: "search",
                  name: "q",
                  placeholder: _ctx.$t('search.placeholder'),
                  "no-correct": "",
                  class: "w-full min-w-25 ps-7",
                  onFocus: _cache[0] || (_cache[0] = ($event: any) => (isSearchFocused.value = true)),
                  onBlur: _cache[1] || (_cache[1] = ($event: any) => (isSearchFocused.value = false)),
                  size: "small",
                  modelValue: _unref(searchQuery),
                  "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((searchQuery).value = $event))
                }, null, 8 /* PROPS */, ["placeholder", "modelValue"]),
                _createElementVNode("button", _hoisted_3, _toDisplayString(_ctx.$t('search.button')), 1 /* TEXT */)
              ])
            ], 2 /* CLASS */)
          ], 32 /* NEED_HYDRATION */)
        ]),
        _: 1 /* STABLE */
      }, 2 /* CLASS */))
      : _createCommentVNode("v-if", true)
}
}

})
