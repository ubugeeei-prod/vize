import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, renderSlot as _renderSlot, normalizeClass as _normalizeClass } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("path", { d: "m188.19 87.657c-1.469 2.3218-3.9315 3.8312-6.667 4.0865-2.2309-1.7379-4.9781-2.6816-7.8061-2.6815h-5.1e-4v12.702h12.702v-5.1e-4c2e-5 -1.9998-0.47213-3.9713-1.378-5.754 2.0709-1.6834 3.2732-4.2102 3.273-6.8791-6e-5 -0.49375-0.0413-0.98662-0.1235-1.4735z", "fill-rule": "evenodd", "stroke-linecap": "round", "stroke-linejoin": "round", "stroke-width": ".33225", style: "paint-order:stroke fill markers" })

export default /*@__PURE__*/_defineComponent({
  __name: 'MkFukidashi',
  props: {
    tail: { type: String, required: false, default: 'right' },
    negativeMargin: { type: Boolean, required: false, default: false },
    shadow: { type: Boolean, required: false, default: false },
    accented: { type: Boolean, required: false, default: false },
    fullWidth: { type: Boolean, required: false, default: false }
  },
  setup(__props: any) {


return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass([
  		_ctx.$style.root,
  		__props.tail === 'left' ? _ctx.$style.left : _ctx.$style.right,
  		__props.negativeMargin === true && _ctx.$style.negativeMargin,
  		__props.shadow === true && _ctx.$style.shadow,
  		__props.accented === true && _ctx.$style.accented,
  		__props.fullWidth === true && _ctx.$style.fullWidth,
  	])
    }, [ _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.bg)
      }, [ (__props.tail !== 'none') ? (_openBlock(), _createElementBlock("svg", {
            key: 0,
            class: _normalizeClass(_ctx.$style.tail),
            version: "1.1",
            viewBox: "0 0 14.597 14.58",
            xmlns: "http://www.w3.org/2000/svg"
          }, [ _createElementVNode("g", { transform: "translate(-173.71 -87.184)" }, [ _hoisted_1 ]) ])) : _createCommentVNode("v-if", true), _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.content)
        }, [ _renderSlot(_ctx.$slots, "default") ]) ]) ], 2 /* CLASS */))
}
}

})
