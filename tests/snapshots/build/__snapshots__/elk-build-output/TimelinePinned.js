import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, resolveComponent as _resolveComponent, unref as _unref } from "vue"


export default /*@__PURE__*/_defineComponent({
  __name: 'TimelinePinned',
  setup(__props) {

const paginator = useMastoClient().v1.accounts.$select(currentUser.value!.account.id).statuses.list({ pinned: true })

return (_ctx: any,_cache: any) => {
  const _component_TimelinePaginator = _resolveComponent("TimelinePaginator")

  return (_openBlock(), _createBlock(_component_TimelinePaginator, { paginator: _unref(paginator) }, null, 8 /* PROPS */, ["paginator"]))
}
}

})
