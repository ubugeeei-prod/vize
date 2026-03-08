import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, unref as _unref } from "vue"


const _hoisted_1 = { style: "text-align: center; padding: 0 16px;" }
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-back-up" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-repeat" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-plus" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-dots" })
const _hoisted_6 = { style: "text-align: center; padding: 0 16px;" }
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-check" })
const _hoisted_8 = /*#__PURE__*/ _createElementVNode("br")
import * as Misskey from 'misskey-js'
import { ref, reactive } from 'vue'
import { i18n } from '@/i18n.js'
import { globalEvents } from '@/events.js'
import { $i } from '@/i.js'
import MkNote from '@/components/MkNote.vue'
import { genId } from '@/utility/id.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkTutorialDialog.Note',
  props: {
    phase: { type: String, required: true }
  },
  emits: ["reacted"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const exampleNote = reactive<Misskey.entities.Note>({
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
	text: 'just setting up my msky',
	cw: null,
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
const onceReacted = ref<boolean>(false);
function addReaction(emoji: string) {
	onceReacted.value = true;
	emit('reacted');
	doNotification(emoji);
}
function doNotification(emoji: string): void {
	if (!$i || !emoji) return;
	const notification: Misskey.entities.Notification = {
		id: genId(),
		createdAt: new Date().toUTCString(),
		type: 'reaction',
		reaction: emoji,
		user: $i,
		userId: $i.id,
		note: exampleNote,
	};
	globalEvents.emit('clientNotification', notification);
}
function removeReaction(emoji: string) {
	delete exampleNote.reactions[emoji];
	exampleNote.myReaction = undefined;
}

return (_ctx: any,_cache: any) => {
  return (__props.phase === 'aboutNote')
      ? (_openBlock(), _createElementBlock("div", {
        key: 0,
        class: "_gaps"
      }, [ _createElementVNode("div", _hoisted_1, _toDisplayString(_unref(i18n).ts._initialTutorial._note.description), 1 /* TEXT */), _createVNode(MkNote, {
          class: _normalizeClass(_ctx.$style.exampleNoteRoot),
          style: "pointer-events: none;",
          note: exampleNote,
          mock: true
        }, null, 8 /* PROPS */, ["note", "mock"]), _createElementVNode("div", { class: "_gaps_s" }, [ _createElementVNode("div", null, [ _hoisted_2, _createTextVNode(), _createElementVNode("b", null, _toDisplayString(_unref(i18n).ts.reply), 1 /* TEXT */), _createTextVNode(" … " + _toDisplayString(_unref(i18n).ts._initialTutorial._note.reply), 1 /* TEXT */) ]), _createElementVNode("div", null, [ _hoisted_3, _createTextVNode(), _createElementVNode("b", null, _toDisplayString(_unref(i18n).ts.renote), 1 /* TEXT */), _createTextVNode(" … " + _toDisplayString(_unref(i18n).ts._initialTutorial._note.renote), 1 /* TEXT */) ]), _createElementVNode("div", null, [ _hoisted_4, _createTextVNode(), _createElementVNode("b", null, _toDisplayString(_unref(i18n).ts.reaction), 1 /* TEXT */), _createTextVNode(" … " + _toDisplayString(_unref(i18n).ts._initialTutorial._note.reaction), 1 /* TEXT */) ]), _createElementVNode("div", null, [ _hoisted_5, _createTextVNode(), _createElementVNode("b", null, _toDisplayString(_unref(i18n).ts.menu), 1 /* TEXT */), _createTextVNode(" … " + _toDisplayString(_unref(i18n).ts._initialTutorial._note.menu), 1 /* TEXT */) ]) ]) ]))
      : (__props.phase === 'howToReact')
        ? (_openBlock(), _createElementBlock("div", {
          key: 1,
          class: "_gaps"
        }, [ _createElementVNode("div", _hoisted_6, _toDisplayString(_unref(i18n).ts._initialTutorial._reaction.description), 1 /* TEXT */), _createElementVNode("div", null, _toDisplayString(_unref(i18n).ts._initialTutorial._reaction.letsTryReacting), 1 /* TEXT */), _createVNode(MkNote, {
            class: _normalizeClass(_ctx.$style.exampleNoteRoot),
            note: exampleNote,
            mock: true,
            onReaction: addReaction,
            onRemoveReaction: removeReaction
          }, null, 8 /* PROPS */, ["note", "mock"]), (onceReacted.value) ? (_openBlock(), _createElementBlock("div", { key: 0 }, [ _createElementVNode("b", { style: "color: var(--MI_THEME-accent);" }, [ _hoisted_7, _createTextVNode(" " + _toDisplayString(_unref(i18n).ts._initialTutorial.wellDone), 1 /* TEXT */) ]), _createTextVNode(), _toDisplayString(_unref(i18n).ts._initialTutorial._reaction.reactNotification), _hoisted_8, _toDisplayString(_unref(i18n).ts._initialTutorial._reaction.reactDone) ])) : _createCommentVNode("v-if", true) ]))
      : _createCommentVNode("v-if", true)
}
}

})
