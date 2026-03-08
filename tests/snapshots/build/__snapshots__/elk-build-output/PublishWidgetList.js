import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, renderList as _renderList, mergeProps as _mergeProps, unref as _unref } from "vue"

import type { DraftItem } from '#shared/types'
import type { mastodon } from 'masto'

export default /*@__PURE__*/_defineComponent({
  __name: 'PublishWidgetList',
  props: {
    draftKey: { type: null, required: true },
    initial: { type: Function, required: false, default: getDefaultDraftItem },
    placeholder: { type: String, required: false },
    inReplyToId: { type: String, required: false },
    inReplyToVisibility: { type: null, required: false },
    expanded: { type: Boolean, required: false, default: false },
    dialogLabelledBy: { type: String, required: false }
  },
  setup(__props: any) {

const threadComposer = useThreadComposer(__props.draftKey, __props.initial)
const threadItems = computed(() => threadComposer.threadItems.value)
onDeactivated(() => {
  clearEmptyDrafts()
})
function isFirstItem(index: number) {
  return index === 0
}

return (_ctx: any,_cache: any) => {
  const _component_PublishWidget = _resolveComponent("PublishWidget")

  return (_ctx.isHydrated && _ctx.currentUser)
      ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(threadItems.value, (_, index) => {
          return (_openBlock(), _createBlock(_component_PublishWidget, _mergeProps(_ctx.$attrs, {
            key: `${__props.draftKey}-${index}`,
            "thread-composer": _unref(threadComposer),
            "draft-key": __props.draftKey,
            "draft-item-index": index,
            expanded: isFirstItem(index) ? __props.expanded : true,
            placeholder: __props.placeholder,
            "dialog-labelled-by": __props.dialogLabelledBy,
            "in-reply-to-id": isFirstItem(index) ? __props.inReplyToId : undefined,
            "in-reply-to-visibility": __props.inReplyToVisibility
          }), null, 16 /* FULL_PROPS */, ["thread-composer", "draft-key", "draft-item-index", "expanded", "placeholder", "dialog-labelled-by", "in-reply-to-id", "in-reply-to-visibility"]))
        }), 128 /* KEYED_FRAGMENT */)) ], 64 /* STABLE_FRAGMENT */))
      : _createCommentVNode("v-if", true)
}
}

})
