import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, resolveComponent as _resolveComponent, normalizeClass as _normalizeClass, withCtx as _withCtx } from "vue"

import type { mastodon } from 'masto'

export default /*@__PURE__*/_defineComponent({
  __name: 'TagActionButton',
  props: {
    tag: { type: null, required: true }
  },
  emits: ["change"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const { client } = useMasto()
async function toggleFollowTag() {
  // We save the state so be can do an optimistic UI update, but fallback to the previous state if the API call fails
  const previousFollowingState = __props.tag.following
  // eslint-disable-next-line vue/no-mutating-props
  __props.tag.following = !__props.tag.following
  try {
    if (previousFollowingState)
      await client.value.v1.tags.$select(__props.tag.name).unfollow()
    else
      await client.value.v1.tags.$select(__props.tag.name).follow()
    emit('change')
  }
  catch {
    // eslint-disable-next-line vue/no-mutating-props
    __props.tag.following = previousFollowingState
  }
}

return (_ctx: any,_cache: any) => {
  const _component_CommonTooltip = _resolveComponent("CommonTooltip")

  return (_openBlock(), _createElementBlock("button", {
      rounded: "",
      group: "",
      "focus:outline-none": "",
      "hover:text-primary": "",
      "focus-visible:text-primary": "",
      "aria-label": __props.tag.following ? _ctx.$t('tag.unfollow_label', [__props.tag.name]) : _ctx.$t('tag.follow_label', [__props.tag.name]),
      onClick: _cache[0] || (_cache[0] = ($event: any) => (toggleFollowTag()))
    }, [ _createVNode(_component_CommonTooltip, {
        placement: "bottom",
        content: __props.tag.following ? _ctx.$t('tag.unfollow') : _ctx.$t('tag.follow')
      }, {
        default: _withCtx(() => [
          _createElementVNode("div", {
            "rounded-full": "",
            p2: "",
            "elk-group-hover": "bg-orange/10",
            "group-focus-visible": "bg-orange/10",
            "group-focus-visible:ring": "2 current"
          }, [
            _createElementVNode("div", {
              class: _normalizeClass([__props.tag.following ? 'i-ri:star-fill' : 'i-ri:star-line'])
            }, null, 2 /* CLASS */)
          ])
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["content"]) ], 8 /* PROPS */, ["aria-label"]))
}
}

})
