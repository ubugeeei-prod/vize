import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-dice-5" })
import * as Misskey from 'misskey-js'
import { i18n } from '@/i18n.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'page.dynamic',
  props: {
    block: { type: null, required: true },
    page: { type: null, required: true }
  },
  setup(__props: any) {

const props = __props

return (_ctx: any,_cache: any) => {
  const _component_MkA = _resolveComponent("MkA")
  const _component_I18n = _resolveComponent("I18n")

  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(_ctx.$style.root)
    }, [ _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.heading)
      }, [ _hoisted_1, _createTextVNode(" " + _toDisplayString(_unref(i18n).ts._pages.blocks.dynamic), 1 /* TEXT */) ]), _createVNode(_component_I18n, {
        src: _unref(i18n).ts._pages.blocks.dynamicDescription,
        tag: "div",
        class: _normalizeClass(_ctx.$style.text)
      }, {
        play: _withCtx(() => [
          _createVNode(_component_MkA, {
            to: "/play",
            class: "_link"
          }, {
            default: _withCtx(() => [
              _createTextVNode("Play")
            ]),
            _: 1 /* STABLE */
          })
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["src"]) ]))
}
}

})
