import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, withDirectives as _withDirectives, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, vShow as _vShow, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-repeat", style: "margin-right: 4px;" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-rocket-off" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-rocket-off" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-back-up" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-device-tv" })
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-back-up" })
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-repeat" })
const _hoisted_8 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-ban" })
const _hoisted_9 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-paperclip" })
const _hoisted_10 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-dots" })
const _hoisted_11 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-back-up" })
const _hoisted_12 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-repeat" })
const _hoisted_13 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-icons" })
const _hoisted_14 = { style: "margin-left: 4px;" }
import { computed, inject, markRaw, provide, ref, useTemplateRef } from 'vue'
import * as mfm from 'mfm-js'
import * as Misskey from 'misskey-js'
import { isLink } from '@@/js/is-link.js'
import { host } from '@@/js/config.js'
import type { OpenOnRemoteOptions } from '@/utility/please-login.js'
import type { Keymap } from '@/utility/hotkey.js'
import MkNoteSub from '@/components/MkNoteSub.vue'
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
import { userPage } from '@/filters/user.js'
import { notePage } from '@/filters/note.js'
import number from '@/filters/number.js'
import * as os from '@/os.js'
import { misskeyApi, misskeyApiGet } from '@/utility/misskey-api.js'
import * as sound from '@/utility/sound.js'
import { reactionPicker } from '@/utility/reaction-picker.js'
import { extractUrlFromMfm } from '@/utility/extract-url-from-mfm.js'
import { $i } from '@/i.js'
import { i18n } from '@/i18n.js'
import { getNoteClipMenu, getNoteMenu, getRenoteMenu } from '@/utility/get-note-menu.js'
import { noteEvents, useNoteCapture } from '@/composables/use-note-capture.js'
import { deepClone } from '@/utility/clone.js'
import { useTooltip } from '@/composables/use-tooltip.js'
import { claimAchievement } from '@/utility/achievements.js'
import MkRippleEffect from '@/components/MkRippleEffect.vue'
import { showMovedDialog } from '@/utility/show-moved-dialog.js'
import MkUserCardMini from '@/components/MkUserCardMini.vue'
import MkPagination from '@/components/MkPagination.vue'
import MkReactionIcon from '@/components/MkReactionIcon.vue'
import MkButton from '@/components/MkButton.vue'
import { isEnabledUrlPreview } from '@/utility/url-preview.js'
import { getAppearNote } from '@/utility/get-appear-note.js'
import { prefer } from '@/preferences.js'
import { getPluginHandlers } from '@/plugin.js'
import { DI } from '@/di.js'
import { globalEvents, useGlobalEvent } from '@/events.js'
import { Paginator } from '@/utility/paginator.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkNoteDetailed',
  props: {
    note: { type: null, required: true },
    initialTab: { type: String, required: false, default: 'replies' }
  },
  setup(__props: any) {

const props = __props
const inChannel = inject('inChannel', null);
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
const isDeleted = ref(false);
const muted = ref($i ? checkWordMute(appearNote, $i, $i.mutedWords) : false);
const translation = ref<Misskey.entities.NotesTranslateResponse | null>(null);
const translating = ref(false);
const parsed = appearNote.text ? mfm.parse(appearNote.text) : null;
const urls = parsed ? extractUrlFromMfm(parsed).filter((url) => appearNote.renote?.url !== url && appearNote.renote?.uri !== url) : null;
const showTicker = (prefer.s.instanceTicker === 'always') || (prefer.s.instanceTicker === 'remote' && appearNote.user.instance);
const conversation = ref<Misskey.entities.Note[]>([]);
const replies = ref<Misskey.entities.Note[]>([]);
const canRenote = computed(() => ['public', 'home'].includes(appearNote.visibility) || appearNote.userId === $i?.id);
useGlobalEvent('noteDeleted', (noteId) => {
	if (noteId === note.id || noteId === appearNote.id) {
		isDeleted.value = true;
	}
});
const pleaseLoginContext = computed<OpenOnRemoteOptions>(() => ({
	type: 'lookup',
	url: `https://${host}/notes/${appearNote.id}`,
}));
const keymap = {
	'r': () => reply(),
	'e|a|plus': () => react(),
	'q': () => renote(),
	'm': () => showMenu(),
	'c': () => {
		if (!prefer.s.showClipButtonInNoteFooter) return;
		clip();
	},
	'o': () => {
		galleryEl.value?.openGallery();
	},
	'v|enter': () => {
		if (appearNote.cw != null) {
			showContent.value = !showContent.value;
		}
	},
	'esc': {
		allowRepeat: true,
		callback: () => blur(),
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
const tab = ref(props.initialTab);
const reactionTabType = ref<string | null>(null);
const renotesPaginator = markRaw(new Paginator('notes/renotes', {
	limit: 10,
	params: {
		noteId: appearNote.id,
	},
}));
const reactionsPaginator = markRaw(new Paginator('notes/reactions', {
	limit: 10,
	computedParams: computed(() => ({
		noteId: appearNote.id,
		type: reactionTabType.value,
	})),
}));
useTooltip(renoteButton, async (showing) => {
	const anchorElement = renoteButton.value;
	if (anchorElement == null) return;
	const renotes = await misskeyApi('notes/renotes', {
		noteId: appearNote.id,
		limit: 11,
	});
	const users = renotes.map(x => x.user);
	if (users.length < 1) return;
	const { dispose } = os.popup(MkUsersTooltip, {
		showing,
		users,
		count: appearNote.renoteCount,
		anchorElement: anchorElement,
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
async function renote() {
	const isLoggedIn = await pleaseLogin({ openOnRemote: pleaseLoginContext.value });
	if (!isLoggedIn) return;
	showMovedDialog();
	const { menu } = getRenoteMenu({ note: note, renoteButton });
	os.popupMenu(menu, renoteButton.value);
	// リノート後は反応が来る可能性があるので手動で購読する
	subscribeManuallyToNoteCapture();
}
async function reply() {
	const isLoggedIn = await pleaseLogin({ openOnRemote: pleaseLoginContext.value });
	if (!isLoggedIn) return;
	showMovedDialog();
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
function undoReact(targetNote: Misskey.entities.Note): void {
	const oldReaction = targetNote.myReaction;
	if (!oldReaction) return;
	misskeyApi('notes/reactions/delete', {
		noteId: targetNote.id,
	}).then(() => {
		noteEvents.emit(`unreacted:${appearNote.id}`, {
			userId: $i!.id,
			reaction: oldReaction,
		});
	});
}
function toggleReact() {
	if (appearNote.myReaction == null) {
		react();
	} else {
		undoReact(appearNote);
	}
}
function onContextmenu(ev: PointerEvent): void {
	if (ev.target && isLink(ev.target as HTMLElement)) return;
	if (window.getSelection()?.toString() !== '') return;
	if (prefer.s.useReactionPickerForContextMenu) {
		ev.preventDefault();
		react();
	} else {
		const { menu, cleanup } = getNoteMenu({ note: note, translating, translation });
		os.contextMenu(menu, ev).then(focus).finally(cleanup);
	}
}
function showMenu(): void {
	const { menu, cleanup } = getNoteMenu({ note: note, translating, translation });
	os.popupMenu(menu, menuButton.value).then(focus).finally(cleanup);
}
async function clip(): Promise<void> {
	os.popupMenu(await getNoteClipMenu({ note: note }), clipButton.value).then(focus);
}
async function showRenoteMenu() {
	if (!isMyRenote) return;
	const isLoggedIn = await pleaseLogin({ openOnRemote: pleaseLoginContext.value });
	if (!isLoggedIn) return;
	os.popupMenu([{
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
	}], renoteTime.value);
}
function focus() {
	rootEl.value?.focus();
}
function blur() {
	rootEl.value?.blur();
}
const repliesLoaded = ref(false);
function loadReplies() {
	repliesLoaded.value = true;
	misskeyApi('notes/children', {
		noteId: appearNote.id,
		limit: 30,
	}).then(res => {
		replies.value = res;
	});
}
const conversationLoaded = ref(false);
function loadConversation() {
	conversationLoaded.value = true;
	if (appearNote.replyId == null) return;
	misskeyApi('notes/conversation', {
		noteId: appearNote.replyId,
	}).then(res => {
		conversation.value = res.reverse();
	});
}

return (_ctx: any,_cache: any) => {
  const _component_MkAvatar = _resolveComponent("MkAvatar")
  const _component_MkUserName = _resolveComponent("MkUserName")
  const _component_MkA = _resolveComponent("MkA")
  const _component_I18n = _resolveComponent("I18n")
  const _component_MkTime = _resolveComponent("MkTime")
  const _component_MkAcct = _resolveComponent("MkAcct")
  const _component_Mfm = _resolveComponent("Mfm")
  const _component_MkLoading = _resolveComponent("MkLoading")
  const _directive_hotkey = _resolveDirective("hotkey")
  const _directive_user_preview = _resolveDirective("user-preview")
  const _directive_tooltip = _resolveDirective("tooltip")

  return (!muted.value && !hideByPlugin.value && !isDeleted.value)
      ? (_openBlock(), _createElementBlock("div", {
        key: 0,
        ref: "rootEl",
        class: _normalizeClass(_ctx.$style.root),
        tabindex: "0"
      }, [ (_unref(appearNote).reply && _unref(appearNote).reply.replyId) ? (_openBlock(), _createElementBlock("div", { key: 0 }, [ (!conversationLoaded.value) ? (_openBlock(), _createElementBlock("div", {
                key: 0,
                style: "padding: 16px"
              }, [ _createVNode(MkButton, {
                  style: "margin: 0 auto;",
                  primary: "",
                  rounded: "",
                  onClick: loadConversation
                }, {
                  default: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.loadConversation), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }) ])) : _createCommentVNode("v-if", true), (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(conversation.value, (note) => {
              return (_openBlock(), _createBlock(MkNoteSub, {
                key: _unref(note).id,
                class: _normalizeClass(_ctx.$style.replyToMore),
                note: _unref(note)
              }, null, 8 /* PROPS */, ["note"]))
            }), 128 /* KEYED_FRAGMENT */)) ])) : _createCommentVNode("v-if", true), (_unref(appearNote).replyId) ? (_openBlock(), _createBlock(MkNoteSub, {
            key: 0,
            note: _unref(appearNote)?.reply ?? null,
            class: _normalizeClass(_ctx.$style.replyTo)
          }, null, 8 /* PROPS */, ["note"])) : _createCommentVNode("v-if", true), (_unref(isRenote)) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: _normalizeClass(_ctx.$style.renote)
          }, [ _createVNode(_component_MkAvatar, {
              class: _normalizeClass(_ctx.$style.renoteAvatar),
              user: _unref(note).user,
              link: "",
              preview: ""
            }, null, 8 /* PROPS */, ["user"]), _hoisted_1, _createElementVNode("span", {
              class: _normalizeClass(_ctx.$style.renoteText)
            }, [ _createVNode(_component_I18n, {
                src: _unref(i18n).ts.renotedBy,
                tag: "span"
              }, {
                user: _withCtx(() => [
                  _createVNode(_component_MkA, {
                    class: _normalizeClass(_ctx.$style.renoteName),
                    to: _unref(userPage)(_unref(note).user)
                  }, {
                    default: _withCtx(() => [
                      _createVNode(_component_MkUserName, { user: _unref(note).user }, null, 8 /* PROPS */, ["user"])
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["to"])
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["src"]) ]), _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.renoteInfo)
            }, [ _createElementVNode("button", {
                ref_key: "renoteTime", ref: renoteTime,
                class: _normalizeClass(["_button", _ctx.$style.renoteTime]),
                onMousedown: _cache[0] || (_cache[0] = _withModifiers(($event: any) => (showRenoteMenu()), ["prevent"]))
              }, [ (_unref(isMyRenote)) ? (_openBlock(), _createElementBlock("i", {
                    key: 0,
                    class: "ti ti-dots",
                    style: "margin-right: 4px;"
                  })) : _createCommentVNode("v-if", true), _createVNode(_component_MkTime, { time: _unref(note).createdAt }, null, 8 /* PROPS */, ["time"]) ], 32 /* NEED_HYDRATION */), (_unref(note).visibility !== 'public') ? (_openBlock(), _createElementBlock("span", {
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
                }, [ _hoisted_2 ])) : _createCommentVNode("v-if", true) ]) ])) : _createCommentVNode("v-if", true), (_unref(isRenote) && _unref(note).renote == null) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: _normalizeClass(_ctx.$style.deleted)
          }, _toDisplayString(_unref(i18n).ts.deletedNote), 1 /* TEXT */)) : (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [ _createElementVNode("article", {
              class: _normalizeClass(_ctx.$style.note),
              onContextmenu: _withModifiers(onContextmenu, ["stop"])
            }, [ _createElementVNode("header", {
                class: _normalizeClass(_ctx.$style.noteHeader)
              }, [ _createVNode(_component_MkAvatar, {
                  class: _normalizeClass(_ctx.$style.noteHeaderAvatar),
                  user: _unref(appearNote).user,
                  indicator: "",
                  link: "",
                  preview: ""
                }, null, 8 /* PROPS */, ["user"]), _createElementVNode("div", {
                  class: _normalizeClass(_ctx.$style.noteHeaderBody)
                }, [ _createElementVNode("div", null, [ _createVNode(_component_MkA, {
                      class: _normalizeClass(_ctx.$style.noteHeaderName),
                      to: _unref(userPage)(_unref(appearNote).user)
                    }, {
                      default: _withCtx(() => [
                        _createVNode(_component_MkUserName, {
                          nowrap: false,
                          user: _unref(appearNote).user
                        }, null, 8 /* PROPS */, ["nowrap", "user"])
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["to"]), (_unref(appearNote).user.isBot) ? (_openBlock(), _createElementBlock("span", {
                        key: 0,
                        class: _normalizeClass(_ctx.$style.isBot)
                      }, "bot")) : _createCommentVNode("v-if", true), _createElementVNode("div", {
                      class: _normalizeClass(_ctx.$style.noteHeaderInfo)
                    }, [ (_unref(appearNote).visibility !== 'public') ? (_openBlock(), _createElementBlock("span", {
                          key: 0,
                          style: "margin-left: 0.5em;",
                          title: _unref(i18n).ts._visibility[_unref(appearNote).visibility]
                        }, [ (_unref(appearNote).visibility === 'home') ? (_openBlock(), _createElementBlock("i", {
                              key: 0,
                              class: "ti ti-home"
                            })) : (_unref(appearNote).visibility === 'followers') ? (_openBlock(), _createElementBlock("i", {
                                key: 1,
                                class: "ti ti-lock"
                              })) : (_unref(appearNote).visibility === 'specified') ? (_openBlock(), _createElementBlock("i", {
                                key: 2,
                                ref: "specified",
                                class: "ti ti-mail"
                              })) : _createCommentVNode("v-if", true) ])) : _createCommentVNode("v-if", true), (_unref(appearNote).localOnly) ? (_openBlock(), _createElementBlock("span", {
                          key: 0,
                          style: "margin-left: 0.5em;",
                          title: _unref(i18n).ts._visibility['disableFederation']
                        }, [ _hoisted_3 ])) : _createCommentVNode("v-if", true) ]) ]), _createElementVNode("div", {
                    class: _normalizeClass(_ctx.$style.noteHeaderUsernameAndBadgeRoles)
                  }, [ _createElementVNode("div", {
                      class: _normalizeClass(_ctx.$style.noteHeaderUsername)
                    }, [ _createVNode(_component_MkAcct, { user: _unref(appearNote).user }, null, 8 /* PROPS */, ["user"]) ]), (_unref(appearNote).user.badgeRoles) ? (_openBlock(), _createElementBlock("div", {
                        key: 0,
                        class: _normalizeClass(_ctx.$style.noteHeaderBadgeRoles)
                      }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(appearNote).user.badgeRoles, (role, i) => {
                          return (_openBlock(), _createElementBlock("img", {
                            key: i,
                            class: _normalizeClass(_ctx.$style.noteHeaderBadgeRole),
                            src: role.iconUrl
                          }, 8 /* PROPS */, ["src"]))
                        }), 128 /* KEYED_FRAGMENT */)) ])) : _createCommentVNode("v-if", true) ]), (_unref(showTicker)) ? (_openBlock(), _createBlock(MkInstanceTicker, {
                      key: 0,
                      host: _unref(appearNote).user.host,
                      instance: _unref(appearNote).user.instance
                    }, null, 8 /* PROPS */, ["host", "instance"])) : _createCommentVNode("v-if", true) ]) ]), _createElementVNode("div", {
                class: _normalizeClass(_ctx.$style.noteContent)
              }, [ (_unref(appearNote).cw != null) ? (_openBlock(), _createElementBlock("p", {
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
                      modelValue: showContent.value,
                      "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((showContent).value = $event))
                    }, null, 8 /* PROPS */, ["text", "renote", "files", "poll", "modelValue"]) ])) : _createCommentVNode("v-if", true), _withDirectives(_createElementVNode("div", null, [ (_unref(appearNote).isHidden) ? (_openBlock(), _createElementBlock("span", {
                      key: 0,
                      style: "opacity: 0.5"
                    }, "(" + _toDisplayString(_unref(i18n).ts.private) + ")", 1 /* TEXT */)) : _createCommentVNode("v-if", true), (_unref(appearNote).replyId) ? (_openBlock(), _createBlock(_component_MkA, {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.noteReplyTarget),
                      to: `/notes/${_unref(appearNote).replyId}`
                    }, {
                      default: _withCtx(() => [
                        _hoisted_4
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["to"])) : _createCommentVNode("v-if", true), (_unref(appearNote).text) ? (_openBlock(), _createBlock(_component_Mfm, {
                      key: 0,
                      parsedNodes: _unref(parsed),
                      text: _unref(appearNote).text,
                      author: _unref(appearNote).user,
                      nyaize: 'respect',
                      emojiUrls: _unref(appearNote).emojis,
                      enableEmojiMenu: true,
                      enableEmojiMenuReaction: true,
                      class: "_selectable"
                    }, null, 8 /* PROPS */, ["parsedNodes", "text", "author", "nyaize", "emojiUrls", "enableEmojiMenu", "enableEmojiMenuReaction"])) : _createCommentVNode("v-if", true), (_unref(appearNote).renote != null) ? (_openBlock(), _createElementBlock("a", {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.rn)
                    }, "RN:")) : _createCommentVNode("v-if", true), (translating.value || translation.value) ? (_openBlock(), _createElementBlock("div", {
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
                            }, null, 8 /* PROPS */, ["text", "author", "nyaize", "emojiUrls"]) ])) : _createCommentVNode("v-if", true) ])) : _createCommentVNode("v-if", true), (_unref(appearNote).files && _unref(appearNote).files.length > 0) ? (_openBlock(), _createElementBlock("div", { key: 0 }, [ _createVNode(MkMediaList, {
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
                    }, null, 8 /* PROPS */, ["noteId", "multiple", "expiresAt", "choices", "author", "emojiUrls"])) : _createCommentVNode("v-if", true), (_unref(isEnabledUrlPreview)) ? (_openBlock(), _createElementBlock("div", { key: 0 }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(urls), (url) => {
                        return (_openBlock(), _createBlock(MkUrlPreview, {
                          key: url,
                          url: url,
                          compact: true,
                          detail: true,
                          style: "margin-top: 6px;"
                        }, null, 8 /* PROPS */, ["url", "compact", "detail"]))
                      }), 128 /* KEYED_FRAGMENT */)) ])) : _createCommentVNode("v-if", true), (_unref(appearNote).renote) ? (_openBlock(), _createElementBlock("div", {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.quote)
                    }, [ _createVNode(MkNoteSimple, {
                        note: _unref(appearNote).renote,
                        class: _normalizeClass(_ctx.$style.quoteNote)
                      }, null, 8 /* PROPS */, ["note"]) ])) : _createCommentVNode("v-if", true) ], 512 /* NEED_PATCH */), [ [_vShow, _unref(appearNote).cw == null || showContent.value] ]), (_unref(appearNote).channel && !inChannel) ? (_openBlock(), _createBlock(_component_MkA, {
                    key: 0,
                    class: _normalizeClass(_ctx.$style.channel),
                    to: `/channels/${_unref(appearNote).channel.id}`
                  }, {
                    default: _withCtx(() => [
                      _hoisted_5,
                      _createTextVNode(" "),
                      _createTextVNode(_toDisplayString(_unref(appearNote).channel.name), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["to"])) : _createCommentVNode("v-if", true) ]), _createElementVNode("footer", null, [ _createElementVNode("div", {
                  class: _normalizeClass(_ctx.$style.noteFooterInfo)
                }, [ _createVNode(_component_MkA, { to: _unref(notePage)(_unref(appearNote)) }, {
                    default: _withCtx(() => [
                      _createVNode(_component_MkTime, {
                        time: _unref(appearNote).createdAt,
                        mode: "detail",
                        colored: ""
                      }, null, 8 /* PROPS */, ["time"])
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["to"]) ]), (_unref(appearNote).reactionAcceptance !== 'likeOnly') ? (_openBlock(), _createBlock(MkReactionsViewer, {
                    key: 0,
                    style: "margin-top: 6px;",
                    reactions: _unref($appearNote).reactions,
                    reactionEmojis: _unref($appearNote).reactionEmojis,
                    myReaction: _unref($appearNote).myReaction,
                    noteId: _unref(appearNote).id
                  }, null, 8 /* PROPS */, ["reactions", "reactionEmojis", "myReaction", "noteId"])) : _createCommentVNode("v-if", true), _createElementVNode("button", {
                  class: _normalizeClass(["_button", _ctx.$style.noteFooterButton]),
                  onClick: _cache[2] || (_cache[2] = ($event: any) => (reply()))
                }, [ _hoisted_6, (_unref(appearNote).repliesCount > 0) ? (_openBlock(), _createElementBlock("p", {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.noteFooterButtonCount)
                    }, _toDisplayString(number(_unref(appearNote).repliesCount)), 1 /* TEXT */)) : _createCommentVNode("v-if", true) ]), (canRenote.value) ? (_openBlock(), _createElementBlock("button", {
                    key: 0,
                    ref: "renoteButton",
                    class: _normalizeClass(["_button", _ctx.$style.noteFooterButton]),
                    onMousedown: _cache[3] || (_cache[3] = _withModifiers(($event: any) => (renote()), ["prevent"]))
                  }, [ _hoisted_7, (_unref(appearNote).renoteCount > 0) ? (_openBlock(), _createElementBlock("p", {
                        key: 0,
                        class: _normalizeClass(_ctx.$style.noteFooterButtonCount)
                      }, _toDisplayString(number(_unref(appearNote).renoteCount)), 1 /* TEXT */)) : _createCommentVNode("v-if", true) ])) : (_openBlock(), _createElementBlock("button", {
                    key: 1,
                    class: _normalizeClass(["_button", _ctx.$style.noteFooterButton]),
                    disabled: ""
                  }, [ _hoisted_8 ])), _createElementVNode("button", {
                  ref_key: "reactButton", ref: reactButton,
                  class: _normalizeClass(["_button", _ctx.$style.noteFooterButton]),
                  onClick: _cache[4] || (_cache[4] = ($event: any) => (toggleReact()))
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
                      class: _normalizeClass(_ctx.$style.noteFooterButtonCount)
                    }, _toDisplayString(number(_unref($appearNote).reactionCount)), 1 /* TEXT */)) : _createCommentVNode("v-if", true) ], 512 /* NEED_PATCH */), (_unref(prefer).s.showClipButtonInNoteFooter) ? (_openBlock(), _createElementBlock("button", {
                    key: 0,
                    ref: "clipButton",
                    class: _normalizeClass(["_button", _ctx.$style.noteFooterButton]),
                    onMousedown: _cache[5] || (_cache[5] = _withModifiers(($event: any) => (clip()), ["prevent"]))
                  }, [ _hoisted_9 ])) : _createCommentVNode("v-if", true), _createElementVNode("button", {
                  ref_key: "menuButton", ref: menuButton,
                  class: _normalizeClass(["_button", _ctx.$style.noteFooterButton]),
                  onMousedown: _cache[6] || (_cache[6] = _withModifiers(($event: any) => (showMenu()), ["prevent"]))
                }, [ _hoisted_10 ], 32 /* NEED_HYDRATION */) ]) ], 32 /* NEED_HYDRATION */), _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.tabs)
            }, [ _createElementVNode("button", {
                class: _normalizeClass(["_button", [_ctx.$style.tab, { [_ctx.$style.tabActive]: tab.value === 'replies' }]]),
                onClick: _cache[7] || (_cache[7] = ($event: any) => (tab.value = 'replies'))
              }, [ _hoisted_11, _createTextVNode(" " + _toDisplayString(_unref(i18n).ts.replies), 1 /* TEXT */) ], 2 /* CLASS */), _createElementVNode("button", {
                class: _normalizeClass(["_button", [_ctx.$style.tab, { [_ctx.$style.tabActive]: tab.value === 'renotes' }]]),
                onClick: _cache[8] || (_cache[8] = ($event: any) => (tab.value = 'renotes'))
              }, [ _hoisted_12, _createTextVNode(" " + _toDisplayString(_unref(i18n).ts.renotes), 1 /* TEXT */) ], 2 /* CLASS */), _createElementVNode("button", {
                class: _normalizeClass(["_button", [_ctx.$style.tab, { [_ctx.$style.tabActive]: tab.value === 'reactions' }]]),
                onClick: _cache[9] || (_cache[9] = ($event: any) => (tab.value = 'reactions'))
              }, [ _hoisted_13, _createTextVNode(" " + _toDisplayString(_unref(i18n).ts.reactions), 1 /* TEXT */) ], 2 /* CLASS */) ]), _createElementVNode("div", null, [ (tab.value === 'replies') ? (_openBlock(), _createElementBlock("div", { key: 0 }, [ (!repliesLoaded.value) ? (_openBlock(), _createElementBlock("div", {
                      key: 0,
                      style: "padding: 16px"
                    }, [ _createVNode(MkButton, {
                        style: "margin: 0 auto;",
                        primary: "",
                        rounded: "",
                        onClick: loadReplies
                      }, {
                        default: _withCtx(() => [
                          _createTextVNode(_toDisplayString(_unref(i18n).ts.loadReplies), 1 /* TEXT */)
                        ]),
                        _: 1 /* STABLE */
                      }) ])) : _createCommentVNode("v-if", true), (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(replies.value, (note) => {
                    return (_openBlock(), _createBlock(MkNoteSub, {
                      key: _unref(note).id,
                      note: _unref(note),
                      class: _normalizeClass(_ctx.$style.reply),
                      detail: true
                    }, null, 8 /* PROPS */, ["note", "detail"]))
                  }), 128 /* KEYED_FRAGMENT */)) ])) : (tab.value === 'renotes') ? (_openBlock(), _createElementBlock("div", {
                    key: 1,
                    class: _normalizeClass(_ctx.$style.tab_renotes)
                  }, [ _createVNode(MkPagination, {
                      paginator: _unref(renotesPaginator),
                      forceDisableInfiniteScroll: true
                    }, {
                      default: _withCtx(({ items }) => [
                        _createElementVNode("div", { style: "display: grid; grid-template-columns: repeat(auto-fill, minmax(270px, 1fr)); grid-gap: 12px;" }, [
                          (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (item) => {
                            return (_openBlock(), _createBlock(_component_MkA, {
                              key: item.id,
                              to: _unref(userPage)(item.user)
                            }, {
                              default: _withCtx(() => [
                                _createVNode(MkUserCardMini, {
                                  user: item.user,
                                  withChart: false
                                }, null, 8 /* PROPS */, ["user", "withChart"])
                              ]),
                              _: 2 /* DYNAMIC */
                            }, 1032 /* PROPS, DYNAMIC_SLOTS */, ["to"]))
                          }), 128 /* KEYED_FRAGMENT */))
                        ])
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["paginator", "forceDisableInfiniteScroll"]) ])) : (tab.value === 'reactions') ? (_openBlock(), _createElementBlock("div", {
                    key: 2,
                    class: _normalizeClass(_ctx.$style.tab_reactions)
                  }, [ _createElementVNode("div", {
                      class: _normalizeClass(_ctx.$style.reactionTabs)
                    }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(Object.keys(_unref($appearNote).reactions), (reaction) => {
                        return (_openBlock(), _createElementBlock("button", {
                          key: reaction,
                          class: _normalizeClass(["_button", [_ctx.$style.reactionTab, { [_ctx.$style.reactionTabActive]: reactionTabType.value === reaction }]]),
                          onClick: _cache[10] || (_cache[10] = ($event: any) => (reactionTabType.value = reaction))
                        }, [
                          _createVNode(MkReactionIcon, { reaction: reaction }, null, 8 /* PROPS */, ["reaction"]),
                          _createElementVNode("span", _hoisted_14, _toDisplayString(_unref($appearNote).reactions[reaction]), 1 /* TEXT */)
                        ], 2 /* CLASS */))
                      }), 128 /* KEYED_FRAGMENT */)) ]), (reactionTabType.value) ? (_openBlock(), _createBlock(MkPagination, {
                        key: reactionTabType.value,
                        paginator: _unref(reactionsPaginator),
                        forceDisableInfiniteScroll: true
                      }, {
                        default: _withCtx(({ items }) => [
                          _createElementVNode("div", { style: "display: grid; grid-template-columns: repeat(auto-fill, minmax(270px, 1fr)); grid-gap: 12px;" }, [
                            (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (item) => {
                              return (_openBlock(), _createBlock(_component_MkA, {
                                key: item.id,
                                to: _unref(userPage)(item.user)
                              }, {
                                default: _withCtx(() => [
                                  _createVNode(MkUserCardMini, {
                                    user: item.user,
                                    withChart: false
                                  }, null, 8 /* PROPS */, ["user", "withChart"])
                                ]),
                                _: 2 /* DYNAMIC */
                              }, 1032 /* PROPS, DYNAMIC_SLOTS */, ["to"]))
                            }), 128 /* KEYED_FRAGMENT */))
                          ])
                        ]),
                        _: 1 /* STABLE */
                      }, 8 /* PROPS */, ["paginator", "forceDisableInfiniteScroll"])) : _createCommentVNode("v-if", true) ])) : _createCommentVNode("v-if", true) ]) ], 64 /* STABLE_FRAGMENT */)) ]))
      : (muted.value)
        ? (_openBlock(), _createElementBlock("div", {
          key: 1,
          class: _normalizeClass(["_panel", _ctx.$style.muted]),
          onClick: _cache[11] || (_cache[11] = ($event: any) => (muted.value = false))
        }, [ _createVNode(_component_I18n, {
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
          }, 8 /* PROPS */, ["src"]) ]))
      : _createCommentVNode("v-if", true)
}
}

})
