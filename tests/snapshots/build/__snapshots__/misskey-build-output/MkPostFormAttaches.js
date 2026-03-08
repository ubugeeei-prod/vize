import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, withDirectives as _withDirectives, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, vShow as _vShow, withModifiers as _withModifiers, withKeys as _withKeys } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-eye-exclamation", style: "margin: auto;" })
import { inject } from 'vue'
import * as Misskey from 'misskey-js'
import type { MenuItem } from '@/types/menu'
import { copyToClipboard } from '@/utility/copy-to-clipboard'
import MkDriveFileThumbnail from '@/components/MkDriveFileThumbnail.vue'
import MkDraggable from '@/components/MkDraggable.vue'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { i18n } from '@/i18n.js'
import { prefer } from '@/preferences.js'
import { DI } from '@/di.js'
import { globalEvents } from '@/events.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkPostFormAttaches',
  props: {
    modelValue: { type: Array, required: true },
    detachMediaFn: { type: Function, required: false }
  },
  emits: ["update:modelValue", "detach", "changeSensitive", "changeName"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const mock = inject(DI.mock, false);
let menuShowing = false;
function detachMedia(id: string) {
	if (mock) return;
	if (props.detachMediaFn) {
		props.detachMediaFn(id);
	} else {
		emit('detach', id);
	}
}
async function detachAndDeleteMedia(file: Misskey.entities.DriveFile) {
	if (mock) return;
	detachMedia(file.id);
	const { canceled } = await os.confirm({
		type: 'warning',
		text: i18n.tsx.driveFileDeleteConfirm({ name: file.name }),
	});
	if (canceled) return;
	await os.apiWithDialog('drive/files/delete', {
		fileId: file.id,
	});
	globalEvents.emit('driveFilesDeleted', [file]);
}
function toggleSensitive(file: Misskey.entities.DriveFile) {
	if (mock) {
		emit('changeSensitive', file, !file.isSensitive);
		return;
	}
	misskeyApi('drive/files/update', {
		fileId: file.id,
		isSensitive: !file.isSensitive,
	}).then(() => {
		emit('changeSensitive', file, !file.isSensitive);
	});
}
async function rename(file: Misskey.entities.DriveFile) {
	if (mock) return;
	const { canceled, result } = await os.inputText({
		title: i18n.ts.enterFileName,
		default: file.name,
		minLength: 1,
	});
	if (canceled) return;
	misskeyApi('drive/files/update', {
		fileId: file.id,
		name: result,
	}).then(() => {
		emit('changeName', file, result);
		file.name = result;
	});
}
async function describe(file: Misskey.entities.DriveFile) {
	if (mock) return;
	const { dispose } = await os.popupAsyncWithDialog(import('@/components/MkFileCaptionEditWindow.vue').then(x => x.default), {
		default: file.comment !== null ? file.comment : '',
		file: file,
	}, {
		done: caption => {
			let comment = caption.length === 0 ? null : caption;
			misskeyApi('drive/files/update', {
				fileId: file.id,
				comment: comment,
			}).then(() => {
				file.comment = comment;
			});
		},
		closed: () => dispose(),
	});
}
function showFileMenu(file: Misskey.entities.DriveFile, ev: PointerEvent | KeyboardEvent): void {
	if (menuShowing) return;
	const isImage = file.type.startsWith('image/');
	const menuItems: MenuItem[] = [];
	menuItems.push({
		text: i18n.ts.renameFile,
		icon: 'ti ti-forms',
		action: () => { rename(file); },
	}, {
		text: file.isSensitive ? i18n.ts.unmarkAsSensitive : i18n.ts.markAsSensitive,
		icon: file.isSensitive ? 'ti ti-eye-exclamation' : 'ti ti-eye',
		action: () => { toggleSensitive(file); },
	}, {
		text: i18n.ts.describeFile,
		icon: 'ti ti-text-caption',
		action: () => { describe(file); },
	});
	if (isImage) {
		menuItems.push({
			text: i18n.ts.preview,
			icon: 'ti ti-photo-search',
			action: async () => {
				const { dispose } = await os.popupAsyncWithDialog(import('@/components/MkImgPreviewDialog.vue').then(x => x.default), {
					file: file,
				}, {
					closed: () => dispose(),
				});
			},
		});
	}
	menuItems.push({
		type: 'divider',
	}, {
		text: i18n.ts.attachCancel,
		icon: 'ti ti-circle-x',
		action: () => { detachMedia(file.id); },
	}, {
		text: i18n.ts.deleteFile,
		icon: 'ti ti-trash',
		danger: true,
		action: () => { detachAndDeleteMedia(file); },
	});
	if (prefer.s.devMode) {
		menuItems.push({ type: 'divider' }, {
			icon: 'ti ti-hash',
			text: i18n.ts.copyFileId,
			action: () => {
				copyToClipboard(file.id);
			},
		});
	}
	os.popupMenu(menuItems, ev.currentTarget ?? ev.target).then(() => menuShowing = false);
	menuShowing = true;
}

return (_ctx: any,_cache: any) => {
  return _withDirectives((_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(_ctx.$style.root)
    }, [ _createVNode(MkDraggable, {
        modelValue: props.modelValue,
        class: _normalizeClass(_ctx.$style.files),
        direction: "horizontal",
        withGaps: "",
        "onUpdate:modelValue": _cache[0] || (_cache[0] = (v) => emit("update:modelValue", v))
      }, {
        default: _withCtx(({ item }) => [
          _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.file),
            role: "button",
            tabindex: "0",
            onClick: _cache[1] || (_cache[1] = ($event: any) => (showFileMenu(_ctx.item, $event))),
            onKeydown: _cache[2] || (_cache[2] = _withKeys(($event: any) => (showFileMenu(_ctx.item, $event)), ["space","enter"])),
            onContextmenu: _cache[3] || (_cache[3] = _withModifiers(($event: any) => (showFileMenu(_ctx.item, $event)), ["prevent","stop"]))
          }, [
            _createVNode(MkDriveFileThumbnail, {
              style: "pointer-events: none;",
              "data-id": item.id,
              class: _normalizeClass(_ctx.$style.thumbnail),
              file: item,
              fit: "cover"
            }, null, 8 /* PROPS */, ["data-id", "file"]),
            (item.isSensitive)
              ? (_openBlock(), _createElementBlock("div", {
                key: 0,
                class: _normalizeClass(_ctx.$style.sensitive),
                style: "pointer-events: none;"
              }, [
                _hoisted_1
              ]))
              : _createCommentVNode("v-if", true)
          ], 32 /* NEED_HYDRATION */)
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["modelValue"]), _createElementVNode("p", {
        class: _normalizeClass([_ctx.$style.remain, {
  			[_ctx.$style.exceeded]: props.modelValue.length > 16,
  		}])
      }, _toDisplayString(props.modelValue.length) + "/16\n\t", 3 /* TEXT, CLASS */) ], 512 /* NEED_PATCH */)), [ [_vShow, props.modelValue.length != 0] ])
}
}

})
