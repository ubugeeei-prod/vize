import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementVNode as _createElementVNode, resolveComponent as _resolveComponent, withCtx as _withCtx } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("div", { "i-ri:hashtag": "true" })

export default /*@__PURE__*/_defineComponent({
  __name: 'Hashtag',
  props: {
    activeClass: { type: String, required: true }
  },
  setup(__props: any) {


return (_ctx: any,_cache: any) => {
  const _component_NuxtLink = _resolveComponent("NuxtLink")

  return (_openBlock(), _createBlock(_component_NuxtLink, {
      to: "/hashtags",
      "aria-label": _ctx.$t('nav.hashtags'),
      "active-class": __props.activeClass,
      flex: "",
      "flex-row": "",
      "items-center": "",
      "place-content-center": "",
      "h-full": "",
      "flex-1": "",
      class: "coarse-pointer:select-none",
      onClick: _cache[0] || (_cache[0] = (...args) => (_ctx.$scrollToTop && _ctx.$scrollToTop(...args)))
    }, {
      default: _withCtx(() => [
        _hoisted_1
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["aria-label", "active-class"]))
}
}

})
