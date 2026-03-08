import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"

import * as Misskey from 'misskey-js'
import { userName } from '@/filters/user.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkFlashPreview',
  props: {
    flash: { type: null, required: true }
  },
  setup(__props: any) {

const props = __props

return (_ctx: any,_cache: any) => {
  const _component_Mfm = _resolveComponent("Mfm")
  const _component_MkA = _resolveComponent("MkA")

  return (_openBlock(), _createBlock(_component_MkA, {
      to: `/play/${__props.flash.id}`,
      class: _normalizeClass(["vhpxefrk _panel", [{ gray: __props.flash.visibility === 'private' }]])
    }, {
      default: _withCtx(() => [
        _createElementVNode("article", null, [
          _createElementVNode("header", null, [
            _createElementVNode("h1", { title: __props.flash.title }, _toDisplayString(__props.flash.title), 9 /* TEXT, PROPS */, ["title"])
          ]),
          (__props.flash.summary)
            ? (_openBlock(), _createElementBlock("p", {
              key: 0,
              title: __props.flash.summary
            }, [
              _createVNode(_component_Mfm, {
                class: "summaryMfm",
                text: __props.flash.summary,
                plain: true,
                nowrap: true
              }, null, 8 /* PROPS */, ["text", "plain", "nowrap"])
            ]))
            : _createCommentVNode("v-if", true),
          _createElementVNode("footer", null, [
            _createElementVNode("img", {
              class: "icon",
              src: __props.flash.user.avatarUrl
            }, null, 8 /* PROPS */, ["src"]),
            _createElementVNode("p", null, _toDisplayString(_unref(userName)(__props.flash.user)), 1 /* TEXT */)
          ])
        ])
      ]),
      _: 1 /* STABLE */
    }, 10 /* CLASS, PROPS */, ["to"]))
}
}

})
