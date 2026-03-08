import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, resolveComponent as _resolveComponent, unref as _unref } from "vue"


export default /*@__PURE__*/_defineComponent({
  __name: 'TimelineScheduledPosts',
  setup(__props) {

const paginator = useMastoClient().v1.scheduledStatuses.list()

return (_ctx: any,_cache: any) => {
  const _component_TimelineScheduledPostsPaginator = _resolveComponent("TimelineScheduledPostsPaginator")

  return (_openBlock(), _createBlock(_component_TimelineScheduledPostsPaginator, {
      "end-message": "common.no_scheduled_posts",
      paginator: _unref(paginator)
    }, null, 8 /* PROPS */, ["paginator"]))
}
}

})
