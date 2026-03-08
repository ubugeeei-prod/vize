import { withAsyncContext as _withAsyncContext, defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, resolveComponent as _resolveComponent, mergeProps as _mergeProps, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("div", { h: "1px", "w-auto": "true", "bg-border": "true", "mb-3": "true" })
import type { mastodon } from 'masto'

export default /*@__PURE__*/_defineComponent({
  __name: 'TimelineHome',
  async setup(__props) {

let __temp: any, __restore: any

const { isSupported, effectiveType } = useNetwork()
const isSlow = computed(() => isSupported.value && effectiveType.value && ['slow-2g', '2g', '3g'].includes(effectiveType.value))
const limit = computed(() => isSlow.value ? 10 : 30)
const paginator = useMastoClient().v1.timelines.home.list({ limit: limit.value })
const stream = useStreaming(client => client.user.subscribe())
function preprocess(items: mastodon.v1.Status[]) {
  return filterAndReorderTimeline(items, 'home')
}
let followedTags: mastodon.v1.Tag[]
if (currentUser.value !== undefined) {
  const { client } = useMasto()
  const paginator = client.value.v1.followedTags.list()
  followedTags = (await paginator.values().next()).value ?? []
}

return (_ctx: any,_cache: any) => {
  const _component_PublishWidgetList = _resolveComponent("PublishWidgetList")
  const _component_TimelinePaginator = _resolveComponent("TimelinePaginator")

  return (_openBlock(), _createElementBlock("div", null, [ _createVNode(_component_PublishWidgetList, { "draft-key": "home" }), _hoisted_1, _createVNode(_component_TimelinePaginator, _mergeProps({ paginator: _unref(paginator), stream: _unref(stream) }, {
        "followed-tags": _unref(followedTags),
        preprocess: preprocess,
        context: "home"
      }), null, 16 /* FULL_PROPS */, ["followed-tags", "preprocess"]) ]))
}
}

})
