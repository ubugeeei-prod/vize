import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveDirective as _resolveDirective, renderSlot as _renderSlot, normalizeClass as _normalizeClass, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-copy" })
import { copyToClipboard } from '@/utility/copy-to-clipboard.js'
import * as os from '@/os.js'
import { i18n } from '@/i18n.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkKeyValue',
  props: {
    copy: { type: String, required: false, default: null },
    oneline: { type: Boolean, required: false, default: false }
  },
  setup(__props: any) {

const props = __props
const copy_ = () => {
	copyToClipboard(props.copy);
};

return (_ctx: any,_cache: any) => {
  const _directive_tooltip = _resolveDirective("tooltip")

  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass([_ctx.$style.root, { [_ctx.$style.oneline]: __props.oneline }])
    }, [ _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.key)
      }, [ _renderSlot(_ctx.$slots, "key") ]), _createElementVNode("div", {
        class: _normalizeClass(["_selectable", _ctx.$style.value])
      }, [ _renderSlot(_ctx.$slots, "value"), (__props.copy) ? (_openBlock(), _createElementBlock("button", {
            key: 0,
            class: "_textButton",
            style: "margin-left: 0.5em;",
            onClick: copy_
          }, [ _hoisted_1 ])) : _createCommentVNode("v-if", true) ]) ], 2 /* CLASS */))
}
}

})
