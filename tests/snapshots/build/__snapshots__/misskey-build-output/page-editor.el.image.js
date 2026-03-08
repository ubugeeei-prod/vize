import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-photo" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-folder" })
import { onMounted, ref } from 'vue'
import * as Misskey from 'misskey-js'
import XContainer from '../page-editor.container.vue'
import MkDriveFileThumbnail from '@/components/MkDriveFileThumbnail.vue'
import { misskeyApi } from '@/utility/misskey-api.js'
import { i18n } from '@/i18n.js'
import { chooseDriveFile } from '@/utility/drive.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'page-editor.el.image',
  props: {
    modelValue: { type: null, required: true }
  },
  emits: ["update:modelValue", "image", "remove"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const file = ref<Misskey.entities.DriveFile | null>(null);
async function choose() {
	chooseDriveFile({ multiple: false }).then((fileResponse) => {
		file.value = fileResponse[0];
		emit('update:modelValue', {
			...props.modelValue,
			fileId: file.value.id,
		});
	});
}
onMounted(async () => {
	if (props.modelValue.fileId == null) {
		await choose();
	} else {
		misskeyApi('drive/files/show', {
			fileId: props.modelValue.fileId,
		}).then(fileResponse => {
			file.value = fileResponse;
		});
	}
});

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createBlock(XContainer, {
      draggable: true,
      onRemove: _cache[0] || (_cache[0] = () => emit('remove'))
    }, {
      header: _withCtx(() => [
        _hoisted_1,
        _createTextVNode(" "),
        _createTextVNode(_toDisplayString(_unref(i18n).ts._pages.blocks.image), 1 /* TEXT */)
      ]),
      func: _withCtx(() => [
        _createElementVNode("button", {
          onClick: _cache[1] || (_cache[1] = ($event: any) => (choose()))
        }, [
          _hoisted_2
        ])
      ]),
      default: _withCtx(() => [
        _createElementVNode("section", null, [
          (file.value)
            ? (_openBlock(), _createBlock(MkDriveFileThumbnail, {
              key: 0,
              style: "height: 150px;",
              file: file.value,
              fit: "contain",
              onClick: _cache[2] || (_cache[2] = ($event: any) => (choose()))
            }, null, 8 /* PROPS */, ["file"]))
            : _createCommentVNode("v-if", true)
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["draggable"]))
}
}

})
