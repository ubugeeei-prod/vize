import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx } from "vue"


const _hoisted_1 = { class: "text-lg text-fg-subtle font-mono" }
const _hoisted_2 = { class: "font-mono text-sm sm:text-base font-medium text-fg group-hover:text-fg transition-colors", dir: "ltr" }
const _hoisted_3 = { class: "text-xs px-1.5 py-0.5 rounded bg-bg-muted border border-border text-fg-muted font-mono" }
const _hoisted_4 = { class: "text-xs sm:text-sm text-fg-muted mt-0.5" }
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:arrow-right rtl-flip w-4 h-4 text-fg-subtle group-hover:text-fg transition-colors shrink-0", "aria-hidden": "true" })

export default /*@__PURE__*/_defineComponent({
  __name: 'SearchSuggestionCard',
  props: {
    type: { type: String, required: true },
    name: { type: String, required: true },
    isExactMatch: { type: Boolean, required: false },
    index: { type: Number, required: false }
  },
  setup(__props: any) {


return (_ctx: any,_cache: any) => {
  const _component_NuxtLink = _resolveComponent("NuxtLink")
  const _component_BaseCard = _resolveComponent("BaseCard")

  return (_openBlock(), _createBlock(_component_BaseCard, { isExactMatch: __props.isExactMatch }, {
      default: _withCtx(() => [
        _createVNode(_component_NuxtLink, {
          to: 
          __props.type === 'user'
            ? { name: '~username', params: { username: __props.name.toLowerCase() } }
            : { name: 'org', params: { org: __props.name.toLowerCase() } }
        ,
          "data-suggestion-index": __props.index,
          class: "flex items-center gap-4 focus-visible:outline-none after:content-[''] after:absolute after:inset-0"
        }, {
          default: _withCtx(() => [
            _createElementVNode("div", {
              class: _normalizeClass(["w-10 h-10 shrink-0 flex items-center justify-center border border-border", __props.type === 'org' ? 'rounded-lg bg-bg-muted' : 'rounded-full bg-bg-muted']),
              "aria-hidden": "true"
            }, [
              _createElementVNode("span", _hoisted_1, _toDisplayString(__props.name.charAt(0).toUpperCase()), 1 /* TEXT */)
            ], 2 /* CLASS */),
            _createElementVNode("div", { class: "min-w-0 flex-1" }, [
              _createElementVNode("div", { class: "flex items-center gap-2" }, [
                _createElementVNode("span", _hoisted_2, _toDisplayString(__props.type === 'user' ? '~' : '@') + _toDisplayString(__props.name), 1 /* TEXT */),
                _createElementVNode("span", _hoisted_3, _toDisplayString(__props.type === 'user' ? _ctx.$t('search.suggestion.user') : _ctx.$t('search.suggestion.org')), 1 /* TEXT */),
                _createTextVNode("\n          " + "\n          "),
                (__props.isExactMatch)
                  ? (_openBlock(), _createElementBlock("span", {
                    key: 0,
                    class: "text-xs px-1.5 py-0.5 rounded bg-accent/20 border border-accent/30 text-accent font-mono"
                  }, _toDisplayString(_ctx.$t('search.exact_match')), 1 /* TEXT */))
                  : _createCommentVNode("v-if", true)
              ]),
              _createElementVNode("p", _hoisted_4, _toDisplayString(__props.type === 'user'
                ? _ctx.$t('search.suggestion.view_user_packages')
                : _ctx.$t('search.suggestion.view_org_packages')), 1 /* TEXT */)
            ]),
            _hoisted_5
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["to", "data-suggestion-index"])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["isExactMatch"]))
}
}

})
