import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, resolveComponent as _resolveComponent, unref as _unref } from "vue"


export default /*@__PURE__*/_defineComponent({
  __name: 'TimelineConversations',
  setup(__props) {

const paginator = useMastoClient().v1.conversations.list()

return (_ctx: any,_cache: any) => {
  const _component_ConversationPaginator = _resolveComponent("ConversationPaginator")

  return (_openBlock(), _createBlock(_component_ConversationPaginator, { paginator: _unref(paginator) }, null, 8 /* PROPS */, ["paginator"]))
}
}

})
