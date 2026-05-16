import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, renderList as _renderList, renderSlot as _renderSlot, toDisplayString as _toDisplayString, normalizeStyle as _normalizeStyle } from "vue"

import type { mastodon } from 'masto'

export default /*@__PURE__*/_defineComponent({
  __name: 'AccountRolesIndicator',
  props: {
    account: { type: null, required: true },
    limit: { type: Number, required: false }
  },
  setup(__props: any) {


return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock(_Fragment, null, [ _createElementVNode("div", {
        flex: "~ gap1",
        "items-center": "",
        class: "border border-base rounded-md px-1",
        "text-secondary-light": ""
      }, [ _renderSlot(_ctx.$slots, "prepend"), (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(__props.account.roles?.slice(0, __props.limit), (role) => {
          return (_openBlock(), _createElementBlock("div", {
            key: role.id,
            flex: ""
          }, [
            _createElementVNode("div", {
              style: _normalizeStyle(`color: ${role.color}; border-color: ${role.color}`)
            }, _toDisplayString(role.name), 5 /* TEXT, STYLE */)
          ]))
        }), 128 /* KEYED_FRAGMENT */)) ]), (__props.limit && __props.account.roles?.length > __props.limit) ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          flex: "~ gap1",
          "items-center": "",
          class: "border border-base rounded-md px-1",
          "text-secondary-light": ""
        }, "\n    +" + _toDisplayString(__props.account.roles?.length - __props.limit), 1 /* TEXT */)) : _createCommentVNode("v-if", true) ], 64 /* STABLE_FRAGMENT */))
}
}

})
