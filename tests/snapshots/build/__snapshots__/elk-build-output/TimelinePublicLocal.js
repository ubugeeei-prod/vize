import { withAsyncContext as _withAsyncContext, defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, resolveComponent as _resolveComponent, mergeProps as _mergeProps, unref as _unref } from "vue"

import type { mastodon } from 'masto'

export default /*@__PURE__*/_defineComponent({
  __name: 'TimelinePublicLocal',
  async setup(__props) {

let __temp: any, __restore: any

const paginator = useMastoClient().v1.timelines.public.list({ limit: 30, local: true })
const stream = useStreaming(client => client.public.local.subscribe())
function preprocess(items: mastodon.v1.Status[]) {
  return filterAndReorderTimeline(items, 'public')
}
let followedTags: mastodon.v1.Tag[]
if (currentUser.value !== undefined) {
  const { client } = useMasto()
  const paginator = client.value.v1.followedTags.list()
  followedTags = (await paginator.values().next()).value ?? []
}

return (_ctx: any,_cache: any) => {
  const _component_TimelinePaginator = _resolveComponent("TimelinePaginator")

  return (_openBlock(), _createElementBlock("div", null, [ _createVNode(_component_TimelinePaginator, _mergeProps({ paginator: _unref(paginator), stream: _unref(stream) }, {
        "followed-tags": _unref(followedTags),
        preprocess: preprocess,
        context: "public"
      }), null, 16 /* FULL_PROPS */, ["followed-tags", "preprocess"]) ]))
}
}

})
