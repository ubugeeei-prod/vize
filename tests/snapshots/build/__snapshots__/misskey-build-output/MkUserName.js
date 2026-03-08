import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, resolveComponent as _resolveComponent } from "vue"

import * as Misskey from 'misskey-js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkUserName',
  props: {
    user: { type: null, required: true },
    nowrap: { type: Boolean, required: false, default: true }
  },
  setup(__props: any) {

const props = __props

return (_ctx: any,_cache: any) => {
  const _component_Mfm = _resolveComponent("Mfm")

  return (_openBlock(), _createBlock(_component_Mfm, {
      text: __props.user.name ?? __props.user.username,
      author: __props.user,
      plain: true,
      nowrap: __props.nowrap,
      emojiUrls: __props.user.emojis
    }, null, 8 /* PROPS */, ["text", "author", "plain", "nowrap", "emojiUrls"]))
}
}

})
