import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = { style: "text-align: center; padding: 0 16px;" }
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-check" })
import * as Misskey from 'misskey-js'
import { ref, reactive } from 'vue'
import { i18n } from '@/i18n.js'
import MkPostForm from '@/components/MkPostForm.vue'
import MkFolder from '@/components/MkFolder.vue'
import MkInfo from '@/components/MkInfo.vue'
import MkNote from '@/components/MkNote.vue'
import { $i } from '@/i.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkTutorialDialog.Sensitive',
  emits: ["succeeded"],
  setup(__props, { emit: __emit }) {

const emit = __emit
const onceSucceeded = ref<boolean>(false);
function doSucceeded(fileId: string, to: boolean) {
	if (fileId === exampleNote.fileIds?.[0] && to) {
		onceSucceeded.value = true;
		emit('succeeded');
	}
}
const exampleNote = reactive<Misskey.entities.Note>({
	id: '0000000000',
	createdAt: '2019-04-14T17:30:49.181Z',
	userId: '0000000001',
	user: $i!,
	text: i18n.ts._initialTutorial._howToMakeAttachmentsSensitive._exampleNote.note,
	cw: null,
	visibility: 'public',
	localOnly: false,
	reactionAcceptance: null,
	renoteCount: 0,
	repliesCount: 1,
	reactionCount: 0,
	reactions: {},
	reactionEmojis: {},
	fileIds: ['0000000002'],
	files: [{
		id: '0000000002',
		createdAt: '2019-04-14T17:30:49.181Z',
		name: 'natto_failed.webp',
		type: 'image/webp',
		md5: 'c44286cf152d0740be0ce5ad45ea85c3',
		size: 827532,
		isSensitive: false,
		blurhash: 'LXNA3TD*XAIA%1%M%gt7.TofRioz',
		properties: {
			width: 256,
			height: 256,
		},
		url: '/client-assets/tutorial/natto_failed.webp',
		thumbnailUrl: '/client-assets/tutorial/natto_failed.webp',
		comment: null,
		folderId: null,
		folder: null,
		userId: null,
		user: null,
	}],
	replyId: null,
	renoteId: null,
});

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", { class: "_gaps" }, [ _createElementVNode("div", _hoisted_1, _toDisplayString(_unref(i18n).ts._initialTutorial._howToMakeAttachmentsSensitive.description), 1 /* TEXT */), _createElementVNode("div", null, _toDisplayString(_unref(i18n).ts._initialTutorial._howToMakeAttachmentsSensitive.tryThisFile), 1 /* TEXT */), _createVNode(MkInfo, null, {
        default: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts._initialTutorial._howToMakeAttachmentsSensitive.method), 1 /* TEXT */)
        ]),
        _: 1 /* STABLE */
      }), _createVNode(MkPostForm, {
        class: _normalizeClass(_ctx.$style.exampleRoot),
        mock: true,
        autofocus: false,
        initialNote: exampleNote,
        onFileChangeSensitive: doSucceeded
      }, null, 8 /* PROPS */, ["mock", "autofocus", "initialNote"]), (onceSucceeded.value) ? (_openBlock(), _createElementBlock("div", { key: 0 }, [ _createElementVNode("b", { style: "color: var(--MI_THEME-accent);" }, [ _hoisted_2, _createTextVNode(" " + _toDisplayString(_unref(i18n).ts._initialTutorial.wellDone), 1 /* TEXT */) ]), _createTextVNode(), _toDisplayString(_unref(i18n).ts._initialTutorial._howToMakeAttachmentsSensitive.sensitiveSucceeded) ])) : _createCommentVNode("v-if", true), _createVNode(MkFolder, null, {
        label: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts.previewNoteText), 1 /* TEXT */)
        ]),
        default: _withCtx(() => [
          _createVNode(MkNote, {
            mock: true,
            note: exampleNote,
            class: _normalizeClass(_ctx.$style.exampleRoot)
          }, null, 8 /* PROPS */, ["mock", "note"])
        ]),
        _: 1 /* STABLE */
      }) ]))
}
}

})
