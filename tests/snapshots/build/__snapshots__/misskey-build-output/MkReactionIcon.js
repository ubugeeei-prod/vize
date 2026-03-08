import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent } from "vue"

import { defineAsyncComponent, useTemplateRef } from 'vue'
import { useTooltip } from '@/composables/use-tooltip.js'
import * as os from '@/os.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkReactionIcon',
  props: {
    reaction: { type: String, required: true },
    noStyle: { type: Boolean, required: false },
    emojiUrl: { type: String, required: false },
    withTooltip: { type: Boolean, required: false }
  },
  setup(__props: any) {

const props = __props
const elRef = useTemplateRef('elRef');
if (props.withTooltip) {
	useTooltip(elRef, (showing) => {
		if (elRef.value == null) return;
		const { dispose } = os.popup(defineAsyncComponent(() => import('@/components/MkReactionTooltip.vue')), {
			showing,
			reaction: props.reaction.replace(/^:(\w+):$/, ':$1@.:'),
			anchorElement: elRef.value.$el,
		}, {
			closed: () => dispose(),
		});
	});
}

return (_ctx: any,_cache: any) => {
  const _component_MkCustomEmoji = _resolveComponent("MkCustomEmoji")
  const _component_MkEmoji = _resolveComponent("MkEmoji")

  return (__props.reaction[0] === ':')
      ? (_openBlock(), _createBlock(_component_MkCustomEmoji, {
        key: 0,
        ref: "elRef",
        name: __props.reaction,
        normal: true,
        noStyle: __props.noStyle,
        url: __props.emojiUrl,
        fallbackToImage: true
      }, null, 8 /* PROPS */, ["name", "normal", "noStyle", "url", "fallbackToImage"]))
      : (_openBlock(), _createBlock(_component_MkEmoji, {
        key: 1,
        ref: "elRef",
        emoji: __props.reaction,
        normal: true,
        noStyle: __props.noStyle
      }, null, 8 /* PROPS */, ["emoji", "normal", "noStyle"]))
}
}

})
