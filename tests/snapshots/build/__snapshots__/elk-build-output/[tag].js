import { withAsyncContext as _withAsyncContext, defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, renderSlot as _renderSlot, toDisplayString as _toDisplayString, mergeProps as _mergeProps, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = { "text-lg": "true", "font-bold": "true" }
import type { mastodon } from 'masto'

export default /*@__PURE__*/_defineComponent({
  __name: '[tag]',
  async setup(__props) {

let __temp: any, __restore: any

definePageMeta({
  name: 'tag',
})
const params = useRoute().params
const tagName = computed(() => params.tag as string)
const { client } = useMasto()
const { data: tag, refresh } = await useAsyncData(() => `tag-${tagName.value}`, () => client.value.v1.tags.$select(tagName.value).fetch(), { default: () => shallowRef() })
const paginator = client.value.v1.timelines.tag.$select(tagName.value).list()
const stream = useStreaming(client => client.hashtag.subscribe({ tag: tagName.value }))
if (tag.value) {
  useHydratedHead({
    title: () => `#${tag.value.name}`,
  })
}
onReactivated(() => {
  // Silently update data when reentering the page
  // The user will see the previous content first, and any changes will be updated to the UI when the request is completed
  refresh()
})
let followedTags: mastodon.v1.Tag[]
if (currentUser.value !== undefined) {
  const { client } = useMasto()
  const paginator = client.value.v1.followedTags.list()
  followedTags = (await paginator.values().next()).value ?? []
}

return (_ctx: any,_cache: any) => {
  const _component_TagActionButton = _resolveComponent("TagActionButton")
  const _component_TimelinePaginator = _resolveComponent("TimelinePaginator")
  const _component_MainContent = _resolveComponent("MainContent")

  return (_openBlock(), _createBlock(_component_MainContent, { back: "" }, {
      title: _withCtx(() => [
        _createElementVNode("bdi", _hoisted_1, "#" + _toDisplayString(tagName.value), 1 /* TEXT */)
      ]),
      actions: _withCtx(() => [
        (typeof _unref(tag)?.following === 'boolean')
          ? (_openBlock(), _createBlock(_component_TagActionButton, {
            key: 0,
            tag: _unref(tag),
            onChange: _cache[0] || (_cache[0] = ($event: any) => (_unref(refresh)()))
          }, null, 8 /* PROPS */, ["tag"]))
          : _createCommentVNode("v-if", true)
      ]),
      default: _withCtx(() => [
        _renderSlot(_ctx.$slots, "default", {}, () => [
          _createVNode(_component_TimelinePaginator, _mergeProps({ paginator: _unref(paginator), stream: _unref(stream) }, {
            "followed-tags": _unref(followedTags),
            context: "public"
          }), null, 16 /* FULL_PROPS */, ["followed-tags"])
        ])
      ]),
      _: 1 /* STABLE */
    }))
}
}

})
