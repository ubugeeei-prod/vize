import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, withDirectives as _withDirectives, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref, vShow as _vShow, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-pin" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-repeat", style: "margin-right: 4px;" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-rocket-off" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-device-tv" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-back-up" })
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-device-tv" })
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-back-up" })
const _hoisted_8 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-repeat" })
const _hoisted_9 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-ban" })
const _hoisted_10 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-paperclip" })
const _hoisted_11 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-dots" })
import { computed, inject, onMounted, ref, useTemplateRef, provide } from 'vue'
import * as mfm from 'mfm-js'
import * as Misskey from 'misskey-js'
import { isLink } from '@@/js/is-link.js'
import { shouldCollapsed } from '@@/js/collapsed.js'
import { host } from '@@/js/config.js'
import type { Ref } from 'vue'
import type { MenuItem } from '@/types/menu.js'
import type { OpenOnRemoteOptions } from '@/utility/please-login.js'
import type { Keymap } from '@/utility/hotkey.js'
import MkNoteSub from '@/components/MkNoteSub.vue'
import MkNoteHeader from '@/components/MkNoteHeader.vue'
import MkNoteSimple from '@/components/MkNoteSimple.vue'
import MkReactionsViewer from '@/components/MkReactionsViewer.vue'
import MkReactionsViewerDetails from '@/components/MkReactionsViewer.details.vue'
import MkMediaList from '@/components/MkMediaList.vue'
import MkCwButton from '@/components/MkCwButton.vue'
import MkPoll from '@/components/MkPoll.vue'
import MkUsersTooltip from '@/components/MkUsersTooltip.vue'
import MkUrlPreview from '@/components/MkUrlPreview.vue'
import MkInstanceTicker from '@/components/MkInstanceTicker.vue'
import { pleaseLogin } from '@/utility/please-login.js'
import { checkWordMute } from '@/utility/check-word-mute.js'
import { notePage } from '@/filters/note.js'
import { userPage } from '@/filters/user.js'
import number from '@/filters/number.js'
import * as os from '@/os.js'
import * as sound from '@/utility/sound.js'
import { misskeyApi, misskeyApiGet } from '@/utility/misskey-api.js'
import { reactionPicker } from '@/utility/reaction-picker.js'
import { extractUrlFromMfm } from '@/utility/extract-url-from-mfm.js'
import { $i } from '@/i.js'
import { i18n } from '@/i18n.js'
import { getAbuseNoteMenu, getCopyNoteLinkMenu, getNoteClipMenu, getNoteMenu, getRenoteMenu } from '@/utility/get-note-menu.js'
import { noteEvents, useNoteCapture } from '@/composables/use-note-capture.js'
import { deepClone } from '@/utility/clone.js'
import { useTooltip } from '@/composables/use-tooltip.js'
import { claimAchievement } from '@/utility/achievements.js'
import { getNoteSummary } from '@/utility/get-note-summary.js'
import MkRippleEffect from '@/components/MkRippleEffect.vue'
import { showMovedDialog } from '@/utility/show-moved-dialog.js'
import { isEnabledUrlPreview } from '@/utility/url-preview.js'
import { focusPrev, focusNext } from '@/utility/focus.js'
import { getAppearNote } from '@/utility/get-appear-note.js'
import { prefer } from '@/preferences.js'
import { getPluginHandlers } from '@/plugin.js'
import { DI } from '@/di.js'
import { globalEvents } from '@/events.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkNote',
  props: {
    note: { type: null, required: true },
    pinned: { type: Boolean, required: false },
    mock: { type: Boolean, required: false, default: false },
    withHardMute: { type: Boolean, required: false }
  },
  emits: ["reaction", "removeReaction"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
provide(DI.mock, props.mock);
const inTimeline = inject<boolean>('inTimeline', false);
const tl_withSensitive = inject<Ref<boolean>>('tl_withSensitive', ref(true));
const inChannel = inject('inChannel', null);
const currentClip = inject<Ref<Misskey.entities.Clip> | null>('currentClip', null);
let note = deepClone(props.note);
// plugin
const noteViewInterruptors = getPluginHandlers('note_view_interruptor');
const hideByPlugin = ref(false);
if (noteViewInterruptors.length > 0) {
	let result: Misskey.entities.Note | null = deepClone(note);
	for (const interruptor of noteViewInterruptors) {
		try {
			result = interruptor.handler(result!) as Misskey.entities.Note | null;
		} catch (err) {
			console.error(err);
		}
	}
	if (result == null) {
		hideByPlugin.value = true;
	} else {
		note = result as Misskey.entities.Note;
	}
}
const isRenote = Misskey.note.isPureRenote(note);
const appearNote = getAppearNote(note) ?? note;
const { $note: $appearNote, subscribe: subscribeManuallyToNoteCapture } = useNoteCapture({
	note: appearNote,
	parentNote: note,
	mock: props.mock,
});
const rootEl = useTemplateRef('rootEl');
const menuButton = useTemplateRef('menuButton');
const renoteButton = useTemplateRef('renoteButton');
const renoteTime = useTemplateRef('renoteTime');
const reactButton = useTemplateRef('reactButton');
const clipButton = useTemplateRef('clipButton');
const galleryEl = useTemplateRef('galleryEl');
const isMyRenote = $i && ($i.id === note.userId);
const showContent = ref(false);
const parsed = computed(() => appearNote.text ? mfm.parse(appearNote.text) : null);
const urls = computed(() => parsed.value ? extractUrlFromMfm(parsed.value).filter((url) => appearNote.renote?.url !== url && appearNote.renote?.uri !== url) : null);
const isLong = shouldCollapsed(appearNote, urls.value ?? []);
const collapsed = ref(appearNote.cw == null && isLong);
const muted = ref(checkMute(appearNote, $i?.mutedWords));
const hardMuted = ref(props.withHardMute && checkMute(appearNote, $i?.hardMutedWords, true));
const showSoftWordMutedWord = computed(() => prefer.s.showSoftWordMutedWord);
const translation = ref<Misskey.entities.NotesTranslateResponse | null>(null);
const translating = ref(false);
const showTicker = (prefer.s.instanceTicker === 'always') || (prefer.s.instanceTicker === 'remote' && appearNote.user.instance);
const canRenote = computed(() => ['public', 'home'].includes(appearNote.visibility) || (appearNote.visibility === 'followers' && appearNote.userId === $i?.id));
const renoteCollapsed = ref(
	prefer.s.collapseRenotes && isRenote && (
		($i && ($i.id === note.userId || $i.id === appearNote.userId)) || // `||` must be `||`! See https://github.com/misskey-dev/misskey/issues/13131
		($appearNote.myReaction != null)
	),
);
const pleaseLoginContext = computed<OpenOnRemoteOptions>(() => ({
	type: 'lookup',
	url: `https://${host}/notes/${appearNote.id}`,
}));
/* eslint-disable no-redeclare */
/** checkOnlyでは純粋なワードミュート結果をbooleanで返却する */
function checkMute(noteToCheck: Misskey.entities.Note, mutedWords: Array<string | string[]> | undefined | null, checkOnly: true): boolean;
function checkMute(noteToCheck: Misskey.entities.Note, mutedWords: Array<string | string[]> | undefined | null, checkOnly?: false): Array<string | string[]> | false | 'sensitiveMute';
function checkMute(noteToCheck: Misskey.entities.Note, mutedWords: Array<string | string[]> | undefined | null, checkOnly = false): Array<string | string[]> | boolean | 'sensitiveMute' {
	if (mutedWords != null) {
		const result = checkWordMute(noteToCheck, $i, mutedWords);
		if (Array.isArray(result)) {
			return checkOnly ? (result.length > 0) : result;
		}
		const replyResult = noteToCheck.reply && checkWordMute(noteToCheck.reply, $i, mutedWords);
		if (Array.isArray(replyResult)) {
			return checkOnly ? (replyResult.length > 0) : replyResult;
		}
		const renoteResult = noteToCheck.renote && checkWordMute(noteToCheck.renote, $i, mutedWords);
		if (Array.isArray(renoteResult)) {
			return checkOnly ? (renoteResult.length > 0) : renoteResult;
		}
	}
	if (checkOnly) return false;
	if (inTimeline && tl_withSensitive.value === false && noteToCheck.files?.some((v) => v.isSensitive)) {
		return 'sensitiveMute';
	}
	return false;
}
/* eslint-enable no-redeclare */
const keymap = {
	'r': () => {
		if (renoteCollapsed.value) return;
		reply();
	},
	'e|a|plus': () => {
		if (renoteCollapsed.value) return;
		react();
	},
	'q': () => {
		if (renoteCollapsed.value) return;
		renote();
	},
	'm': () => {
		if (renoteCollapsed.value) return;
		showMenu();
	},
	'c': () => {
		if (renoteCollapsed.value) return;
		if (!prefer.s.showClipButtonInNoteFooter) return;
		clip();
	},
	'o': () => {
		if (renoteCollapsed.value) return;
		galleryEl.value?.openGallery();
	},
	'v|enter': () => {
		if (renoteCollapsed.value) {
			renoteCollapsed.value = false;
		} else if (appearNote.cw != null) {
			showContent.value = !showContent.value;
		} else if (isLong) {
			collapsed.value = !collapsed.value;
		}
	},
	'esc': {
		allowRepeat: true,
		callback: () => blur(),
	},
	'up|k|shift+tab': {
		allowRepeat: true,
		callback: () => focusBefore(),
	},
	'down|j|tab': {
		allowRepeat: true,
		callback: () => focusAfter(),
	},
} as const satisfies Keymap;
provide(DI.mfmEmojiReactCallback, (reaction) => {
	sound.playMisskeySfx('reaction');
	misskeyApi('notes/reactions/create', {
		noteId: appearNote.id,
		reaction: reaction,
	}).then(() => {
		noteEvents.emit(`reacted:${appearNote.id}`, {
			userId: $i!.id,
			reaction: reaction,
		});
	});
});
if (!props.mock) {
	useTooltip(renoteButton, async (showing) => {
		const renotes = await misskeyApi('notes/renotes', {
			noteId: appearNote.id,
			limit: 11,
		});
		const users = renotes.map(x => x.user);
		if (users.length < 1 || renoteButton.value == null) return;
		const { dispose } = os.popup(MkUsersTooltip, {
			showing,
			users,
			count: appearNote.renoteCount,
			anchorElement: renoteButton.value,
		}, {
			closed: () => dispose(),
		});
	});
	if (appearNote.reactionAcceptance === 'likeOnly') {
		useTooltip(reactButton, async (showing) => {
			const reactions = await misskeyApiGet('notes/reactions', {
				noteId: appearNote.id,
				limit: 10,
				_cacheKey_: $appearNote.reactionCount,
			});
			const users = reactions.map(x => x.user);
			if (users.length < 1) return;
			const { dispose } = os.popup(MkReactionsViewerDetails, {
				showing,
				reaction: '❤️',
				users,
				count: $appearNote.reactionCount,
				anchorElement: reactButton.value!,
			}, {
				closed: () => dispose(),
			});
		});
	}
}
async function renote() {
	if (props.mock) return;
	const isLoggedIn = await pleaseLogin({ openOnRemote: pleaseLoginContext.value });
	if (!isLoggedIn) return;
	showMovedDialog();
	const { menu } = getRenoteMenu({ note: note, renoteButton, mock: props.mock });
	os.popupMenu(menu, renoteButton.value);
	subscribeManuallyToNoteCapture();
}
async function reply() {
	if (props.mock) return;
	const isLoggedIn = await pleaseLogin({ openOnRemote: pleaseLoginContext.value });
	if (!isLoggedIn) return;
	os.post({
		reply: appearNote,
		channel: appearNote.channel,
	}).then(() => {
		focus();
	});
}
async function react() {
	const isLoggedIn = await pleaseLogin({ openOnRemote: pleaseLoginContext.value });
	if (!isLoggedIn) return;
	showMovedDialog();
	if (appearNote.reactionAcceptance === 'likeOnly') {
		sound.playMisskeySfx('reaction');
		if (props.mock) {
			return;
		}
		misskeyApi('notes/reactions/create', {
			noteId: appearNote.id,
			reaction: '❤️',
		}).then(() => {
			noteEvents.emit(`reacted:${appearNote.id}`, {
				userId: $i!.id,
				reaction: '❤️',
			});
		});
		const el = reactButton.value;
		if (el && prefer.s.animation) {
			const rect = el.getBoundingClientRect();
			const x = rect.left + (el.offsetWidth / 2);
			const y = rect.top + (el.offsetHeight / 2);
			const { dispose } = os.popup(MkRippleEffect, { x, y }, {
				end: () => dispose(),
			});
		}
	} else {
		blur();
		reactionPicker.show(reactButton.value ?? null, note, async (reaction) => {
			if (prefer.s.confirmOnReact) {
				const confirm = await os.confirm({
					type: 'question',
					text: i18n.tsx.reactAreYouSure({ emoji: reaction.replace('@.', '') }),
				});
				if (confirm.canceled) return;
			}
			sound.playMisskeySfx('reaction');
			if (props.mock) {
				emit('reaction', reaction);
				$appearNote.reactions[reaction] = 1;
				$appearNote.reactionCount++;
				$appearNote.myReaction = reaction;
				return;
			}
			misskeyApi('notes/reactions/create', {
				noteId: appearNote.id,
				reaction: reaction,
			}).then(() => {
				noteEvents.emit(`reacted:${appearNote.id}`, {
					userId: $i!.id,
					reaction: reaction,
				});
			});
			if (appearNote.text && appearNote.text.length > 100 && (Date.now() - new Date(appearNote.createdAt).getTime() < 1000 * 3)) {
				claimAchievement('reactWithoutRead');
			}
		}, () => {
			focus();
		});
	}
}
function undoReact(): void {
	const oldReaction = $appearNote.myReaction;
	if (!oldReaction) return;
	if (props.mock) {
		emit('removeReaction', oldReaction);
		return;
	}
	misskeyApi('notes/reactions/delete', {
		noteId: appearNote.id,
	}).then(() => {
		noteEvents.emit(`unreacted:${appearNote.id}`, {
			userId: $i!.id,
			reaction: oldReaction,
		});
	});
}
function toggleReact() {
	if ($appearNote.myReaction == null) {
		react();
	} else {
		undoReact();
	}
}
function onContextmenu(ev: PointerEvent): void {
	if (props.mock) {
		return;
	}
	if (ev.target && isLink(ev.target as HTMLElement)) return;
	if (window.getSelection()?.toString() !== '') return;
	if (prefer.s.useReactionPickerForContextMenu) {
		ev.preventDefault();
		react();
	} else {
		const { menu, cleanup } = getNoteMenu({ note: note, translating, translation, currentClip: currentClip?.value });
		os.contextMenu(menu, ev).then(focus).finally(cleanup);
	}
}
function showMenu(): void {
	if (props.mock) {
		return;
	}
	const { menu, cleanup } = getNoteMenu({ note: note, translating, translation, currentClip: currentClip?.value });
	os.popupMenu(menu, menuButton.value).then(focus).finally(cleanup);
}
async function clip(): Promise<void> {
	if (props.mock) {
		return;
	}
	os.popupMenu(await getNoteClipMenu({ note: note, currentClip: currentClip?.value }), clipButton.value).then(focus);
}
async function showRenoteMenu() {
	if (props.mock) {
		return;
	}
	const isLoggedIn = await pleaseLogin({ openOnRemote: pleaseLoginContext.value });
	if (!isLoggedIn) return;
	function getUnrenote(): MenuItem {
		return {
			text: i18n.ts.unrenote,
			icon: 'ti ti-trash',
			danger: true,
			action: () => {
				misskeyApi('notes/delete', {
					noteId: note.id,
				}).then(() => {
					globalEvents.emit('noteDeleted', note.id);
				});
			},
		};
	}
	const renoteDetailsMenu: MenuItem = {
		type: 'link',
		text: i18n.ts.renoteDetails,
		icon: 'ti ti-info-circle',
		to: notePage(note),
	};
	if (isMyRenote) {
		os.popupMenu([
			renoteDetailsMenu,
			getCopyNoteLinkMenu(note, i18n.ts.copyLinkRenote),
			{ type: 'divider' },
			getUnrenote(),
		], renoteTime.value);
	} else {
		os.popupMenu([
			renoteDetailsMenu,
			getCopyNoteLinkMenu(note, i18n.ts.copyLinkRenote),
			{ type: 'divider' },
			getAbuseNoteMenu(note, i18n.ts.reportAbuseRenote),
			...(($i?.isModerator || $i?.isAdmin) ? [getUnrenote()] : []),
		], renoteTime.value);
	}
}
function focus() {
	rootEl.value?.focus();
}
function blur() {
	rootEl.value?.blur();
}
function focusBefore() {
	focusPrev(rootEl.value);
}
function focusAfter() {
	focusNext(rootEl.value);
}
function readPromo() {
	misskeyApi('promo/read', {
		noteId: appearNote.id,
	});
}
function emitUpdReaction(emoji: string, delta: number) {
	if (delta < 0) {
		emit('removeReaction', emoji);
	} else if (delta > 0) {
		emit('reaction', emoji);
	}
}

return (_ctx: any,_cache: any) => {
  const _component_MkAvatar = _resolveComponent("MkAvatar")
  const _component_MkUserName = _resolveComponent("MkUserName")
  const _component_MkA = _resolveComponent("MkA")
  const _component_I18n = _resolveComponent("I18n")
  const _component_MkTime = _resolveComponent("MkTime")
  const _component_Mfm = _resolveComponent("Mfm")
  const _component_MkLoading = _resolveComponent("MkLoading")
  const _directive_hotkey = _resolveDirective("hotkey")
  const _directive_user_preview = _resolveDirective("user-preview")

  return (!hardMuted.value && !hideByPlugin.value && muted.value === false)
      ? (_openBlock(), _createElementBlock("div", {
        key: 0,
        ref: "rootEl",
        class: _normalizeClass([_ctx.$style.root, { [_ctx.$style.showActionsOnlyHover]: _unref(prefer).s.showNoteActionsOnlyHover, [_ctx.$style.skipRender]: _unref(prefer).s.skipNoteRender }]),
        tabindex: "0"
      }, [ (_unref(appearNote).replyId && !renoteCollapsed.value) ? (_openBlock(), _createBlock(MkNoteSub, {
            key: 0,
            note: _unref(appearNote)?.reply ?? null,
            class: _normalizeClass(_ctx.$style.replyTo)
          }, null, 8 /* PROPS */, ["note"])) : _createCommentVNode("v-if", true), (__props.pinned) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: _normalizeClass(_ctx.$style.tip)
          }, [ _hoisted_1, _createTextVNode(), _toDisplayString(_unref(i18n).ts.pinnedNote) ])) : _createCommentVNode("v-if", true), (_unref(isRenote)) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: _normalizeClass(_ctx.$style.renote)
          }, [ (_unref(note).channel) ? (_openBlock(), _createElementBlock("div", {
                key: 0,
                class: _normalizeClass(_ctx.$style.colorBar),
                style: _normalizeStyle({ background: _unref(note).channel.color })
              })) : _createCommentVNode("v-if", true), _createVNode(_component_MkAvatar, {
              class: _normalizeClass(_ctx.$style.renoteAvatar),
              user: _unref(note).user,
              link: "",
              preview: ""
            }, null, 8 /* PROPS */, ["user"]), _hoisted_2, _createVNode(_component_I18n, {
              src: _unref(i18n).ts.renotedBy,
              tag: "span",
              class: _normalizeClass(_ctx.$style.renoteText)
            }, {
              user: _withCtx(() => [
                _createVNode(_component_MkA, {
                  class: _normalizeClass(_ctx.$style.renoteUserName),
                  to: _unref(userPage)(_unref(note).user)
                }, {
                  default: _withCtx(() => [
                    _createVNode(_component_MkUserName, { user: _unref(note).user }, null, 8 /* PROPS */, ["user"])
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["to"])
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["src"]), _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.renoteInfo)
            }, [ _createElementVNode("button", {
                ref_key: "renoteTime", ref: renoteTime,
                class: _normalizeClass(["_button", _ctx.$style.renoteTime]),
                onMousedown: _cache[0] || (_cache[0] = _withModifiers(($event: any) => (showRenoteMenu()), ["prevent"]))
              }, [ _createElementVNode("i", {
                  class: _normalizeClass(["ti ti-dots", _ctx.$style.renoteMenu])
                }), _createVNode(_component_MkTime, { time: _unref(note).createdAt }, null, 8 /* PROPS */, ["time"]) ], 32 /* NEED_HYDRATION */), (_unref(note).visibility !== 'public') ? (_openBlock(), _createElementBlock("span", {
                  key: 0,
                  style: "margin-left: 0.5em;",
                  title: _unref(i18n).ts._visibility[_unref(note).visibility]
                }, [ (_unref(note).visibility === 'home') ? (_openBlock(), _createElementBlock("i", {
                      key: 0,
                      class: "ti ti-home"
                    })) : (_unref(note).visibility === 'followers') ? (_openBlock(), _createElementBlock("i", {
                        key: 1,
                        class: "ti ti-lock"
                      })) : (_unref(note).visibility === 'specified') ? (_openBlock(), _createElementBlock("i", {
                        key: 2,
                        ref: "specified",
                        class: "ti ti-mail"
                      })) : _createCommentVNode("v-if", true) ])) : _createCommentVNode("v-if", true), (_unref(note).localOnly) ? (_openBlock(), _createElementBlock("span", {
                  key: 0,
                  style: "margin-left: 0.5em;",
                  title: _unref(i18n).ts._visibility['disableFederation']
                }, [ _hoisted_3 ])) : _createCommentVNode("v-if", true), (_unref(note).channel) ? (_openBlock(), _createElementBlock("span", {
                  key: 0,
                  style: "margin-left: 0.5em;",
                  title: _unref(note).channel.name
                }, [ _hoisted_4 ])) : _createCommentVNode("v-if", true) ]) ])) : _createCommentVNode("v-if", true), (_unref(isRenote) && _unref(note).renote == null) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: _normalizeClass(_ctx.$style.deleted)
          }, _toDisplayString(_unref(i18n).ts.deletedNote), 1 /* TEXT */)) : (renoteCollapsed.value) ? (_openBlock(), _createElementBlock("div", {
              key: 1,
              class: _normalizeClass(_ctx.$style.collapsedRenoteTarget)
            }, [ _createVNode(_component_MkAvatar, {
                class: _normalizeClass(_ctx.$style.collapsedRenoteTargetAvatar),
                user: _unref(appearNote).user,
                link: "",
                preview: ""
              }, null, 8 /* PROPS */, ["user"]), _createVNode(_component_Mfm, {
                text: _unref(getNoteSummary)(_unref(appearNote)),
                plain: true,
                nowrap: true,
                author: _unref(appearNote).user,
                nyaize: 'respect',
                class: _normalizeClass(_ctx.$style.collapsedRenoteTargetText),
                onClick: _cache[1] || (_cache[1] = ($event: any) => (renoteCollapsed.value = false))
              }, null, 8 /* PROPS */, ["text", "plain", "nowrap", "author", "nyaize"]) ])) : (_openBlock(), _createElementBlock("article", {
            key: 2,
            class: _normalizeClass(_ctx.$style.article),
            onContextmenu: _withModifiers(onContextmenu, ["stop"])
          }, [ (_unref(appearNote).channel) ? (_openBlock(), _createElementBlock("div", {
                key: 0,
                class: _normalizeClass(_ctx.$style.colorBar),
                style: _normalizeStyle({ background: _unref(appearNote).channel.color })
              })) : _createCommentVNode("v-if", true), _createVNode(_component_MkAvatar, {
              class: _normalizeClass([_ctx.$style.avatar, _unref(prefer).s.useStickyIcons ? _ctx.$style.useSticky : null]),
              user: _unref(appearNote).user,
              link: !__props.mock,
              preview: !__props.mock
            }, null, 10 /* CLASS, PROPS */, ["user", "link", "preview"]), _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.main)
            }, [ _createVNode(MkNoteHeader, {
                note: _unref(appearNote),
                mini: true
              }, null, 8 /* PROPS */, ["note", "mini"]), (_unref(showTicker)) ? (_openBlock(), _createBlock(MkInstanceTicker, {
                  key: 0,
                  host: _unref(appearNote).user.host,
                  instance: _unref(appearNote).user.instance
                }, null, 8 /* PROPS */, ["host", "instance"])) : _createCommentVNode("v-if", true), _createElementVNode("div", { style: "container-type: inline-size;" }, [ (_unref(appearNote).cw != null) ? (_openBlock(), _createElementBlock("p", {
                    key: 0,
                    class: _normalizeClass(_ctx.$style.cw)
                  }, [ (_unref(appearNote).cw != '') ? (_openBlock(), _createBlock(_component_Mfm, {
                        key: 0,
                        text: _unref(appearNote).cw,
                        author: _unref(appearNote).user,
                        nyaize: 'respect',
                        enableEmojiMenu: true,
                        enableEmojiMenuReaction: true
                      }, null, 8 /* PROPS */, ["text", "author", "nyaize", "enableEmojiMenu", "enableEmojiMenuReaction"])) : _createCommentVNode("v-if", true), _createVNode(MkCwButton, {
                      text: _unref(appearNote).text,
                      renote: _unref(appearNote).renote,
                      files: _unref(appearNote).files,
                      poll: _unref(appearNote).poll,
                      style: "margin: 4px 0;",
                      modelValue: showContent.value,
                      "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((showContent).value = $event))
                    }, null, 8 /* PROPS */, ["text", "renote", "files", "poll", "modelValue"]) ])) : _createCommentVNode("v-if", true), _withDirectives(_createElementVNode("div", {
                  class: _normalizeClass([{ [_ctx.$style.contentCollapsed]: collapsed.value }])
                }, [ _createElementVNode("div", {
                    class: _normalizeClass(_ctx.$style.text)
                  }, [ (_unref(appearNote).isHidden) ? (_openBlock(), _createElementBlock("span", {
                        key: 0,
                        style: "opacity: 0.5"
                      }, "(" + _toDisplayString(_unref(i18n).ts.private) + ")", 1 /* TEXT */)) : _createCommentVNode("v-if", true), (_unref(appearNote).replyId) ? (_openBlock(), _createBlock(_component_MkA, {
                        key: 0,
                        class: _normalizeClass(_ctx.$style.replyIcon),
                        to: `/notes/${_unref(appearNote).replyId}`
                      }, {
                        default: _withCtx(() => [
                          _hoisted_5
                        ]),
                        _: 1 /* STABLE */
                      }, 8 /* PROPS */, ["to"])) : _createCommentVNode("v-if", true), (_unref(appearNote).text) ? (_openBlock(), _createBlock(_component_Mfm, {
                        key: 0,
                        parsedNodes: parsed.value,
                        text: _unref(appearNote).text,
                        author: _unref(appearNote).user,
                        nyaize: 'respect',
                        emojiUrls: _unref(appearNote).emojis,
                        enableEmojiMenu: true,
                        enableEmojiMenuReaction: true,
                        class: "_selectable"
                      }, null, 8 /* PROPS */, ["parsedNodes", "text", "author", "nyaize", "emojiUrls", "enableEmojiMenu", "enableEmojiMenuReaction"])) : _createCommentVNode("v-if", true), (translating.value || translation.value) ? (_openBlock(), _createElementBlock("div", {
                        key: 0,
                        class: _normalizeClass(_ctx.$style.translation)
                      }, [ (translating.value) ? (_openBlock(), _createBlock(_component_MkLoading, {
                            key: 0,
                            mini: ""
                          })) : (translation.value) ? (_openBlock(), _createElementBlock("div", { key: 1 }, [ _createElementVNode("b", null, _toDisplayString(_unref(i18n).tsx.translatedFrom({ x: translation.value.sourceLang })) + ": ", 1 /* TEXT */), _createVNode(_component_Mfm, {
                                text: translation.value.text,
                                author: _unref(appearNote).user,
                                nyaize: 'respect',
                                emojiUrls: _unref(appearNote).emojis,
                                class: "_selectable"
                              }, null, 8 /* PROPS */, ["text", "author", "nyaize", "emojiUrls"]) ])) : _createCommentVNode("v-if", true) ])) : _createCommentVNode("v-if", true) ]), (_unref(appearNote).files && _unref(appearNote).files.length > 0) ? (_openBlock(), _createElementBlock("div", {
                      key: 0,
                      style: "margin-top: 8px;"
                    }, [ _createVNode(MkMediaList, {
                        ref_key: "galleryEl", ref: galleryEl,
                        mediaList: _unref(appearNote).files
                      }, null, 8 /* PROPS */, ["mediaList"]) ])) : _createCommentVNode("v-if", true), (_unref(appearNote).poll) ? (_openBlock(), _createBlock(MkPoll, {
                      key: 0,
                      noteId: _unref(appearNote).id,
                      multiple: _unref(appearNote).poll.multiple,
                      expiresAt: _unref(appearNote).poll.expiresAt,
                      choices: _unref($appearNote).pollChoices,
                      author: _unref(appearNote).user,
                      emojiUrls: _unref(appearNote).emojis,
                      class: _normalizeClass(_ctx.$style.poll)
                    }, null, 8 /* PROPS */, ["noteId", "multiple", "expiresAt", "choices", "author", "emojiUrls"])) : _createCommentVNode("v-if", true), (_unref(isEnabledUrlPreview)) ? (_openBlock(), _createElementBlock("div", { key: 0 }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(urls.value, (url) => {
                        return (_openBlock(), _createBlock(MkUrlPreview, {
                          key: url,
                          url: url,
                          compact: true,
                          detail: false,
                          class: _normalizeClass(_ctx.$style.urlPreview)
                        }, null, 8 /* PROPS */, ["url", "compact", "detail"]))
                      }), 128 /* KEYED_FRAGMENT */)) ])) : _createCommentVNode("v-if", true), (_unref(appearNote).renoteId) ? (_openBlock(), _createElementBlock("div", {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.quote)
                    }, [ _createVNode(MkNoteSimple, {
                        note: _unref(appearNote)?.renote ?? null,
                        class: _normalizeClass(_ctx.$style.quoteNote)
                      }, null, 8 /* PROPS */, ["note"]) ])) : _createCommentVNode("v-if", true), (_unref(isLong) && collapsed.value) ? (_openBlock(), _createElementBlock("button", {
                      key: 0,
                      class: _normalizeClass(["_button", _ctx.$style.collapsed]),
                      onClick: _cache[3] || (_cache[3] = ($event: any) => (collapsed.value = false))
                    }, [ _createElementVNode("span", {
                        class: _normalizeClass(_ctx.$style.collapsedLabel)
                      }, _toDisplayString(_unref(i18n).ts.showMore), 1 /* TEXT */) ])) : (_unref(isLong) && !collapsed.value) ? (_openBlock(), _createElementBlock("button", {
                        key: 1,
                        class: _normalizeClass(["_button", _ctx.$style.showLess]),
                        onClick: _cache[4] || (_cache[4] = ($event: any) => (collapsed.value = true))
                      }, [ _createElementVNode("span", {
                          class: _normalizeClass(_ctx.$style.showLessLabel)
                        }, _toDisplayString(_unref(i18n).ts.showLess), 1 /* TEXT */) ])) : _createCommentVNode("v-if", true) ], 2 /* CLASS */), [ [_vShow, _unref(appearNote).cw == null || showContent.value] ]), (_unref(appearNote).channel && !inChannel) ? (_openBlock(), _createBlock(_component_MkA, {
                    key: 0,
                    class: _normalizeClass(_ctx.$style.channel),
                    to: `/channels/${_unref(appearNote).channel.id}`
                  }, {
                    default: _withCtx(() => [
                      _hoisted_6,
                      _createTextVNode(" "),
                      _createTextVNode(_toDisplayString(_unref(appearNote).channel.name), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["to"])) : _createCommentVNode("v-if", true) ]), (_unref(appearNote).reactionAcceptance !== 'likeOnly') ? (_openBlock(), _createBlock(MkReactionsViewer, {
                  key: 0,
                  style: "margin-top: 6px;",
                  reactions: _unref($appearNote).reactions,
                  reactionEmojis: _unref($appearNote).reactionEmojis,
                  myReaction: _unref($appearNote).myReaction,
                  noteId: _unref(appearNote).id,
                  maxNumber: 16,
                  onMockUpdateMyReaction: emitUpdReaction
                }, {
                  more: _withCtx(() => [
                    _createVNode(_component_MkA, {
                      to: `/notes/${_unref(appearNote).id}/reactions`,
                      class: _normalizeClass([_ctx.$style.reactionOmitted])
                    }, {
                      default: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.more), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["to"])
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["reactions", "reactionEmojis", "myReaction", "noteId", "maxNumber"])) : _createCommentVNode("v-if", true), _createElementVNode("footer", {
                class: _normalizeClass(_ctx.$style.footer)
              }, [ _createElementVNode("button", {
                  class: _normalizeClass(["_button", _ctx.$style.footerButton]),
                  onClick: _cache[5] || (_cache[5] = ($event: any) => (reply()))
                }, [ _hoisted_7, (_unref(appearNote).repliesCount > 0) ? (_openBlock(), _createElementBlock("p", {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.footerButtonCount)
                    }, _toDisplayString(number(_unref(appearNote).repliesCount)), 1 /* TEXT */)) : _createCommentVNode("v-if", true) ]), (canRenote.value) ? (_openBlock(), _createElementBlock("button", {
                    key: 0,
                    ref: "renoteButton",
                    class: _normalizeClass(["_button", _ctx.$style.footerButton]),
                    onMousedown: _cache[6] || (_cache[6] = _withModifiers(($event: any) => (renote()), ["prevent"]))
                  }, [ _hoisted_8, (_unref(appearNote).renoteCount > 0) ? (_openBlock(), _createElementBlock("p", {
                        key: 0,
                        class: _normalizeClass(_ctx.$style.footerButtonCount)
                      }, _toDisplayString(number(_unref(appearNote).renoteCount)), 1 /* TEXT */)) : _createCommentVNode("v-if", true) ])) : (_openBlock(), _createElementBlock("button", {
                    key: 1,
                    class: _normalizeClass(["_button", _ctx.$style.footerButton]),
                    disabled: ""
                  }, [ _hoisted_9 ])), _createElementVNode("button", {
                  ref_key: "reactButton", ref: reactButton,
                  class: _normalizeClass(["_button", _ctx.$style.footerButton]),
                  onClick: _cache[7] || (_cache[7] = ($event: any) => (toggleReact()))
                }, [ (_unref(appearNote).reactionAcceptance === 'likeOnly' && _unref($appearNote).myReaction != null) ? (_openBlock(), _createElementBlock("i", {
                      key: 0,
                      class: "ti ti-heart-filled",
                      style: "color: var(--MI_THEME-love);"
                    })) : (_unref($appearNote).myReaction != null) ? (_openBlock(), _createElementBlock("i", {
                        key: 1,
                        class: "ti ti-minus",
                        style: "color: var(--MI_THEME-accent);"
                      })) : (_unref(appearNote).reactionAcceptance === 'likeOnly') ? (_openBlock(), _createElementBlock("i", {
                        key: 2,
                        class: "ti ti-heart"
                      })) : (_openBlock(), _createElementBlock("i", {
                      key: 3,
                      class: "ti ti-plus"
                    })), ((_unref(appearNote).reactionAcceptance === 'likeOnly' || _unref(prefer).s.showReactionsCount) && _unref($appearNote).reactionCount > 0) ? (_openBlock(), _createElementBlock("p", {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.footerButtonCount)
                    }, _toDisplayString(number(_unref($appearNote).reactionCount)), 1 /* TEXT */)) : _createCommentVNode("v-if", true) ], 512 /* NEED_PATCH */), (_unref(prefer).s.showClipButtonInNoteFooter) ? (_openBlock(), _createElementBlock("button", {
                    key: 0,
                    ref: "clipButton",
                    class: _normalizeClass(["_button", _ctx.$style.footerButton]),
                    onMousedown: _cache[8] || (_cache[8] = _withModifiers(($event: any) => (clip()), ["prevent"]))
                  }, [ _hoisted_10 ])) : _createCommentVNode("v-if", true), _createElementVNode("button", {
                  ref_key: "menuButton", ref: menuButton,
                  class: _normalizeClass(["_button", _ctx.$style.footerButton]),
                  onMousedown: _cache[9] || (_cache[9] = _withModifiers(($event: any) => (showMenu()), ["prevent"]))
                }, [ _hoisted_11 ], 32 /* NEED_HYDRATION */) ]) ]) ])) ]))
      : (!hardMuted.value && !hideByPlugin.value)
        ? (_openBlock(), _createElementBlock("div", {
          key: 1,
          class: _normalizeClass(_ctx.$style.muted),
          onClick: _cache[10] || (_cache[10] = ($event: any) => (muted.value = false))
        }, [ (muted.value === 'sensitiveMute') ? (_openBlock(), _createBlock(_component_I18n, {
              key: 0,
              src: _unref(i18n).ts.userSaysSomethingSensitive,
              tag: "small"
            }, {
              name: _withCtx(() => [
                _createVNode(_component_MkA, { to: _unref(userPage)(_unref(appearNote).user) }, {
                  default: _withCtx(() => [
                    _createVNode(_component_MkUserName, { user: _unref(appearNote).user }, null, 8 /* PROPS */, ["user"])
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["to"])
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["src"])) : (showSoftWordMutedWord.value !== true) ? (_openBlock(), _createBlock(_component_I18n, {
                key: 1,
                src: _unref(i18n).ts.userSaysSomething,
                tag: "small"
              }, {
                name: _withCtx(() => [
                  _createVNode(_component_MkA, { to: _unref(userPage)(_unref(appearNote).user) }, {
                    default: _withCtx(() => [
                      _createVNode(_component_MkUserName, { user: _unref(appearNote).user }, null, 8 /* PROPS */, ["user"])
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["to"])
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["src"])) : (_openBlock(), _createBlock(_component_I18n, {
              key: 2,
              src: _unref(i18n).ts.userSaysSomethingAbout,
              tag: "small"
            }, {
              name: _withCtx(() => [
                _createVNode(_component_MkA, { to: _unref(userPage)(_unref(appearNote).user) }, {
                  default: _withCtx(() => [
                    _createVNode(_component_MkUserName, { user: _unref(appearNote).user }, null, 8 /* PROPS */, ["user"])
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["to"])
              ]),
              word: _withCtx(() => [
                _createTextVNode(_toDisplayString(Array.isArray(muted.value) ? muted.value.map(words => Array.isArray(words) ? words.join() : words).slice(0, 3).join(' ') : muted.value), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["src"])) ]))
      : (_openBlock(), _createElementBlock("div", { key: 2 }))
}
}

})
