import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, withDirectives as _withDirectives, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, unref as _unref, vModelText as _vModelText, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-photo-plus" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-mood-happy" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-send" })
import { onMounted, watch, ref, shallowRef, computed, nextTick, readonly, onBeforeUnmount } from 'vue'
import * as Misskey from 'misskey-js'
import { formatTimeString } from '@/utility/format-time-string.js'
import { selectFile } from '@/utility/drive.js'
import * as os from '@/os.js'
import { i18n } from '@/i18n.js'
import { miLocalStorage } from '@/local-storage.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { prefer } from '@/preferences.js'
import { Autocomplete } from '@/utility/autocomplete.js'
import { emojiPicker } from '@/utility/emoji-picker.js'
import { checkDragDataType, getDragData } from '@/drag-and-drop.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'room.form',
  props: {
    user: { type: null, required: false },
    room: { type: null, required: false }
  },
  setup(__props: any) {

const props = __props
//import insertTextAtCursor from 'insert-text-at-cursor';
const textareaEl = shallowRef<HTMLTextAreaElement>();
const fileEl = shallowRef<HTMLInputElement>();
const text = ref<string>('');
const file = ref<Misskey.entities.DriveFile | null>(null);
const sending = ref(false);
const textareaReadOnly = ref(false);
let autocompleteInstance: Autocomplete | null = null;
const canSend = computed(() => (text.value != null && text.value !== '') || file.value != null);
function getDraftKey() {
	return props.user ? 'user:' + props.user.id : 'room:' + props.room?.id;
}
watch([text, file], saveDraft);
async function onPaste(ev: ClipboardEvent) {
	if (!ev.clipboardData) return;
	const pastedFileName = 'yyyy-MM-dd HH-mm-ss [{{number}}]';
	const clipboardData = ev.clipboardData;
	const items = clipboardData.items;
	if (items.length === 1) {
		if (items[0].kind === 'file') {
			const pastedFile = items[0].getAsFile();
			if (!pastedFile) return;
			const lio = pastedFile.name.lastIndexOf('.');
			const ext = lio >= 0 ? pastedFile.name.slice(lio) : '';
			const formattedName = formatTimeString(new Date(pastedFile.lastModified), pastedFileName).replace(/{{number}}/g, '1') + ext;
			const renamedFile = new File([pastedFile], formattedName, { type: pastedFile.type });
			os.launchUploader([renamedFile], { multiple: false }).then(driveFiles => {
				file.value = driveFiles[0];
			});
		}
	} else {
		if (items[0].kind === 'file') {
			os.alert({
				type: 'error',
				text: i18n.ts.onlyOneFileCanBeAttached,
			});
		}
	}
}
function onDragover(ev: DragEvent) {
	if (!ev.dataTransfer) return;
	const isFile = ev.dataTransfer.items[0].kind === 'file';
	if (isFile || checkDragDataType(ev, ['driveFiles'])) {
		ev.preventDefault();
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
	}
}
function onDrop(ev: DragEvent): void {
	if (!ev.dataTransfer) return;
	// ファイルだったら
	if (ev.dataTransfer.files.length === 1) {
		ev.preventDefault();
		os.launchUploader([Array.from(ev.dataTransfer.files)[0]], { multiple: false });
		return;
	} else if (ev.dataTransfer.files.length > 1) {
		ev.preventDefault();
		os.alert({
			type: 'error',
			text: i18n.ts.onlyOneFileCanBeAttached,
		});
		return;
	}
	//#region ドライブのファイル
	{
		const droppedData = getDragData(ev, 'driveFiles');
		if (droppedData != null) {
			file.value = droppedData[0];
			ev.preventDefault();
		}
	}
	//#endregion
}
function onKeydown(ev: KeyboardEvent) {
	if (ev.key === 'Enter') {
		if (prefer.s['chat.sendOnEnter']) {
			if (!(ev.ctrlKey || ev.metaKey || ev.shiftKey)) {
				send();
			}
		} else {
			if ((ev.ctrlKey || ev.metaKey)) {
				send();
			}
		}
	}
}
function chooseFile(ev: PointerEvent) {
	selectFile({
		anchorElement: ev.currentTarget ?? ev.target,
		multiple: false,
		label: i18n.ts.selectFile,
	}).then(selectedFile => {
		file.value = selectedFile;
	});
}
function onChangeFile() {
	if (fileEl.value == null || fileEl.value.files == null) return;
	if (fileEl.value.files[0]) {
		os.launchUploader(Array.from(fileEl.value.files), { multiple: false }).then(driveFiles => {
			file.value = driveFiles[0];
		});
	}
}
function send() {
	if (!canSend.value) return;
	sending.value = true;
	if (props.user) {
		misskeyApi('chat/messages/create-to-user', {
			toUserId: props.user.id,
			text: text.value ? text.value : undefined,
			fileId: file.value ? file.value.id : undefined,
		}).then(message => {
			clear();
		}).catch(err => {
			console.error(err);
		}).then(() => {
			sending.value = false;
		});
	} else if (props.room) {
		misskeyApi('chat/messages/create-to-room', {
			toRoomId: props.room.id,
			text: text.value ? text.value : undefined,
			fileId: file.value ? file.value.id : undefined,
		}).then(message => {
			clear();
		}).catch(err => {
			console.error(err);
		}).then(() => {
			sending.value = false;
		});
	}
}
function clear() {
	text.value = '';
	file.value = null;
	deleteDraft();
}
function saveDraft() {
	const drafts = JSON.parse(miLocalStorage.getItem('chatMessageDrafts') || '{}');
	drafts[getDraftKey()] = {
		updatedAt: new Date(),
		data: {
			text: text.value,
			file: file.value,
		},
	};
	miLocalStorage.setItem('chatMessageDrafts', JSON.stringify(drafts));
}
function deleteDraft() {
	const drafts = JSON.parse(miLocalStorage.getItem('chatMessageDrafts') || '{}');
	delete drafts[getDraftKey()];
	miLocalStorage.setItem('chatMessageDrafts', JSON.stringify(drafts));
}
async function insertEmoji(ev: MouseEvent) {
	textareaReadOnly.value = true;
	const target = ev.currentTarget ?? ev.target;
	if (target == null) return;
	// emojiPickerはダイアログが閉じずにtextareaとやりとりするので、
	// focustrapをかけているとinsertTextAtCursorが効かない
	// そのため、投稿フォームのテキストに直接注入する
	// See: https://github.com/misskey-dev/misskey/pull/14282
	//      https://github.com/misskey-dev/misskey/issues/14274
	let pos = textareaEl.value?.selectionStart ?? 0;
	let posEnd = textareaEl.value?.selectionEnd ?? text.value.length;
	emojiPicker.show(
		target as HTMLElement,
		emoji => {
			const textBefore = text.value.substring(0, pos);
			const textAfter = text.value.substring(posEnd);
			text.value = textBefore + emoji + textAfter;
			pos += emoji.length;
			posEnd += emoji.length;
		},
		() => {
			textareaReadOnly.value = false;
			nextTick(() => focus());
		},
	);
}
onMounted(() => {
	if (textareaEl.value != null) {
		autocompleteInstance = new Autocomplete(textareaEl.value, text);
	}
	// 書きかけの投稿を復元
	const draft = JSON.parse(miLocalStorage.getItem('chatMessageDrafts') || '{}')[getDraftKey()];
	if (draft) {
		text.value = draft.data.text;
		file.value = draft.data.file;
	}
});
onBeforeUnmount(() => {
	if (autocompleteInstance) {
		autocompleteInstance.detach();
		autocompleteInstance = null;
	}
});

return (_ctx: any,_cache: any) => {
  const _component_MkLoading = _resolveComponent("MkLoading")

  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(_ctx.$style.root),
      onDragover: _withModifiers(onDragover, ["stop"]),
      onDrop: _withModifiers(onDrop, ["stop"])
    }, [ _withDirectives(_createElementVNode("textarea", {
        ref_key: "textareaEl", ref: textareaEl,
        "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((text).value = $event)),
        class: _normalizeClass(["_acrylic", _ctx.$style.textarea]),
        placeholder: _unref(i18n).ts.inputMessageHere,
        readonly: textareaReadOnly.value,
        onKeydown: onKeydown,
        onPaste: onPaste
      }, null, 40 /* PROPS, NEED_HYDRATION */, ["placeholder", "readonly"]), [ [_vModelText, text.value] ]), _createElementVNode("footer", {
        class: _normalizeClass(_ctx.$style.footer)
      }, [ (file.value) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: _normalizeClass(_ctx.$style.file),
            onClick: _cache[1] || (_cache[1] = ($event: any) => (file.value = null))
          }, _toDisplayString(file.value.name), 1 /* TEXT */)) : _createCommentVNode("v-if", true), _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.buttons)
        }, [ _createElementVNode("button", {
            class: _normalizeClass(["_button", _ctx.$style.button]),
            onClick: chooseFile
          }, [ _hoisted_1 ]), _createElementVNode("button", {
            class: _normalizeClass(["_button", _ctx.$style.button]),
            onClick: insertEmoji
          }, [ _hoisted_2 ]), _createElementVNode("button", {
            class: _normalizeClass(["_button", [_ctx.$style.button, _ctx.$style.send]]),
            disabled: !canSend.value || sending.value,
            title: _unref(i18n).ts.send,
            onClick: send
          }, [ (!sending.value) ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ _hoisted_3 ], 64 /* STABLE_FRAGMENT */)) : _createCommentVNode("v-if", true), (sending.value) ? (_openBlock(), _createBlock(_component_MkLoading, {
                key: 0,
                em: true
              }, null, 8 /* PROPS */, ["em"])) : _createCommentVNode("v-if", true) ], 8 /* PROPS */, ["disabled", "title"]) ]) ]), _createElementVNode("input", {
        ref_key: "fileEl", ref: fileEl,
        style: "display: none;",
        type: "file",
        onChange: onChangeFile
      }, null, 32 /* NEED_HYDRATION */) ], 32 /* NEED_HYDRATION */))
}
}

})
