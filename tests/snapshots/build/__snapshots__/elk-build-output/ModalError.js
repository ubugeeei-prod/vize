import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, renderList as _renderList, toDisplayString as _toDisplayString } from "vue"


const _hoisted_1 = { "font-bold": "true", "text-lg": "true", "text-center": "true" }
import type { ErrorDialogData } from '#shared/types'

export default /*@__PURE__*/_defineComponent({
  __name: 'ModalError',
  setup(__props: any) {


return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      flex: "~ col",
      "gap-6": ""
    }, [ _createElementVNode("div", _hoisted_1, _toDisplayString(_ctx.title), 1 /* TEXT */), _createElementVNode("div", {
        flex: "~ col",
        "gap-1": "",
        "text-sm": "",
        "pt-1": "",
        "ps-2": "",
        "pe-1": "",
        "pb-2": "",
        "text-red-600": "",
        "dark:text-red-400": "",
        border: "~ base rounded red-600 dark:red-400"
      }, [ _createElementVNode("ol", {
          "ps-2": "",
          "sm:ps-1": ""
        }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_ctx.messages, (message, i) => {
            return (_openBlock(), _createElementBlock("li", {
              key: i,
              flex: "~ col sm:row",
              "gap-y-1": "",
              "sm:gap-x-2": ""
            }, _toDisplayString(message), 1 /* TEXT */))
          }), 128 /* KEYED_FRAGMENT */)) ]) ]), _createElementVNode("div", {
        flex: "",
        "justify-end": "",
        "gap-2": ""
      }, [ _createElementVNode("button", {
          "btn-text": "",
          onClick: _cache[0] || (_cache[0] = ($event: any) => (_ctx.closeErrorDialog()))
        }, _toDisplayString(_ctx.close), 1 /* TEXT */) ]) ]))
}
}

})
