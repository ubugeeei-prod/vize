import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, resolveComponent as _resolveComponent, withCtx as _withCtx } from "vue"

import type { mastodon } from 'masto'

export default /*@__PURE__*/_defineComponent({
  __name: 'ConversationPaginator',
  props: {
    paginator: { type: null, required: true }
  },
  setup(__props: any) {

function preprocess(items: mastodon.v1.Conversation[]): mastodon.v1.Conversation[] {
  const isAuthored = (conversation: mastodon.v1.Conversation) => conversation.lastStatus ? conversation.lastStatus.account.id === currentUser.value?.account.id : false
  return items.filter(item => isAuthored(item) || !item.lastStatus?.filtered?.find(
    filter => filter.filter.filterAction === 'hide' && filter.filter.context.includes('thread'),
  ))
}

return (_ctx: any,_cache: any) => {
  const _component_ConversationCard = _resolveComponent("ConversationCard")
  const _component_CommonPaginator = _resolveComponent("CommonPaginator")

  return (_openBlock(), _createBlock(_component_CommonPaginator, {
      paginator: __props.paginator,
      preprocess: preprocess
    }, {
      default: _withCtx(({ item }) => [
        _createVNode(_component_ConversationCard, {
          conversation: item,
          border: "b base",
          "py-1": ""
        }, null, 8 /* PROPS */, ["conversation"])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["paginator", "preprocess"]))
}
}

})
