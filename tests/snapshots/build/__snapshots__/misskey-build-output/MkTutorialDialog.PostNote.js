import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = { style: "text-align: center; padding: 0 16px;" }
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-world" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-home" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-lock" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-mail" })
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-rocket-off" })
import * as Misskey from 'misskey-js'
import { reactive } from 'vue'
import { i18n } from '@/i18n.js'
import MkNote from '@/components/MkNote.vue'
import MkPostForm from '@/components/MkPostForm.vue'
import MkFormSection from '@/components/form/section.vue'
import MkInfo from '@/components/MkInfo.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkTutorialDialog.PostNote',
  setup(__props) {

const exampleCWNote = reactive<Misskey.entities.Note>({
	id: '0000000000',
	createdAt: '2019-04-14T17:30:49.181Z',
	userId: '0000000001',
	user: {
		id: '0000000001',
		name: '藍',
		username: 'ai',
		host: null,
		avatarDecorations: [],
		avatarUrl: '/client-assets/tutorial/ai.webp',
		avatarBlurhash: 'eiKmhHIByXxZ~qWXs:-pR*NbR*s:xuRjoL-oR*WCt6WWf6WVf6oeWB',
		isBot: false,
		isCat: true,
		emojis: {},
		onlineStatus: 'unknown',
		badgeRoles: [],
	},
	text: i18n.ts._initialTutorial._postNote._cw._exampleNote.note,
	cw: i18n.ts._initialTutorial._postNote._cw._exampleNote.cw,
	visibility: 'public',
	localOnly: false,
	reactionAcceptance: null,
	renoteCount: 0,
	repliesCount: 1,
	reactionCount: 0,
	reactions: {},
	reactionEmojis: {},
	fileIds: [],
	files: [],
	replyId: null,
	renoteId: null,
});

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", { class: "_gaps" }, [ _createElementVNode("div", _hoisted_1, _toDisplayString(_unref(i18n).ts._initialTutorial._postNote.description1), 1 /* TEXT */), _createVNode(MkPostForm, {
        class: _normalizeClass(_ctx.$style.exampleRoot),
        mock: true,
        autofocus: false
      }, null, 8 /* PROPS */, ["mock", "autofocus"]), _createVNode(MkFormSection, null, {
        label: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts.visibility), 1 /* TEXT */)
        ]),
        default: _withCtx(() => [
          _createElementVNode("div", { class: "_gaps" }, [
            _createElementVNode("div", null, _toDisplayString(_unref(i18n).ts._initialTutorial._postNote._visibility.description), 1 /* TEXT */),
            _createElementVNode("div", null, [
              _hoisted_2,
              _createTextVNode(),
              _createElementVNode("b", null, _toDisplayString(_unref(i18n).ts._visibility.public), 1 /* TEXT */),
              _createTextVNode(" … " + _toDisplayString(_unref(i18n).ts._initialTutorial._postNote._visibility.public), 1 /* TEXT */)
            ]),
            _createElementVNode("div", null, [
              _hoisted_3,
              _createTextVNode(),
              _createElementVNode("b", null, _toDisplayString(_unref(i18n).ts._visibility.home), 1 /* TEXT */),
              _createTextVNode(" … " + _toDisplayString(_unref(i18n).ts._initialTutorial._postNote._visibility.home), 1 /* TEXT */)
            ]),
            _createElementVNode("div", null, [
              _hoisted_4,
              _createTextVNode(),
              _createElementVNode("b", null, _toDisplayString(_unref(i18n).ts._visibility.followers), 1 /* TEXT */),
              _createTextVNode(" … " + _toDisplayString(_unref(i18n).ts._initialTutorial._postNote._visibility.followers), 1 /* TEXT */)
            ]),
            _createElementVNode("div", { class: "_gaps_s" }, [
              _createElementVNode("div", null, [
                _hoisted_5,
                _createTextVNode(),
                _createElementVNode("b", null, _toDisplayString(_unref(i18n).ts._visibility.specified), 1 /* TEXT */),
                _createTextVNode(" … " + _toDisplayString(_unref(i18n).ts._initialTutorial._postNote._visibility.direct), 1 /* TEXT */)
              ]),
              _createVNode(MkInfo, { warn: true }, {
                default: _withCtx(() => [
                  _createElementVNode("b", null, _toDisplayString(_unref(i18n).ts._initialTutorial._postNote._visibility.doNotSendConfidencialOnDirect1), 1 /* TEXT */),
                  _createTextVNode(" "),
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._initialTutorial._postNote._visibility.doNotSendConfidencialOnDirect2), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["warn"])
            ]),
            _createElementVNode("div", null, [
              _hoisted_6,
              _createTextVNode(),
              _createElementVNode("b", null, _toDisplayString(_unref(i18n).ts._visibility.disableFederation), 1 /* TEXT */),
              _createTextVNode(" … " + _toDisplayString(_unref(i18n).ts._initialTutorial._postNote._visibility.localOnly), 1 /* TEXT */)
            ])
          ])
        ]),
        _: 1 /* STABLE */
      }), _createVNode(MkFormSection, null, {
        label: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts._initialTutorial._postNote._cw.title), 1 /* TEXT */)
        ]),
        default: _withCtx(() => [
          _createElementVNode("div", { class: "_gaps" }, [
            _createElementVNode("div", null, _toDisplayString(_unref(i18n).ts._initialTutorial._postNote._cw.description), 1 /* TEXT */),
            _createVNode(MkNote, {
              class: _normalizeClass(_ctx.$style.exampleRoot),
              note: exampleCWNote,
              mock: true
            }, null, 8 /* PROPS */, ["note", "mock"]),
            _createElementVNode("div", null, _toDisplayString(_unref(i18n).ts._initialTutorial._postNote._cw.useCases), 1 /* TEXT */)
          ])
        ]),
        _: 1 /* STABLE */
      }) ]))
}
}

})
