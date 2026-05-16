import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("rect", { fill: "var(--bg)", width: "512", height: "512", rx: "64" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("rect", { fill: "currentColor", x: "110", y: "310", width: "60", height: "60" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("tspan", null, "/")

export default /*@__PURE__*/_defineComponent({
  __name: 'AppLogo',
  props: {
    class: { type: String, required: false }
  },
  setup(__props: any) {


return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("svg", {
      "aria-hidden": "true",
      xmlns: "http://www.w3.org/2000/svg",
      viewBox: "0 0 512 512",
      width: "96",
      height: "96",
      class: _normalizeClass(__props.class)
    }, [ _createElementVNode("title", null, _toDisplayString(_ctx.$t('alt_logo')), 1 /* TEXT */), _hoisted_1, _hoisted_2, _createElementVNode("text", {
        fill: "var(--accent)",
        x: "320",
        y: "370",
        "font-family": "'Geist Mono',ui-monospace, SFMono-Regular, 'SF Mono', Menlo, Consolas, monospace",
        "font-size": "420",
        "font-weight": "500",
        "text-anchor": "middle",
        style: "user-select: none"
      }, [ _hoisted_3 ]) ], 2 /* CLASS */))
}
}

})
