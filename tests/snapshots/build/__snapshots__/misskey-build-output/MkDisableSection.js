import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, renderSlot as _renderSlot, normalizeClass as _normalizeClass } from "vue"


export default /*@__PURE__*/_defineComponent({
  __name: 'MkDisableSection',
  props: {
    disabled: { type: Boolean, required: false }
  },
  setup(__props: any) {


return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass([_ctx.$style.root])
    }, [ _createElementVNode("div", {
        inert: __props.disabled,
        class: _normalizeClass([{ [_ctx.$style.disabled]: __props.disabled }])
      }, [ _renderSlot(_ctx.$slots, "default") ], 10 /* CLASS, PROPS */, ["inert"]), (__props.disabled) ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          class: _normalizeClass([_ctx.$style.cover])
        })) : _createCommentVNode("v-if", true) ]))
}
}

})
