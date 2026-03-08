import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, renderSlot as _renderSlot, normalizeClass as _normalizeClass } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-x" })

export default /*@__PURE__*/_defineComponent({
  __name: 'MkInfo',
  props: {
    warn: { type: Boolean, required: false },
    closable: { type: Boolean, required: false }
  },
  emits: ["close"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
function close() {
	// こいつの中では非表示動作は行わない
	emit('close');
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(["_selectable", [_ctx.$style.root, { [_ctx.$style.warn]: __props.warn }]])
    }, [ (__props.warn) ? (_openBlock(), _createElementBlock("i", {
          key: 0,
          class: _normalizeClass(["ti ti-alert-triangle", _ctx.$style.i])
        })) : (_openBlock(), _createElementBlock("i", {
          key: 1,
          class: _normalizeClass(["ti ti-info-circle", _ctx.$style.i])
        })), _createElementVNode("div", null, [ _renderSlot(_ctx.$slots, "default") ]), (__props.closable) ? (_openBlock(), _createElementBlock("button", {
          key: 0,
          class: _normalizeClass(["_button", _ctx.$style.button]),
          onClick: _cache[0] || (_cache[0] = ($event: any) => (close()))
        }, [ _hoisted_1 ])) : _createCommentVNode("v-if", true) ], 2 /* CLASS */))
}
}

})
