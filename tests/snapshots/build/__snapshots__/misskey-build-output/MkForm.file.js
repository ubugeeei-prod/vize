import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"

import * as Misskey from 'misskey-js'
import { computed, ref } from 'vue'
import { i18n } from '@/i18n.js'
import MkButton from '@/components/MkButton.vue'
import { selectFile } from '@/utility/drive.js'
import { misskeyApi } from '@/utility/misskey-api.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkForm.file',
  props: {
    fileId: { type: String, required: false },
    validate: { type: Function, required: false }
  },
  emits: ["update"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const fileUrl = ref('');
const fileName = ref<string>('');
const friendlyFileName = computed<string>(() => {
	if (fileName.value) {
		return fileName.value;
	}
	if (fileUrl.value) {
		return fileUrl.value;
	}

	return i18n.ts.fileNotSelected;
});
if (props.fileId) {
	misskeyApi('drive/files/show', {
		fileId: props.fileId,
	}).then((apiRes) => {
		fileName.value = apiRes.name;
		fileUrl.value = apiRes.url;
	});
}
function selectButton(ev: PointerEvent) {
	selectFile({
		anchorElement: ev.currentTarget ?? ev.target,
		multiple: false,
	}).then(async (file) => {
		if (!file) return;
		if (props.validate && !await props.validate(file)) return;
		emit('update', file);
		fileName.value = file.name;
		fileUrl.value = file.url;
	});
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", null, [ _createVNode(MkButton, {
        inline: "",
        rounded: "",
        primary: "",
        onClick: _cache[0] || (_cache[0] = ($event: any) => (selectButton($event)))
      }, {
        default: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts.selectFile), 1 /* TEXT */)
        ]),
        _: 1 /* STABLE */
      }), _createElementVNode("div", {
        class: _normalizeClass(['_nowrap', !fileName.value && _ctx.$style.fileNotSelected])
      }, _toDisplayString(friendlyFileName.value), 3 /* TEXT, CLASS */) ]))
}
}

})
