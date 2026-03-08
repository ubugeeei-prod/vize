import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-alert-triangle", style: "margin-right: 8px;" })
import { i18n } from '@/i18n.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkRemoteCaution',
  props: {
    href: { type: String, required: false }
  },
  setup(__props: any) {


return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(_ctx.$style.root)
    }, [ _hoisted_1, _createTextVNode(_toDisplayString(_unref(i18n).ts.remoteUserCaution), 1 /* TEXT */), (__props.href) ? (_openBlock(), _createElementBlock("a", {
          key: 0,
          class: _normalizeClass(_ctx.$style.link),
          href: __props.href,
          rel: "nofollow noopener",
          target: "_blank"
        }, _toDisplayString(_unref(i18n).ts.showOnRemote), 1 /* TEXT */)) : _createCommentVNode("v-if", true) ]))
}
}

})
