import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, toDisplayString as _toDisplayString, unref as _unref } from "vue"

import * as Misskey from 'misskey-js'
import { toUnicode } from 'punycode.js'
import { host as hostRaw } from '@@/js/config.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkAcct',
  props: {
    user: { type: null, required: true },
    detail: { type: Boolean, required: false }
  },
  setup(__props: any) {

const host = toUnicode(hostRaw);

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("span", null, [ _createElementVNode("span", null, "@" + _toDisplayString(__props.user.username), 1 /* TEXT */), (__props.user.host || __props.detail) ? (_openBlock(), _createElementBlock("span", {
          key: 0,
          style: "opacity: 0.5;"
        }, "@" + _toDisplayString(__props.user.host || _unref(host)), 1 /* TEXT */)) : _createCommentVNode("v-if", true) ]))
}
}

})
