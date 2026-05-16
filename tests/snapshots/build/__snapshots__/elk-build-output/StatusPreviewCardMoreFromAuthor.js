import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, resolveComponent as _resolveComponent } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span", { "z-0": "true" }, "More from")
import type { mastodon } from 'masto'

export default /*@__PURE__*/_defineComponent({
  __name: 'StatusPreviewCardMoreFromAuthor',
  props: {
    account: { type: null, required: true }
  },
  setup(__props: any) {


return (_ctx: any,_cache: any) => {
  const _component_AccountInlineInfo = _resolveComponent("AccountInlineInfo")

  return (_openBlock(), _createElementBlock("div", {
      "max-h-2xl": "",
      flex: "",
      "gap-2": "",
      "my-auto": "",
      "p-4": "",
      "py-2": "",
      "light:bg-gray-3": "",
      "dark:bg-gray-8": ""
    }, [ _hoisted_1, _createVNode(_component_AccountInlineInfo, {
        account: __props.account,
        "hover:bg-inherit": "",
        "ps-0": "",
        "ms-0": ""
      }, null, 8 /* PROPS */, ["account"]) ]))
}
}

})
