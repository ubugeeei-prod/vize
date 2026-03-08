import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"

import { ref, computed, markRaw } from 'vue'
import { i18n } from '@/i18n.js'
import MkInfo from '@/components/MkInfo.vue'
import MkNotesTimeline from '@/components/MkNotesTimeline.vue'
import { Paginator } from '@/utility/paginator.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'drive.file.notes',
  props: {
    fileId: { type: String, required: true }
  },
  setup(__props: any) {

const props = __props
const realFileId = computed(() => props.fileId);
const paginator = markRaw(new Paginator('drive/files/attached-notes', {
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
      }), _createVNode(MkNotesTimeline, { paginator: _unref(paginator) }, null, 8 /* PROPS */, ["paginator"]) ]))
}
}

})
