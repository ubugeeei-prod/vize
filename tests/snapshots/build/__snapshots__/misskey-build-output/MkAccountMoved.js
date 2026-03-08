import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-plane-departure", style: "margin-right: 8px;" })
import { ref } from 'vue'
import * as Misskey from 'misskey-js'
import MkMention from './MkMention.vue'
import { i18n } from '@/i18n.js'
import { host as localHost } from '@@/js/config.js'
import { misskeyApi } from '@/utility/misskey-api.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkAccountMoved',
  props: {
    movedTo: { type: String, required: true }
  },
  setup(__props: any) {

const props = __props
const user = ref<Misskey.entities.UserLite>();
misskeyApi('users/show', { userId: props.movedTo }).then(u => user.value = u);

return (_ctx: any,_cache: any) => {
  return (user.value)
      ? (_openBlock(), _createElementBlock("div", {
        key: 0,
        class: _normalizeClass(_ctx.$style.root)
      }, [ _hoisted_1, _createTextVNode("\n\t"), _toDisplayString(_unref(i18n).ts.accountMoved), _createTextVNode("\n\t"), _createVNode(MkMention, {
          class: _normalizeClass(_ctx.$style.link),
          username: user.value.username,
          host: user.value.host ?? _unref(localHost)
        }, null, 8 /* PROPS */, ["username", "host"]) ]))
      : _createCommentVNode("v-if", true)
}
}

})
