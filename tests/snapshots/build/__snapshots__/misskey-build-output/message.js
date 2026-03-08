import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, withCtx as _withCtx } from "vue"

import { ref, onMounted } from 'vue'
import * as Misskey from 'misskey-js'
import XMessage from './XMessage.vue'
import { i18n } from '@/i18n.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { definePage } from '@/page.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'message',
  props: {
    messageId: { type: String, required: true }
  },
  setup(__props: any) {

const props = __props
const initializing = ref(true);
const message = ref<Misskey.entities.ChatMessage | null>();
async function initialize() {
	initializing.value = true;
	message.value = await misskeyApi('chat/messages/show', {
		messageId: props.messageId,
	});
	initializing.value = false;
}
onMounted(() => {
	initialize();
});
definePage({
	title: i18n.ts.directMessage,
});

return (_ctx: any,_cache: any) => {
  const _component_MkLoading = _resolveComponent("MkLoading")
  const _component_PageWithHeader = _resolveComponent("PageWithHeader")

  return (_openBlock(), _createBlock(_component_PageWithHeader, null, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-w: 700px;"
        }, [
          (initializing.value || message.value == null)
            ? (_openBlock(), _createElementBlock("div", { key: 0 }, [
              _createVNode(_component_MkLoading)
            ]))
            : (_openBlock(), _createElementBlock("div", { key: 1 }, [
              _createVNode(XMessage, {
                message: message.value,
                isSearchResult: true
              }, null, 8 /* PROPS */, ["message", "isSearchResult"])
            ]))
        ])
      ]),
      _: 1 /* STABLE */
    }))
}
}

})
