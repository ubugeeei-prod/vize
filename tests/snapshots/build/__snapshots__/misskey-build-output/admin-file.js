import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, withCtx as _withCtx } from "vue"

import { ref } from 'vue'
import * as Misskey from 'misskey-js'
import XRoot from './admin-file.root.vue'
import { misskeyApi } from '@/utility/misskey-api.js'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'admin-file',
  props: {
    fileId: { type: String, required: true }
  },
  setup(__props: any) {

const props = __props
function _fetch_() {
	return Promise.all([
		misskeyApi('drive/files/show', { fileId: props.fileId }),
		misskeyApi('admin/drive/show-file', { fileId: props.fileId }),
	]).then((result) => ({
		file: result[0],
		info: result[1],
	}));
}
const file = ref<Misskey.entities.DriveFile | null>(null);
definePage(() => ({
	title: file.value ? `${i18n.ts.file}: ${file.value.name}` : i18n.ts.file,
	icon: 'ti ti-file',
}));

return (_ctx: any,_cache: any) => {
  const _component_MkSuspense = _resolveComponent("MkSuspense")

  return (_openBlock(), _createBlock(_component_MkSuspense, {
      p: _fetch_,
      onResolved: _cache[0] || (_cache[0] = (result) => file.value = result.file)
    }, {
      default: _withCtx(({ result }) => [
        (result.file != null && result.info != null)
          ? (_openBlock(), _createBlock(XRoot, {
            key: 0,
            file: result.file,
            info: result.info
          }, null, 8 /* PROPS */, ["file", "info"]))
          : _createCommentVNode("v-if", true)
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["p"]))
}
}

})
