import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, normalizeClass as _normalizeClass } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("circle", { cx: "64", cy: "64", r: "64", style: "fill:none;stroke:currentColor;stroke-width:21.33px;" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("path", { d: "M128,64C128,28.654 99.346,0 64,0C99.346,0 128,28.654 128,64Z", style: "fill:none;stroke:currentColor;stroke-width:21.33px;" })

export default /*@__PURE__*/_defineComponent({
  __name: 'MkLoading',
  props: {
    static: { type: Boolean, required: false, default: false },
    inline: { type: Boolean, required: false, default: false },
    colored: { type: Boolean, required: false, default: true },
    mini: { type: Boolean, required: false, default: false },
    em: { type: Boolean, required: false, default: false }
  },
  setup(__props: any) {

const props = __props

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass([_ctx.$style.root, { [_ctx.$style.inline]: __props.inline, [_ctx.$style.colored]: __props.colored, [_ctx.$style.mini]: __props.mini, [_ctx.$style.em]: __props.em }])
    }, [ _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.container)
      }, [ _createElementVNode("svg", {
          class: _normalizeClass([_ctx.$style.spinner, _ctx.$style.bg]),
          viewBox: "0 0 168 168",
          xmlns: "http://www.w3.org/2000/svg"
        }, [ _createElementVNode("g", { transform: "matrix(1.125,0,0,1.125,12,12)" }, [ _hoisted_1 ]) ]), _createElementVNode("svg", {
          class: _normalizeClass([_ctx.$style.spinner, _ctx.$style.fg, { [_ctx.$style.static]: __props.static }]),
          viewBox: "0 0 168 168",
          xmlns: "http://www.w3.org/2000/svg"
        }, [ _createElementVNode("g", { transform: "matrix(1.125,0,0,1.125,12,12)" }, [ _hoisted_2 ]) ], 2 /* CLASS */) ]) ], 2 /* CLASS */))
}
}

})
