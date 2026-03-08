import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, resolveComponent as _resolveComponent, unref as _unref } from "vue"


export default /*@__PURE__*/_defineComponent({
  __name: 'TimelineBlocks',
  setup(__props) {

const paginator = useMastoClient().v1.blocks.list()

return (_ctx: any,_cache: any) => {
  const _component_AccountPaginator = _resolveComponent("AccountPaginator")

  return (_openBlock(), _createBlock(_component_AccountPaginator, { paginator: _unref(paginator) }, null, 8 /* PROPS */, ["paginator"]))
}
}

})
