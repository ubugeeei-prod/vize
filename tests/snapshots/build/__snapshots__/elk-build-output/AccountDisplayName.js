import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, resolveComponent as _resolveComponent } from "vue"

import type { mastodon } from 'masto'

export default /*@__PURE__*/_defineComponent({
  __name: 'AccountDisplayName',
  props: {
    account: { type: null, required: true },
    hideEmojis: { type: Boolean, required: false, default: false }
  },
  setup(__props: any) {


return (_ctx: any,_cache: any) => {
  const _component_ContentRich = _resolveComponent("ContentRich")

  return (_openBlock(), _createBlock(_component_ContentRich, {
      content: _ctx.getDisplayName(__props.account, { rich: true }),
      emojis: __props.account.emojis,
      "hide-emojis": __props.hideEmojis,
      markdown: false
    }, null, 8 /* PROPS */, ["content", "emojis", "hide-emojis", "markdown"]))
}
}

})
