import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, unref as _unref, withModifiers as _withModifiers } from "vue"

import { ref } from 'vue'
import * as Misskey from 'misskey-js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { i18n } from '@/i18n.js'
import { globalEvents } from '@/events.js'
import { checkDragDataType, getDragData } from '@/drag-and-drop.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkDrive.navFolder',
  props: {
    folder: { type: null, required: false },
    parentFolder: { type: null, required: true }
  },
  emits: ["upload"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const draghover = ref(false);
function onDragover(ev: DragEvent) {
	if (!ev.dataTransfer) return;
	// このフォルダがルートかつカレントディレクトリならドロップ禁止
	if (props.folder == null && props.parentFolder == null) {
		ev.dataTransfer.dropEffect = 'none';
	}
	const isFile = ev.dataTransfer.items[0].kind === 'file';
	if (isFile || checkDragDataType(ev, ['driveFiles', 'driveFolders'])) {
		switch (ev.dataTransfer.effectAllowed) {
			case 'all':
			case 'uninitialized':
			case 'copy':
			case 'copyLink':
			case 'copyMove':
				ev.dataTransfer.dropEffect = 'copy';
				break;
			case 'linkMove':
			case 'move':
				ev.dataTransfer.dropEffect = 'move';
				break;
			default:
				ev.dataTransfer.dropEffect = 'none';
				break;
		}
	} else {
		ev.dataTransfer.dropEffect = 'none';
	}
	return false;
}
function onDragenter() {
	if (props.folder || props.parentFolder) draghover.value = true;
}
function onDragleave() {
	if (props.folder || props.parentFolder) draghover.value = false;
}
function onDrop(ev: DragEvent) {
	draghover.value = false;
	if (!ev.dataTransfer) return;
	// ファイルだったら
	if (ev.dataTransfer.files.length > 0) {
		emit('upload', Array.from(ev.dataTransfer.files), props.folder);
		return;
	}
	//#region ドライブのファイル
	{
		const droppedData = getDragData(ev, 'driveFiles');
		if (droppedData != null) {
			misskeyApi('drive/files/move-bulk', {
				fileIds: droppedData.map(f => f.id),
				folderId: props.folder ? props.folder.id : null,
			}).then(() => {
				globalEvents.emit('driveFilesUpdated', droppedData.map(x => ({
					...x,
					folderId: props.folder ? props.folder.id : null,
					folder: props.folder ?? null,
				})));
			});
		}
	}
	//#endregion
	//#region ドライブのフォルダ
	{
		const droppedData = getDragData(ev, 'driveFolders');
		if (droppedData != null) {
			const droppedFolder = droppedData[0];
			// 移動先が自分自身ならreject
			if (props.folder && droppedFolder.id === props.folder.id) return;
			misskeyApi('drive/folders/update', {
				folderId: droppedFolder.id,
				parentId: props.folder ? props.folder.id : null,
			}).then(() => {
				globalEvents.emit('driveFoldersUpdated', [droppedFolder].map(x => ({
					...x,
					parentId: props.folder ? props.folder.id : null,
					parent: props.folder ?? null,
				})));
			});
		}
	}
	//#endregion
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass([_ctx.$style.root, { [_ctx.$style.draghover]: draghover.value }]),
      onDragover: _withModifiers(onDragover, ["prevent","stop"]),
      onDragenter: onDragenter,
      onDragleave: onDragleave,
      onDrop: _withModifiers(onDrop, ["stop"])
    }, [ (__props.folder == null) ? (_openBlock(), _createElementBlock("i", {
          key: 0,
          class: "ti ti-cloud",
          style: "margin-right: 4px;"
        })) : _createCommentVNode("v-if", true), _createElementVNode("span", null, _toDisplayString(__props.folder == null ? _unref(i18n).ts.drive : __props.folder.name), 1 /* TEXT */) ], 34 /* CLASS, NEED_HYDRATION */))
}
}

})
