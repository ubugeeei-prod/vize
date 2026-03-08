import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-pencil" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-eye" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-eye-exclamation" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-download" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-trash" })
import { ref, computed, defineAsyncComponent, onMounted } from 'vue'
import * as Misskey from 'misskey-js'
import MkInfo from '@/components/MkInfo.vue'
import MkMediaList from '@/components/MkMediaList.vue'
import MkKeyValue from '@/components/MkKeyValue.vue'
import bytes from '@/filters/bytes.js'
import { i18n } from '@/i18n.js'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { useRouter } from '@/router.js'
import { selectDriveFolder } from '@/utility/drive.js'
import { globalEvents } from '@/events.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'drive.file.info',
  props: {
    fileId: { type: String, required: true }
  },
  setup(__props: any) {

const props = __props
const router = useRouter();
const fetching = ref(true);
const file = ref<Misskey.entities.DriveFile>();
const folderHierarchy = computed(() => {
	if (!file.value) return [i18n.ts.drive];
	const folderNames = [i18n.ts.drive];

	function get(folder: Misskey.entities.DriveFolder) {
		if (folder.parent) get(folder.parent);
		folderNames.push(folder.name);
	}

	if (file.value.folder) get(file.value.folder);
	return folderNames;
});
const isImage = computed(() => file.value?.type.startsWith('image/'));
async function _fetch_() {
	fetching.value = true;
	file.value = await misskeyApi('drive/files/show', {
		fileId: props.fileId,
	}).catch((err) => {
		console.error(err);
		return undefined;
	});
	fetching.value = false;
}
function postThis() {
	if (file.value == null) return;
	os.post({
		initialFiles: [file.value],
	});
}
function move() {
	if (file.value == null) return;
	const f = file.value;
	selectDriveFolder(null).then(({ canceled, folders }) => {
		if (canceled) return;
		misskeyApi('drive/files/update', {
			fileId: f.id,
			folderId: folders[0] ? folders[0].id : null,
		}).then(async () => {
			await _fetch_();
		});
	});
}
function toggleSensitive() {
	if (file.value == null) return;
	os.apiWithDialog('drive/files/update', {
		fileId: file.value.id,
		isSensitive: !file.value.isSensitive,
	}).then(async () => {
		await _fetch_();
	}).catch(err => {
		os.alert({
			type: 'error',
			title: i18n.ts.error,
			text: err.message,
		});
	});
}
function rename() {
	if (file.value == null) return;
	const f = file.value;
	os.inputText({
		title: i18n.ts.renameFile,
		placeholder: i18n.ts.inputNewFileName,
		default: file.value.name,
	}).then(({ canceled, result: name }) => {
		if (canceled) return;
		os.apiWithDialog('drive/files/update', {
			fileId: f.id,
			name: name,
		}).then(async () => {
			await _fetch_();
		});
	});
}
async function describe() {
	if (file.value == null) return;
	const f = file.value;
	const { dispose } = await os.popupAsyncWithDialog(import('@/components/MkFileCaptionEditWindow.vue').then(x => x.default), {
		default: file.value.comment ?? '',
		file: file.value,
	}, {
		done: caption => {
			os.apiWithDialog('drive/files/update', {
				fileId: f.id,
				comment: caption.length === 0 ? null : caption,
			}).then(async () => {
				await _fetch_();
			});
		},
		closed: () => dispose(),
	});
}
async function deleteFile() {
	if (file.value == null) return;
	const { canceled } = await os.confirm({
		type: 'warning',
		text: i18n.tsx.driveFileDeleteConfirm({ name: file.value.name }),
	});
	if (canceled) return;
	await os.apiWithDialog('drive/files/delete', {
		fileId: file.value.id,
	});
	globalEvents.emit('driveFilesDeleted', [file.value]);
	router.push('/my/drive');
}
onMounted(async () => {
	await _fetch_();
});

return (_ctx: any,_cache: any) => {
  const _component_MkLoading = _resolveComponent("MkLoading")
  const _component_MkTime = _resolveComponent("MkTime")
  const _component_MkResult = _resolveComponent("MkResult")
  const _directive_tooltip = _resolveDirective("tooltip")

  return (_openBlock(), _createElementBlock("div", { class: "_gaps" }, [ _createVNode(MkInfo, null, {
        default: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts._fileViewer.thisPageCanBeSeenFromTheAuthor), 1 /* TEXT */)
        ]),
        _: 1 /* STABLE */
      }), (fetching.value) ? (_openBlock(), _createBlock(_component_MkLoading, { key: 0 })) : (file.value) ? (_openBlock(), _createElementBlock("div", {
            key: 1,
            class: "_gaps"
          }, [ _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.filePreviewRoot)
            }, [ _createVNode(MkMediaList, { mediaList: [file.value] }, null, 8 /* PROPS */, ["mediaList"]) ]), _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.fileQuickActionsRoot)
            }, [ _createElementVNode("button", {
                class: _normalizeClass(["_button", _ctx.$style.fileNameEditBtn]),
                onClick: _cache[0] || (_cache[0] = ($event: any) => (rename()))
              }, [ _createElementVNode("h2", {
                  class: _normalizeClass(["_nowrap", _ctx.$style.fileName])
                }, _toDisplayString(file.value.name), 1 /* TEXT */), _createElementVNode("i", {
                  class: _normalizeClass(["ti ti-pencil", _ctx.$style.fileNameEditIcon])
                }) ]), _createElementVNode("div", {
                class: _normalizeClass(_ctx.$style.fileQuickActionsOthers)
              }, [ _createElementVNode("button", {
                  class: _normalizeClass(["_button", _ctx.$style.fileQuickActionsOthersButton]),
                  onClick: _cache[1] || (_cache[1] = ($event: any) => (postThis()))
                }, [ _hoisted_1 ]), (file.value.isSensitive) ? (_openBlock(), _createElementBlock("button", {
                    key: 0,
                    class: _normalizeClass(["_button", _ctx.$style.fileQuickActionsOthersButton]),
                    onClick: _cache[2] || (_cache[2] = ($event: any) => (toggleSensitive()))
                  }, [ _hoisted_2 ])) : (_openBlock(), _createElementBlock("button", {
                    key: 1,
                    class: _normalizeClass(["_button", _ctx.$style.fileQuickActionsOthersButton]),
                    onClick: _cache[3] || (_cache[3] = ($event: any) => (toggleSensitive()))
                  }, [ _hoisted_3 ])), _createElementVNode("a", {
                  href: file.value.url,
                  download: file.value.name,
                  class: _normalizeClass(["_button", _ctx.$style.fileQuickActionsOthersButton])
                }, [ _hoisted_4 ], 8 /* PROPS */, ["href", "download"]), _createElementVNode("button", {
                  class: _normalizeClass(["_button", [_ctx.$style.fileQuickActionsOthersButton, _ctx.$style.danger]]),
                  onClick: _cache[4] || (_cache[4] = ($event: any) => (deleteFile()))
                }, [ _hoisted_5 ]) ]) ]), _createElementVNode("div", { class: "_gaps_s" }, [ _createElementVNode("button", {
                class: _normalizeClass(["_button", _ctx.$style.kvEditBtn]),
                onClick: _cache[5] || (_cache[5] = ($event: any) => (move()))
              }, [ _createVNode(MkKeyValue, null, {
                  key: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.folder), 1 /* TEXT */)
                  ]),
                  value: _withCtx(() => [
                    _createTextVNode(_toDisplayString(folderHierarchy.value.join(' > ')), 1 /* TEXT */),
                    _createElementVNode("i", {
                      class: _normalizeClass(["ti ti-pencil", _ctx.$style.kvEditIcon])
                    })
                  ]),
                  _: 1 /* STABLE */
                }) ]), _createElementVNode("button", {
                class: _normalizeClass(["_button", _ctx.$style.kvEditBtn]),
                onClick: _cache[6] || (_cache[6] = ($event: any) => (describe()))
              }, [ _createVNode(MkKeyValue, {
                  class: _normalizeClass(_ctx.$style.multiline)
                }, {
                  key: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.description), 1 /* TEXT */)
                  ]),
                  value: _withCtx(() => [
                    _createTextVNode(_toDisplayString(file.value.comment ? file.value.comment : `(${_unref(i18n).ts.none})`), 1 /* TEXT */),
                    _createElementVNode("i", {
                      class: _normalizeClass(["ti ti-pencil", _ctx.$style.kvEditIcon])
                    })
                  ]),
                  _: 1 /* STABLE */
                }) ]), _createVNode(MkKeyValue, {
                class: _normalizeClass(_ctx.$style.fileMetaDataChildren)
              }, {
                key: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._fileViewer.uploadedAt), 1 /* TEXT */)
                ]),
                value: _withCtx(() => [
                  _createVNode(_component_MkTime, {
                    time: file.value.createdAt,
                    mode: "detail"
                  }, null, 8 /* PROPS */, ["time"])
                ]),
                _: 1 /* STABLE */
              }), _createVNode(MkKeyValue, {
                class: _normalizeClass(_ctx.$style.fileMetaDataChildren)
              }, {
                key: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._fileViewer.type), 1 /* TEXT */)
                ]),
                value: _withCtx(() => [
                  _createTextVNode(_toDisplayString(file.value.type), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }), _createVNode(MkKeyValue, {
                class: _normalizeClass(_ctx.$style.fileMetaDataChildren)
              }, {
                key: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._fileViewer.size), 1 /* TEXT */)
                ]),
                value: _withCtx(() => [
                  _createTextVNode(_toDisplayString(bytes(file.value.size)), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }), _createVNode(MkKeyValue, {
                class: _normalizeClass(_ctx.$style.fileMetaDataChildren),
                copy: file.value.url
              }, {
                key: _withCtx(() => [
                  _createTextVNode("URL")
                ]),
                value: _withCtx(() => [
                  _createTextVNode(_toDisplayString(file.value.url), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["copy"]) ]) ])) : (_openBlock(), _createBlock(_component_MkResult, {
          key: 2,
          type: "empty"
        })) ]))
}
}

})
