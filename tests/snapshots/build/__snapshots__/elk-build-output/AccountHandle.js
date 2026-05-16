import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, toDisplayString as _toDisplayString } from "vue"


const _hoisted_1 = { "text-secondary": "true" }
import type { mastodon } from 'masto'

export default /*@__PURE__*/_defineComponent({
  __name: 'AccountHandle',
  props: {
    account: { type: null, required: true }
  },
  setup(__props: any) {

const serverName = computed(() => getServerName(__props.account))

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("p", {
      "line-clamp-1": "",
      "whitespace-pre-wrap": "",
      "break-all": "",
      "text-secondary-light": "",
      "leading-tight": "",
      dir: "ltr"
    }, [ _createElementVNode("span", _hoisted_1, _toDisplayString(_ctx.getShortHandle(__props.account)), 1 /* TEXT */), (serverName.value) ? (_openBlock(), _createElementBlock("span", {
          key: 0,
          "text-secondary-light": ""
        }, "@" + _toDisplayString(serverName.value), 1 /* TEXT */)) : _createCommentVNode("v-if", true) ]))
}
}

})
