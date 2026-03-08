import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createTextVNode as _createTextVNode, renderList as _renderList, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"

import { ref, computed, markRaw } from 'vue'
import XMessage from './chat/XMessage.vue'
import { i18n } from '@/i18n.js'
import MkInfo from '@/components/MkInfo.vue'
import { Paginator } from '@/utility/paginator.js'
import MkPagination from '@/components/MkPagination.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'admin-file.chat',
  props: {
    fileId: { type: String, required: true }
  },
  setup(__props: any) {

const props = __props
const realFileId = computed(() => props.fileId);
const paginator = markRaw(new Paginator('drive/files/attached-chat-messages', {
	limit: 10,
	params: {
		fileId: realFileId.value,
	},
}));

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", { class: "_gaps" }, [ _createVNode(MkInfo, null, {
        default: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts._fileViewer.thisPageCanBeSeenFromTheAuthor), 1 /* TEXT */)
        ]),
        _: 1 /* STABLE */
      }), _createVNode(MkPagination, { paginator: _unref(paginator) }, {
        default: _withCtx(({ items }) => [
          (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (item) => {
            return (_openBlock(), _createBlock(XMessage, {
              key: item.id,
              message: item,
              isSearchResult: true
            }, null, 8 /* PROPS */, ["message", "isSearchResult"]))
          }), 128 /* KEYED_FRAGMENT */))
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["paginator"]) ]))
}
}

})
