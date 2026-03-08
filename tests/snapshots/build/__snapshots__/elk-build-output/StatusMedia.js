import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, resolveComponent as _resolveComponent, renderList as _renderList } from "vue"

import type { mastodon } from 'masto'

export default /*@__PURE__*/_defineComponent({
  __name: 'StatusMedia',
  props: {
    status: { type: null, required: true },
    fullSize: { type: Boolean, required: false },
    isPreview: { type: Boolean, required: false, default: false }
  },
  setup(__props: any) {

const gridColumnNumber = computed(() => {
  const num = __props.status.mediaAttachments.length
  if (num <= 1)
    return 1
  else if (num <= 4)
    return 2
  else
    return 3
})

return (_ctx: any,_cache: any) => {
  const _component_StatusAttachment = _resolveComponent("StatusAttachment")

  return (_openBlock(), _createElementBlock("div", { class: "status-media-container" }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(__props.status.mediaAttachments, (attachment) => {
        return (_openBlock(), _createElementBlock(_Fragment, { key: attachment.id }, [
          _createVNode(_component_StatusAttachment, {
            attachment: attachment,
            attachments: __props.status.mediaAttachments,
            "full-size": __props.fullSize,
            "w-full": "",
            "h-full": "",
            "is-preview": __props.isPreview
          }, null, 8 /* PROPS */, ["attachment", "attachments", "full-size", "is-preview"])
        ], 64 /* STABLE_FRAGMENT */))
      }), 128 /* KEYED_FRAGMENT */)) ]))
}
}

})
