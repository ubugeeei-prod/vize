import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, unref as _unref, withModifiers as _withModifiers } from "vue"

import { computed, ref } from 'vue'
import * as Misskey from 'misskey-js'
import MkDriveFileThumbnail from '@/components/MkDriveFileThumbnail.vue'
import bytes from '@/filters/bytes.js'
import * as os from '@/os.js'
import { i18n } from '@/i18n.js'
import { $i } from '@/i.js'
import { getDriveFileMenu } from '@/utility/get-drive-file-menu.js'
import { setDragData } from '@/drag-and-drop.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkDrive.file',
  props: {
    file: { type: null, required: true },
    folder: { type: null, required: true },
    isSelected: { type: Boolean, required: false, default: false }
  },
  emits: ["dragstart", "dragend"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const isDragging = ref(false);
const title = computed(() => `${props.file.name}\n${props.file.type} ${bytes(props.file.size)}`);
function onContextmenu(ev: PointerEvent) {
	os.contextMenu(getDriveFileMenu(props.file, props.folder), ev);
}
function onDragstart(ev: DragEvent) {
	if (ev.dataTransfer) {
		ev.dataTransfer.effectAllowed = 'move';
		setDragData(ev, 'driveFiles', [props.file]);
	}
	isDragging.value = true;
	emit('dragstart', ev);
}
function onDragend() {
	isDragging.value = false;
	emit('dragend');
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass([_ctx.$style.root, { [_ctx.$style.isSelected]: __props.isSelected }]),
      draggable: "true",
      title: title.value,
      onContextmenu: _withModifiers(onContextmenu, ["stop"]),
      onDragstart: onDragstart,
      onDragend: onDragend
    }, [ _createElementVNode("div", { style: "pointer-events: none;" }, [ (_unref($i)?.avatarId == __props.file.id) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: _normalizeClass([_ctx.$style.label])
          }, [ _createElementVNode("img", {
              class: _normalizeClass(_ctx.$style.labelImg),
              src: "/client-assets/label.svg"
            }), _createElementVNode("p", {
              class: _normalizeClass(_ctx.$style.labelText)
            }, _toDisplayString(_unref(i18n).ts.avatar), 1 /* TEXT */) ])) : _createCommentVNode("v-if", true), (_unref($i)?.bannerId == __props.file.id) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: _normalizeClass([_ctx.$style.label])
          }, [ _createElementVNode("img", {
              class: _normalizeClass(_ctx.$style.labelImg),
              src: "/client-assets/label.svg"
            }), _createElementVNode("p", {
              class: _normalizeClass(_ctx.$style.labelText)
            }, _toDisplayString(_unref(i18n).ts.banner), 1 /* TEXT */) ])) : _createCommentVNode("v-if", true), (__props.file.isSensitive) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: _normalizeClass([_ctx.$style.label, _ctx.$style.red])
          }, [ _createElementVNode("img", {
              class: _normalizeClass(_ctx.$style.labelImg),
              src: "/client-assets/label-red.svg"
            }), _createElementVNode("p", {
              class: _normalizeClass(_ctx.$style.labelText)
            }, _toDisplayString(_unref(i18n).ts.sensitive), 1 /* TEXT */) ])) : _createCommentVNode("v-if", true), _createVNode(MkDriveFileThumbnail, {
          class: _normalizeClass(_ctx.$style.thumbnail),
          file: __props.file,
          fit: "contain"
        }, null, 8 /* PROPS */, ["file"]), _createElementVNode("p", {
          class: _normalizeClass(_ctx.$style.name)
        }, [ _createElementVNode("span", null, _toDisplayString(__props.file.name.lastIndexOf('.') != -1 ? __props.file.name.substring(0, __props.file.name.lastIndexOf('.')) : __props.file.name), 1 /* TEXT */), (__props.file.name.lastIndexOf('.') != -1) ? (_openBlock(), _createElementBlock("span", {
              key: 0,
              style: "opacity: 0.5;"
            }, _toDisplayString(__props.file.name.substring(__props.file.name.lastIndexOf('.'))), 1 /* TEXT */)) : _createCommentVNode("v-if", true) ]) ]) ], 42 /* CLASS, PROPS, NEED_HYDRATION */, ["title"]))
}
}

})
