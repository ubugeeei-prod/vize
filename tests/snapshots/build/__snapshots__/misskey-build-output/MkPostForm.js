import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, withDirectives as _withDirectives, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref, vShow as _vShow, vModelText as _vModelText, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-x" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-world" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-home" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-lock" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-mail" })
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-device-tv" })
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-rocket" })
const _hoisted_8 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-rocket-off" })
const _hoisted_9 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-dots" })
const _hoisted_10 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-quote" })
const _hoisted_11 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-x" })
const _hoisted_12 = { style: "margin-right: 8px;" }
const _hoisted_13 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-x" })
const _hoisted_14 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-plus ti-fw" })
const _hoisted_15 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-photo-plus" })
const _hoisted_16 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-cloud-download" })
const _hoisted_17 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-chart-arrows" })
const _hoisted_18 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-eye-off" })
const _hoisted_19 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-hash" })
const _hoisted_20 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-at" })
const _hoisted_21 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-palette" })
const _hoisted_22 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-plug" })
const _hoisted_23 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-mood-happy" })
import { watch, nextTick, onMounted, defineAsyncComponent, provide, shallowRef, ref, computed, useTemplateRef, onUnmounted, onBeforeUnmount } from 'vue'
import * as mfm from 'mfm-js'
import * as Misskey from 'misskey-js'
import insertTextAtCursor from 'insert-text-at-cursor'
import { toASCII } from 'punycode.js'
import { host, url } from '@@/js/config.js'
import MkUploaderItems from './MkUploaderItems.vue'
import type { ShallowRef } from 'vue'
import type { PostFormProps } from '@/types/post-form.js'
import type { MenuItem } from '@/types/menu.js'
import type { PollEditorModelValue } from '@/components/MkPollEditor.vue'
import type { UploaderItem } from '@/composables/use-uploader.js'
import MkNotePreview from '@/components/MkNotePreview.vue'
import XPostFormAttaches from '@/components/MkPostFormAttaches.vue'
import XTextCounter from '@/components/MkPostForm.TextCounter.vue'
import MkPollEditor from '@/components/MkPollEditor.vue'
import MkNoteSimple from '@/components/MkNoteSimple.vue'
import { erase, unique } from '@/utility/array.js'
import { extractMentions } from '@/utility/extract-mentions.js'
import { formatTimeString } from '@/utility/format-time-string.js'
import { Autocomplete } from '@/utility/autocomplete.js'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { chooseDriveFile } from '@/utility/drive.js'
import { store } from '@/store.js'
import MkInfo from '@/components/MkInfo.vue'
import { i18n } from '@/i18n.js'
import { instance } from '@/instance.js'
import { ensureSignin, notesCount, incNotesCount } from '@/i.js'
import { getAccounts, getAccountMenu } from '@/accounts.js'
import { deepClone } from '@/utility/clone.js'
import MkRippleEffect from '@/components/MkRippleEffect.vue'
import { miLocalStorage } from '@/local-storage.js'
import { claimAchievement } from '@/utility/achievements.js'
import { emojiPicker } from '@/utility/emoji-picker.js'
import { mfmFunctionPicker } from '@/utility/mfm-function-picker.js'
import { prefer } from '@/preferences.js'
import { getPluginHandlers } from '@/plugin.js'
import { DI } from '@/di.js'
import { globalEvents } from '@/events.js'
import { checkDragDataType, getDragData } from '@/drag-and-drop.js'
import { useUploader } from '@/composables/use-uploader.js'
import { startTour } from '@/utility/tour.js'
import { closeTip } from '@/tips.js'

type StoredDrafts = {
	[key: string]: {
		updatedAt: string;
		data: {
			text: string;
			useCw: boolean;
			cw: string | null;
			visibility: 'public' | 'home' | 'followers' | 'specified';
			localOnly: boolean;
			files: Misskey.entities.DriveFile[];
			poll: PollEditorModelValue | null;
			visibleUserIds?: string[];
			quoteId: string | null;
			reactionAcceptance: 'likeOnly' | 'likeOnlyForRemote' | 'nonSensitiveOnly' | 'nonSensitiveOnlyForLocalLikeOnlyForRemote' | null;
			scheduledAt: number | null;
		};
	};
};
const maxCwTextLength = 100;
const pastedFileName = 'yyyy-MM-dd HH-mm-ss [{{number}}]';

export default /*@__PURE__*/_defineComponent({
  __name: 'MkPostForm',
  props: {
    fixed: { type: Boolean, required: false },
    autofocus: { type: Boolean, required: false, default: true },
    freezeAfterPosted: { type: Boolean, required: false },
    mock: { type: Boolean, required: false, default: false }
  },
  emits: ["posted", "cancel", "esc", "fileChangeSensitive"],
  setup(__props: any, { expose: __expose, emit: __emit }) {

const emit = __emit
const props = __props
const $i = ensureSignin();
provide(DI.mock, props.mock);
const textareaEl = useTemplateRef('textareaEl');
const cwInputEl = useTemplateRef('cwInputEl');
const hashtagsInputEl = useTemplateRef('hashtagsInputEl');
const visibilityButton = useTemplateRef('visibilityButton');
const otherSettingsButton = useTemplateRef('otherSettingsButton');
const accountMenuEl = useTemplateRef('accountMenuEl');
const footerEl = useTemplateRef('footerEl');
const submitButtonEl = useTemplateRef('submitButtonEl');
const posting = ref(false);
const posted = ref(false);
const text = ref(props.initialText ?? '');
const files = ref(props.initialFiles ?? []);
const poll = ref<PollEditorModelValue | null>(null);
const useCw = ref<boolean>(!!props.initialCw);
const showPreview = ref(store.s.showPreview);
watch(showPreview, () => store.set('showPreview', showPreview.value));
const showAddMfmFunction = ref(prefer.s.enableQuickAddMfmFunction);
watch(showAddMfmFunction, () => prefer.commit('enableQuickAddMfmFunction', showAddMfmFunction.value));
const cw = ref<string | null>(props.initialCw ?? null);
const localOnly = ref(props.initialLocalOnly ?? (prefer.s.rememberNoteVisibility ? store.s.localOnly : prefer.s.defaultNoteLocalOnly));
const visibility = ref(props.initialVisibility ?? (prefer.s.rememberNoteVisibility ? store.s.visibility : prefer.s.defaultNoteVisibility));
const visibleUsers = ref<Misskey.entities.UserDetailed[]>([]);
if (props.initialVisibleUsers) {
	props.initialVisibleUsers.forEach(u => pushVisibleUser(u));
}
const reactionAcceptance = ref(store.s.reactionAcceptance);
const scheduledAt = ref<number | null>(null);
const draghover = ref(false);
const quoteId = ref<string | null>(null);
const hasNotSpecifiedMentions = ref(false);
const recentHashtags = ref(JSON.parse(miLocalStorage.getItem('hashtags') ?? '[]'));
const imeText = ref('');
const showingOptions = ref(false);
const textAreaReadOnly = ref(false);
const justEndedComposition = ref(false);
const renoteTargetNote: ShallowRef<PostFormProps['renote'] | null> = shallowRef(props.renote);
const replyTargetNote: ShallowRef<PostFormProps['reply'] | null> = shallowRef(props.reply);
const targetChannel = shallowRef(props.channel);
const serverDraftId = ref<string | null>(null);
const postFormActions = getPluginHandlers('post_form_action');
let textAutocomplete: Autocomplete | null = null;
let cwAutocomplete: Autocomplete | null = null;
let hashtagAutocomplete: Autocomplete | null = null;
const uploader = useUploader({
	multiple: true,
});
onUnmounted(() => {
	uploader.dispose();
});
uploader.events.on('itemUploaded', ctx => {
	files.value.push(ctx.item.uploaded!);
	uploader.removeItem(ctx.item);
});
const draftKey = computed((): string => {
	let key = targetChannel.value ? `channel:${targetChannel.value.id}` : '';

	if (renoteTargetNote.value) {
		key += `renote:${renoteTargetNote.value.id}`;
	} else if (replyTargetNote.value) {
		key += `reply:${replyTargetNote.value.id}`;
	} else {
		key += `note:${$i.id}`;
	}

	return key;
});
const placeholder = computed((): string => {
	if (renoteTargetNote.value) {
		return i18n.ts._postForm.quotePlaceholder;
	} else if (replyTargetNote.value) {
		return i18n.ts._postForm.replyPlaceholder;
	} else if (targetChannel.value) {
		return i18n.ts._postForm.channelPlaceholder;
	} else {
		const xs = [
			i18n.ts._postForm._placeholders.a,
			i18n.ts._postForm._placeholders.b,
			i18n.ts._postForm._placeholders.c,
			i18n.ts._postForm._placeholders.d,
			i18n.ts._postForm._placeholders.e,
			i18n.ts._postForm._placeholders.f,
		];
		return xs[Math.floor(Math.random() * xs.length)];
	}
});
const submitText = computed((): string => {
	return scheduledAt.value != null
		? i18n.ts.schedule
		: renoteTargetNote.value
			? i18n.ts.quote
			: replyTargetNote.value
				? i18n.ts.reply
				: i18n.ts.note;
});
const submitIcon = computed((): string => {
	return posted.value ? 'ti ti-check' : scheduledAt.value != null ? 'ti ti-calendar-time' : replyTargetNote.value ? 'ti ti-arrow-back-up' : renoteTargetNote.value ? 'ti ti-quote' : 'ti ti-send';
});
const textLength = computed((): number => {
	return (text.value + imeText.value).length;
});
const maxTextLength = computed((): number => {
	return instance ? instance.maxNoteTextLength : 1000;
});
const cwTextLength = computed((): number => {
	return cw.value?.length ?? 0;
});
const canPost = computed((): boolean => {
	return !props.mock && !posting.value && !posted.value && !uploader.uploading.value && (uploader.items.value.length === 0 || uploader.readyForUpload.value) &&
		(
			1 <= textLength.value ||
			1 <= files.value.length ||
			1 <= uploader.items.value.length ||
			poll.value != null ||
			renoteTargetNote.value != null ||
			quoteId.value != null
		) &&
		(textLength.value <= maxTextLength.value) &&
		(
			useCw.value ?
				(
					cw.value != null && cw.value.trim() !== '' &&
					cwTextLength.value <= maxCwTextLength
				) : true
		) &&
		(files.value.length <= 16) &&
		(!poll.value || poll.value.choices.length >= 2);
});
// cannot save pure renote as draft
const canSaveAsServerDraft = computed((): boolean => {
	return canPost.value && (textLength.value > 0 || files.value.length > 0 || poll.value != null);
});
const withHashtags = store.model('postFormWithHashtags');
const hashtags = store.model('postFormHashtags');
watch(text, () => {
	checkMissingMention();
}, { immediate: true });
watch(visibility, () => {
	checkMissingMention();
}, { immediate: true });
watch(visibleUsers, () => {
	checkMissingMention();
}, {
	deep: true,
});
if (props.mention) {
	text.value = props.mention.host ? `@${props.mention.username}@${toASCII(props.mention.host)}` : `@${props.mention.username}`;
	text.value += ' ';
}
if (replyTargetNote.value && (replyTargetNote.value.user.username !== $i.username || (replyTargetNote.value.user.host != null && replyTargetNote.value.user.host !== host))) {
	text.value = `@${replyTargetNote.value.user.username}${replyTargetNote.value.user.host != null ? '@' + toASCII(replyTargetNote.value.user.host) : ''} `;
}
if (replyTargetNote.value && replyTargetNote.value.text != null) {
	const ast = mfm.parse(replyTargetNote.value.text);
	const otherHost = replyTargetNote.value.user.host;
	for (const x of extractMentions(ast)) {
		const mention = x.host ?
			`@${x.username}@${toASCII(x.host)}` :
			(otherHost == null || otherHost === host) ?
				`@${x.username}` :
				`@${x.username}@${toASCII(otherHost)}`;
		// 自分は除外
		if ($i.username === x.username && (x.host == null || x.host === host)) continue;
		// 重複は除外
		if (text.value.includes(`${mention} `)) continue;
		text.value += `${mention} `;
	}
}
if ($i.isSilenced && visibility.value === 'public') {
	visibility.value = 'home';
}
if (targetChannel.value) {
	visibility.value = 'public';
	localOnly.value = true; // TODO: チャンネルが連合するようになった折には消す
}
// 公開以外へのリプライ時は元の公開範囲を引き継ぐ
if (replyTargetNote.value && ['home', 'followers', 'specified'].includes(replyTargetNote.value.visibility)) {
	if (replyTargetNote.value.visibility === 'home' && visibility.value === 'followers') {
		visibility.value = 'followers';
	} else if (['home', 'followers'].includes(replyTargetNote.value.visibility) && visibility.value === 'specified') {
		visibility.value = 'specified';
	} else {
		visibility.value = replyTargetNote.value.visibility;
	}
	if (visibility.value === 'specified') {
		if (replyTargetNote.value.visibleUserIds) {
			misskeyApi('users/show', {
				userIds: replyTargetNote.value.visibleUserIds.filter(uid => uid !== $i.id && uid !== replyTargetNote.value?.userId),
			}).then(users => {
				users.forEach(u => pushVisibleUser(u));
			});
		}
		if (replyTargetNote.value.userId !== $i.id) {
			misskeyApi('users/show', { userId: replyTargetNote.value.userId }).then(user => {
				pushVisibleUser(user);
			});
		}
	}
}
if (props.specified) {
	visibility.value = 'specified';
	pushVisibleUser(props.specified);
}
// keep cw when reply
if (prefer.s.keepCw && replyTargetNote.value && replyTargetNote.value.cw) {
	useCw.value = true;
	cw.value = replyTargetNote.value.cw;
}
function watchForDraft() {
	watch(text, () => saveDraft());
	watch(useCw, () => saveDraft());
	watch(cw, () => saveDraft());
	watch(poll, () => saveDraft());
	watch(files, () => saveDraft(), { deep: true });
	watch(visibility, () => saveDraft());
	watch(localOnly, () => saveDraft());
	watch(quoteId, () => saveDraft());
	watch(reactionAcceptance, () => saveDraft());
	watch(scheduledAt, () => saveDraft());
}
function checkMissingMention() {
	if (visibility.value === 'specified') {
		const ast = mfm.parse(text.value);
		for (const x of extractMentions(ast)) {
			if (!visibleUsers.value.some(u => (u.username === x.username) && (u.host === x.host))) {
				hasNotSpecifiedMentions.value = true;
				return;
			}
		}
	}
	hasNotSpecifiedMentions.value = false;
}
function addMissingMention() {
	const ast = mfm.parse(text.value);
	for (const x of extractMentions(ast)) {
		if (!visibleUsers.value.some(u => (u.username === x.username) && (u.host === x.host))) {
			misskeyApi('users/show', { username: x.username, host: x.host }).then(user => {
				pushVisibleUser(user);
			});
		}
	}
}
function togglePoll() {
	if (poll.value) {
		poll.value = null;
	} else {
		poll.value = {
			choices: ['', ''],
			multiple: false,
			expiresAt: null,
			expiredAfter: null,
		};
	}
}
function addTag(tag: string) {
	if (textareaEl.value == null) return;
	insertTextAtCursor(textareaEl.value, ` #${tag} `);
}
function focus() {
	if (textareaEl.value) {
		textareaEl.value.focus();
		textareaEl.value.setSelectionRange(textareaEl.value.value.length, textareaEl.value.value.length);
	}
}
function chooseFileFromPc(ev: PointerEvent) {
	if (props.mock) return;
	os.chooseFileFromPc({ multiple: true }).then(files => {
		if (files.length === 0) return;
		uploader.addFiles(files);
	});
}
function chooseFileFromDrive(ev: PointerEvent) {
	if (props.mock) return;
	chooseDriveFile({ multiple: true }).then(driveFiles => {
		files.value.push(...driveFiles);
	});
}
function detachFile(id: Misskey.entities.DriveFile['id']) {
	files.value = files.value.filter(x => x.id !== id);
}
function updateFileSensitive(file: Misskey.entities.DriveFile, isSensitive: boolean) {
	if (props.mock) {
		emit('fileChangeSensitive', file.id, isSensitive);
	}
	files.value[files.value.findIndex(x => x.id === file.id)].isSensitive = isSensitive;
}
function updateFileName(file: Misskey.entities.DriveFile, name: Misskey.entities.DriveFile['name']) {
	files.value[files.value.findIndex(x => x.id === file.id)].name = name;
}
function setVisibility() {
	if (targetChannel.value) {
		visibility.value = 'public';
		localOnly.value = true; // TODO: チャンネルが連合するようになった折には消す
		return;
	}
	const { dispose } = os.popup(defineAsyncComponent(() => import('@/components/MkVisibilityPicker.vue')), {
		currentVisibility: visibility.value,
		isSilenced: $i.isSilenced,
		anchorElement: visibilityButton.value,
		...(replyTargetNote.value ? { isReplyVisibilitySpecified: replyTargetNote.value.visibility === 'specified' } : {}),
	}, {
		changeVisibility: v => {
			visibility.value = v;
			if (prefer.s.rememberNoteVisibility) {
				store.set('visibility', visibility.value);
			}
		},
		closed: () => dispose(),
	});
}
async function toggleLocalOnly() {
	if (targetChannel.value) {
		visibility.value = 'public';
		localOnly.value = true; // TODO: チャンネルが連合するようになった折には消す
		return;
	}
	const neverShowInfo = miLocalStorage.getItem('neverShowLocalOnlyInfo');
	if (!localOnly.value && neverShowInfo !== 'true') {
		const confirm = await os.actions({
			type: 'question',
			title: i18n.ts.disableFederationConfirm,
			text: i18n.ts.disableFederationConfirmWarn,
			actions: [
				{
					value: 'yes' as const,
					text: i18n.ts.disableFederationOk,
					primary: true,
				},
				{
					value: 'neverShow' as const,
					text: `${i18n.ts.disableFederationOk} (${i18n.ts.neverShow})`,
					danger: true,
				},
				{
					value: 'no' as const,
					text: i18n.ts.cancel,
				},
			],
		});
		if (confirm.canceled) return;
		if (confirm.result === 'no') return;
		if (confirm.result === 'neverShow') {
			miLocalStorage.setItem('neverShowLocalOnlyInfo', 'true');
		}
	}
	localOnly.value = !localOnly.value;
	if (prefer.s.rememberNoteVisibility) {
		store.set('localOnly', localOnly.value);
	}
}
async function toggleReactionAcceptance() {
	const select = await os.select({
		title: i18n.ts.reactionAcceptance,
		items: [
			{ value: null, label: i18n.ts.all },
			{ value: 'likeOnlyForRemote' as const, label: i18n.ts.likeOnlyForRemote },
			{ value: 'nonSensitiveOnly' as const, label: i18n.ts.nonSensitiveOnly },
			{ value: 'nonSensitiveOnlyForLocalLikeOnlyForRemote' as const, label: i18n.ts.nonSensitiveOnlyForLocalLikeOnlyForRemote },
			{ value: 'likeOnly' as const, label: i18n.ts.likeOnly },
		],
		default: reactionAcceptance.value,
	});
	if (select.canceled) return;
	reactionAcceptance.value = select.result;
}
//#region その他の設定メニューpopup
function showOtherSettings() {
	let reactionAcceptanceIcon = 'ti ti-icons';
	let reactionAcceptanceCaption = '';
	switch (reactionAcceptance.value) {
		case 'likeOnly':
			reactionAcceptanceIcon = 'ti ti-heart _love';
			reactionAcceptanceCaption = i18n.ts.likeOnly;
			break;
		case 'likeOnlyForRemote':
			reactionAcceptanceIcon = 'ti ti-heart-plus';
			reactionAcceptanceCaption = i18n.ts.likeOnlyForRemote;
			break;
		case 'nonSensitiveOnly':
			reactionAcceptanceCaption = i18n.ts.nonSensitiveOnly;
			break;
		case 'nonSensitiveOnlyForLocalLikeOnlyForRemote':
			reactionAcceptanceCaption = i18n.ts.nonSensitiveOnlyForLocalLikeOnlyForRemote;
			break;
		default:
			reactionAcceptanceCaption = i18n.ts.all;
			break;
	}
	const menuItems = [{
		type: 'component',
		component: XTextCounter,
		props: {
			textLength: textLength,
		},
	}, { type: 'divider' }, {
		icon: reactionAcceptanceIcon,
		text: i18n.ts.reactionAcceptance,
		caption: reactionAcceptanceCaption,
		action: () => {
			toggleReactionAcceptance();
		},
	}, { type: 'divider' }, {
		type: 'button',
		text: i18n.ts._drafts.saveToDraft,
		icon: 'ti ti-cloud-upload',
		action: async () => {
			if (!canSaveAsServerDraft.value) {
				return os.alert({
					type: 'error',
					text: i18n.ts._drafts.cannotCreateDraft,
				});
			}
			saveServerDraft();
		},
	}, ...($i.policies.scheduledNoteLimit > 0 ? [{
		icon: 'ti ti-calendar-time',
		text: i18n.ts.schedulePost + '...',
		action: () => {
			schedule();
		},
	}] : []), { type: 'divider' }, {
		type: 'switch',
		icon: 'ti ti-eye',
		text: i18n.ts.preview,
		ref: showPreview,
	}, {
		icon: 'ti ti-trash',
		text: i18n.ts.reset,
		danger: true,
		action: async () => {
			if (props.mock) return;
			const { canceled } = await os.confirm({
				type: 'question',
				text: i18n.ts.resetAreYouSure,
			});
			if (canceled) return;
			clear();
		},
	}] satisfies MenuItem[];
	os.popupMenu(menuItems, otherSettingsButton.value);
}
//#endregion
function pushVisibleUser(user: Misskey.entities.UserDetailed) {
	if (!visibleUsers.value.some(u => u.username === user.username && u.host === user.host)) {
		visibleUsers.value.push(user);
	}
}
function addVisibleUser() {
	os.selectUser().then(user => {
		pushVisibleUser(user);
		if (!text.value.toLowerCase().includes(`@${user.username.toLowerCase()}`)) {
			text.value = `@${Misskey.acct.toString(user)} ${text.value}`;
		}
	});
}
function removeVisibleUser(id: string) {
	visibleUsers.value = visibleUsers.value.filter(u => u.id !== id);
}
function clear() {
	text.value = '';
	cw.value = null;
	files.value = [];
	poll.value = null;
	quoteId.value = null;
	scheduledAt.value = null;
}
function onKeydown(ev: KeyboardEvent) {
	if (ev.key === 'Enter' && (ev.ctrlKey || ev.metaKey) && canPost.value) post();
	// justEndedComposition.value is for Safari, which keyDown occurs after compositionend.
	// ev.isComposing is for another browsers.
	if (ev.key === 'Escape' && !justEndedComposition.value && !ev.isComposing) emit('esc');
}
function onKeyup(ev: KeyboardEvent) {
	justEndedComposition.value = false;
}
function onCompositionUpdate(ev: CompositionEvent) {
	imeText.value = ev.data;
}
function onCompositionEnd(ev: CompositionEvent) {
	imeText.value = '';
	justEndedComposition.value = true;
}
async function onPaste(ev: ClipboardEvent) {
	if (props.mock) return;
	if (ev.clipboardData == null) return;
	if (textareaEl.value == null) return;
	let pastedFiles: File[] = [];
	for (const { item, i } of Array.from(ev.clipboardData.items, (data, x) => ({ item: data, i: x }))) {
		if (item.kind === 'file') {
			const file = item.getAsFile();
			if (!file) continue;
			const lio = file.name.lastIndexOf('.');
			const ext = lio >= 0 ? file.name.slice(lio) : '';
			const formattedName = `${formatTimeString(new Date(file.lastModified), pastedFileName).replace(/{{number}}/g, `${i + 1}`)}${ext}`;
			const renamedFile = new File([file], formattedName, { type: file.type });
			pastedFiles.push(renamedFile);
		}
	}
	if (pastedFiles.length > 0) {
		ev.preventDefault();
		uploader.addFiles(pastedFiles);
		return;
	}
	const paste = ev.clipboardData.getData('text');
	if (!renoteTargetNote.value && !quoteId.value && paste.startsWith(url + '/notes/')) {
		ev.preventDefault();
		const { canceled } = await os.confirm({
			type: 'info',
			text: i18n.ts.quoteQuestion,
		});
		if (canceled) {
			insertTextAtCursor(textareaEl.value, paste);
			return;
		}
		quoteId.value = paste.substring(url.length).match(/^\/notes\/(.+?)\/?$/)?.[1] ?? null;
	}
	if (paste.length > 1000) {
		ev.preventDefault();
		const { canceled } = await os.confirm({
			type: 'info',
			text: i18n.ts.attachAsFileQuestion,
		});
		if (canceled) {
			insertTextAtCursor(textareaEl.value, paste);
			return;
		}
		const fileName = formatTimeString(new Date(), pastedFileName).replace(/{{number}}/g, '0');
		const file = new File([paste], `${fileName}.txt`, { type: 'text/plain' });
		uploader.addFiles([file]);
	}
}
function onDragover(ev: DragEvent) {
	if (ev.dataTransfer == null) return;
	if (ev.dataTransfer.items[0] == null) return;
	const isFile = ev.dataTransfer.items[0].kind === 'file';
	if (isFile || checkDragDataType(ev, ['driveFiles'])) {
		ev.preventDefault();
		draghover.value = true;
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
function onDragenter() {
	draghover.value = true;
}
function onDragleave() {
	draghover.value = false;
}
function onDrop(ev: DragEvent): void {
	draghover.value = false;
	// ファイルだったら
	if (ev.dataTransfer && ev.dataTransfer.files.length > 0) {
		ev.preventDefault();
		uploader.addFiles(Array.from(ev.dataTransfer.files));
		return;
	}
	//#region ドライブのファイル
	{
		const droppedData = getDragData(ev, 'driveFiles');
		if (droppedData != null) {
			files.value.push(...droppedData);
			ev.preventDefault();
		}
	}
	//#endregion
}
function saveDraft() {
	if (props.instant || props.mock) return;
	const draftsData = JSON.parse(miLocalStorage.getItem('drafts') ?? '{}') as StoredDrafts;
	draftsData[draftKey.value] = {
		updatedAt: new Date().toISOString(),
		data: {
			text: text.value,
			useCw: useCw.value,
			cw: cw.value,
			visibility: visibility.value,
			localOnly: localOnly.value,
			files: files.value,
			poll: poll.value,
			...( visibleUsers.value.length > 0 ? { visibleUserIds: visibleUsers.value.map(x => x.id) } : {}),
			quoteId: quoteId.value,
			reactionAcceptance: reactionAcceptance.value,
			scheduledAt: scheduledAt.value,
		},
	};
	miLocalStorage.setItem('drafts', JSON.stringify(draftsData));
}
function deleteDraft() {
	const draftsData = JSON.parse(miLocalStorage.getItem('drafts') ?? '{}') as StoredDrafts;
	delete draftsData[draftKey.value];
	miLocalStorage.setItem('drafts', JSON.stringify(draftsData));
}
async function saveServerDraft(options: {
	isActuallyScheduled?: boolean;
} = {}) {
	return await os.apiWithDialog(serverDraftId.value == null ? 'notes/drafts/create' : 'notes/drafts/update', {
		...(serverDraftId.value == null ? {} : { draftId: serverDraftId.value }),
		text: text.value,
		cw: useCw.value ? cw.value || null : null,
		visibility: visibility.value,
		localOnly: localOnly.value,
		hashtag: hashtags.value,
		fileIds: files.value.map(f => f.id),
		poll: poll.value,
		visibleUserIds: visibleUsers.value.map(x => x.id),
		renoteId: renoteTargetNote.value ? renoteTargetNote.value.id : quoteId.value ? quoteId.value : null,
		replyId: replyTargetNote.value ? replyTargetNote.value.id : null,
		channelId: targetChannel.value ? targetChannel.value.id : null,
		reactionAcceptance: reactionAcceptance.value,
		scheduledAt: scheduledAt.value,
		isActuallyScheduled: options.isActuallyScheduled ?? false,
	});
}
function isAnnoying(text: string): boolean {
	return text.includes('$[x2') ||
		text.includes('$[x3') ||
		text.includes('$[x4') ||
		text.includes('$[scale') ||
		text.includes('$[position');
}
async function uploadFiles() {
	await uploader.upload();
	for (const uploadedItem of uploader.items.value.filter(x => x.uploaded != null)) {
		files.value.push(uploadedItem.uploaded!);
		uploader.removeItem(uploadedItem);
	}
}
async function post(ev?: PointerEvent) {
	if (ev != null) {
		const el = (ev.currentTarget ?? ev.target) as HTMLElement | null;
		if (el && prefer.s.animation) {
			const rect = el.getBoundingClientRect();
			const x = rect.left + (el.offsetWidth / 2);
			const y = rect.top + (el.offsetHeight / 2);
			const { dispose } = os.popup(MkRippleEffect, { x, y }, {
				end: () => dispose(),
			});
		}
	}
	if (scheduledAt.value != null) {
		if (uploader.items.value.some(x => x.uploaded == null)) {
			await uploadFiles();
			// アップロード失敗したものがあったら中止
			if (uploader.items.value.some(x => x.uploaded == null)) {
				return;
			}
		}
		await postAsScheduled();
		clear();
		return;
	}
	if (props.mock) return;
	if (visibility.value === 'public' && (
		(useCw.value && cw.value != null && cw.value.trim() !== '' && isAnnoying(cw.value)) || // CWが迷惑になる場合
		((!useCw.value || cw.value == null || cw.value.trim() === '') && text.value != null && text.value.trim() !== '' && isAnnoying(text.value)) // CWが無い かつ 本文が迷惑になる場合
	)) {
		const { canceled, result } = await os.actions({
			type: 'warning',
			text: i18n.ts.thisPostMayBeAnnoying,
			actions: [{
				value: 'home',
				text: i18n.ts.thisPostMayBeAnnoyingHome,
				primary: true,
			}, {
				value: 'cancel',
				text: i18n.ts.thisPostMayBeAnnoyingCancel,
			}, {
				value: 'ignore',
				text: i18n.ts.thisPostMayBeAnnoyingIgnore,
			}],
		});
		if (canceled) return;
		if (result === 'cancel') return;
		if (result === 'home') {
			visibility.value = 'home';
		}
	}
	if (uploader.items.value.some(x => x.uploaded == null)) {
		await uploadFiles();
		// アップロード失敗したものがあったら中止
		if (uploader.items.value.some(x => x.uploaded == null)) {
			return;
		}
	}
	let postData = {
		text: text.value === '' ? null : text.value,
		fileIds: files.value.length > 0 ? files.value.map(f => f.id) : undefined,
		replyId: replyTargetNote.value ? replyTargetNote.value.id : undefined,
		renoteId: renoteTargetNote.value ? renoteTargetNote.value.id : quoteId.value ? quoteId.value : undefined,
		channelId: targetChannel.value ? targetChannel.value.id : undefined,
		poll: poll.value,
		cw: useCw.value ? cw.value ?? '' : null,
		localOnly: visibility.value === 'specified' ? false : localOnly.value,
		visibility: visibility.value,
		visibleUserIds: visibility.value === 'specified' ? visibleUsers.value.map(u => u.id) : undefined,
		reactionAcceptance: reactionAcceptance.value,
	};
	if (withHashtags.value && hashtags.value && hashtags.value.trim() !== '') {
		const hashtags_ = hashtags.value.trim().split(' ').map(x => x.startsWith('#') ? x : '#' + x).join(' ');
		if (!postData.text) {
			postData.text = hashtags_;
		} else {
			const postTextLines = postData.text.split('\n');
			if (postTextLines[postTextLines.length - 1].trim() === '') {
				postTextLines[postTextLines.length - 1] += hashtags_;
			} else {
				postTextLines[postTextLines.length - 1] += ' ' + hashtags_;
			}
			postData.text = postTextLines.join('\n');
		}
	}
	// plugin
	const notePostInterruptors = getPluginHandlers('note_post_interruptor');
	if (notePostInterruptors.length > 0) {
		for (const interruptor of notePostInterruptors) {
			try {
				postData = await interruptor.handler(deepClone(postData)) as typeof postData;
			} catch (err) {
				console.error(err);
			}
		}
	}
	let token: string | undefined = undefined;
	if (postAccount.value) {
		const storedAccounts = await getAccounts();
		const storedAccount = storedAccounts.find(x => x.id === postAccount.value?.id);
		if (storedAccount && storedAccount.token != null) {
			token = storedAccount.token;
		} else {
			await os.alert({
				type: 'error',
				text: 'cannot find the token of the selected account.',
			});
			return;
		}
	}
	posting.value = true;
	misskeyApi('notes/create', postData, token).then((res) => {
		if (props.freezeAfterPosted) {
			posted.value = true;
		} else {
			clear();
		}
		globalEvents.emit('notePosted', res.createdNote);
		nextTick(() => {
			deleteDraft();
			emit('posted');
			if (postData.text && postData.text !== '') {
				const hashtags_ = mfm.parse(postData.text).map(x => x.type === 'hashtag' && x.props.hashtag).filter(x => x) as string[];
				const history = JSON.parse(miLocalStorage.getItem('hashtags') ?? '[]') as string[];
				miLocalStorage.setItem('hashtags', JSON.stringify(unique(hashtags_.concat(history))));
			}
			posting.value = false;
			postAccount.value = null;
			incNotesCount();
			if (notesCount === 1) {
				claimAchievement('notes1');
			}
			const text = postData.text ?? '';
			const lowerCase = text.toLowerCase();
			if ((lowerCase.includes('love') || lowerCase.includes('❤')) && lowerCase.includes('misskey')) {
				claimAchievement('iLoveMisskey');
			}
			if ([
				'https://youtu.be/Efrlqw8ytg4',
				'https://www.youtube.com/watch?v=Efrlqw8ytg4',
				'https://m.youtube.com/watch?v=Efrlqw8ytg4',
				'https://youtu.be/XVCwzwxdHuA',
				'https://www.youtube.com/watch?v=XVCwzwxdHuA',
				'https://m.youtube.com/watch?v=XVCwzwxdHuA',
				'https://open.spotify.com/track/3Cuj0mZrlLoXx9nydNi7RB',
				'https://open.spotify.com/track/7anfcaNPQWlWCwyCHmZqNy',
				'https://open.spotify.com/track/5Odr16TvEN4my22K9nbH7l',
				'https://open.spotify.com/album/5bOlxyl4igOrp2DwVQxBco',
			].some(url => text.includes(url))) {
				claimAchievement('brainDiver');
			}
			if (renoteTargetNote.value && (renoteTargetNote.value.userId === $i.id) && text.length > 0) {
				claimAchievement('selfQuote');
			}
			const date = new Date();
			const h = date.getHours();
			const m = date.getMinutes();
			const s = date.getSeconds();
			if (h >= 0 && h <= 3) {
				claimAchievement('postedAtLateNight');
			}
			if (m === 0 && s === 0) {
				claimAchievement('postedAt0min0sec');
			}
			if (serverDraftId.value != null) {
				misskeyApi('notes/drafts/delete', { draftId: serverDraftId.value });
			}
		});
	}).catch(err => {
		posting.value = false;
		os.alert({
			type: 'error',
			text: err.message + '\n' + (err as any).id,
		});
	});
}
async function postAsScheduled() {
	if (props.mock) return;
	await saveServerDraft({
		isActuallyScheduled: true,
	});
}
function cancel() {
	emit('cancel');
}
function insertMention() {
	os.selectUser({ localOnly: localOnly.value, includeSelf: true }).then(user => {
		if (textareaEl.value == null) return;
		insertTextAtCursor(textareaEl.value, '@' + Misskey.acct.toString(user) + ' ');
	});
}
async function insertEmoji(ev: PointerEvent) {
	textAreaReadOnly.value = true;
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
			textAreaReadOnly.value = false;
			nextTick(() => {
				if (textareaEl.value) {
					textareaEl.value.focus();
					textareaEl.value.setSelectionRange(pos, posEnd);
				}
			});
		},
	);
}
async function insertMfmFunction(ev: PointerEvent) {
	if (textareaEl.value == null) return;
	let pos = textareaEl.value.selectionStart ?? 0;
	let posEnd = textareaEl.value.selectionEnd ?? text.value.length;
	mfmFunctionPicker(
		ev.currentTarget ?? ev.target,
		(tag) => {
			if (pos === posEnd) {
				text.value = `${text.value.substring(0, pos)}$[${tag} ]${text.value.substring(pos)}`;
				pos += tag.length + 3;
				posEnd = pos;
			} else {
				text.value = `${text.value.substring(0, pos)}$[${tag} ${text.value.substring(pos, posEnd)}]${text.value.substring(posEnd)}`;
				pos += tag.length + 3;
				posEnd = pos;
			}
		},
		() => {
			nextTick(() => {
				if (textareaEl.value) {
					textareaEl.value.focus();
					textareaEl.value.setSelectionRange(pos, posEnd);
				}
			});
		},
	);
}
function showActions(ev: PointerEvent) {
	os.popupMenu(postFormActions.map(action => ({
		text: action.title,
		action: () => {
			action.handler({
				text: text.value,
				cw: cw.value,
			}, (key, value) => {
				if (typeof key !== 'string' || typeof value !== 'string') return;
				if (key === 'text') { text.value = value; }
				if (key === 'cw') { useCw.value = value !== null; cw.value = value; }
			});
		},
	})), ev.currentTarget ?? ev.target);
}
const postAccount = ref<Misskey.entities.UserDetailed | null>(null);
async function openAccountMenu(ev: PointerEvent) {
	if (props.mock) return;
	function showDraftsDialog(scheduled: boolean) {
		const { dispose } = os.popup(defineAsyncComponent(() => import('@/components/MkNoteDraftsDialog.vue')), {
			scheduled,
		}, {
			restore: async (draft: Misskey.entities.NoteDraft) => {
				text.value = draft.text ?? '';
				useCw.value = draft.cw != null;
				cw.value = draft.cw ?? null;
				visibility.value = draft.visibility;
				localOnly.value = draft.localOnly ?? false;
				files.value = draft.files ?? [];
				hashtags.value = draft.hashtag ?? '';
				if (draft.hashtag) withHashtags.value = true;
				if (draft.poll) {
					// 投票を一時的に空にしないと反映されないため
					poll.value = null;
					nextTick(() => {
						poll.value = {
							choices: draft.poll!.choices,
							multiple: draft.poll!.multiple,
							expiresAt: draft.poll!.expiresAt ? (new Date(draft.poll!.expiresAt)).getTime() : null,
							expiredAfter: null,
						};
					});
				}
				if (draft.visibleUserIds) {
					misskeyApi('users/show', { userIds: draft.visibleUserIds }).then(users => {
						users.forEach(u => pushVisibleUser(u));
					});
				}
				quoteId.value = draft.renoteId ?? null;
				renoteTargetNote.value = draft.renote;
				replyTargetNote.value = draft.reply;
				reactionAcceptance.value = draft.reactionAcceptance;
				scheduledAt.value = draft.scheduledAt ?? null;
				if (draft.channel) targetChannel.value = draft.channel as unknown as Misskey.entities.Channel;

				visibleUsers.value = [];
				draft.visibleUserIds?.forEach(uid => {
					if (!visibleUsers.value.some(u => u.id === uid)) {
						misskeyApi('users/show', { userId: uid }).then(user => {
							pushVisibleUser(user);
						});
					}
				});

				serverDraftId.value = draft.id;
			},
			cancel: () => {

			},
			closed: () => {
				dispose();
			},
		});
	}
	const items = await getAccountMenu({
		withExtraOperation: false,
		includeCurrentAccount: true,
		active: postAccount.value != null ? postAccount.value.id : $i.id,
		onChoose: (account) => {
			if (account.id === $i.id) {
				postAccount.value = null;
			} else {
				postAccount.value = account;
			}
		},
	});
	os.popupMenu([{
		type: 'button',
		text: i18n.ts._drafts.listDrafts,
		icon: 'ti ti-cloud-download',
		action: () => {
			showDraftsDialog(false);
		},
	}, {
		type: 'button',
		text: i18n.ts._drafts.listScheduledNotes,
		icon: 'ti ti-clock-down',
		action: () => {
			showDraftsDialog(true);
		},
	}, { type: 'divider' }, ...items], (ev.currentTarget ?? ev.target ?? undefined) as HTMLElement | undefined);
}
function showPerUploadItemMenu(item: UploaderItem, ev: PointerEvent) {
	const menu = uploader.getMenu(item);
	os.popupMenu(menu, ev.currentTarget ?? ev.target);
}
function showPerUploadItemMenuViaContextmenu(item: UploaderItem, ev: PointerEvent) {
	const menu = uploader.getMenu(item);
	os.contextMenu(menu, ev);
}
async function schedule() {
	const { canceled, result } = await os.inputDatetime({
		title: i18n.ts.schedulePost,
	});
	if (canceled) return;
	if (result.getTime() <= Date.now()) return;
	scheduledAt.value = result.getTime();
}
function cancelSchedule() {
	scheduledAt.value = null;
}
function showTour() {
	if (textareaEl.value == null ||
		footerEl.value == null ||
		accountMenuEl.value == null ||
		visibilityButton.value == null ||
		otherSettingsButton.value == null ||
		submitButtonEl.value == null) {
		return;
	}
	startTour([{
		element: textareaEl.value,
		title: i18n.ts._postForm._howToUse.content_title,
		description: i18n.ts._postForm._howToUse.content_description,
	}, {
		element: footerEl.value,
		title: i18n.ts._postForm._howToUse.toolbar_title,
		description: i18n.ts._postForm._howToUse.toolbar_description,
	}, {
		element: accountMenuEl.value,
		title: i18n.ts._postForm._howToUse.account_title,
		description: i18n.ts._postForm._howToUse.account_description,
	}, {
		element: visibilityButton.value,
		title: i18n.ts._postForm._howToUse.visibility_title,
		description: i18n.ts._postForm._howToUse.visibility_description,
	}, {
		element: otherSettingsButton.value,
		title: i18n.ts._postForm._howToUse.menu_title,
		description: i18n.ts._postForm._howToUse.menu_description,
	}, {
		element: submitButtonEl.value,
		title: i18n.ts._postForm._howToUse.submit_title,
		description: i18n.ts._postForm._howToUse.submit_description,
	}]).then(() => {
		closeTip('postForm');
	});
}
onMounted(() => {
	if (props.autofocus) {
		focus();
		nextTick(() => {
			focus();
		});
	}
	if (textareaEl.value) textAutocomplete = new Autocomplete(textareaEl.value, text);
	if (cwInputEl.value) cwAutocomplete = new Autocomplete(cwInputEl.value, cw);
	if (hashtagsInputEl.value) hashtagAutocomplete = new Autocomplete(hashtagsInputEl.value, hashtags);
	nextTick(() => {
		// 書きかけの投稿を復元
		if (!props.instant && !props.mention && !props.specified && !props.mock) {
			const draft = JSON.parse(miLocalStorage.getItem('drafts') ?? '{}')[draftKey.value] as StoredDrafts[string] | undefined;
			if (draft != null) {
				text.value = draft.data.text;
				useCw.value = draft.data.useCw;
				cw.value = draft.data.cw;
				visibility.value = draft.data.visibility;
				localOnly.value = draft.data.localOnly;
				files.value = (draft.data.files || []).filter(draftFile => draftFile);
				if (draft.data.poll) {
					poll.value = draft.data.poll;
				}
				if (draft.data.visibleUserIds) {
					misskeyApi('users/show', { userIds: draft.data.visibleUserIds }).then(users => {
						users.forEach(u => pushVisibleUser(u));
					});
				}
				quoteId.value = draft.data.quoteId;
				reactionAcceptance.value = draft.data.reactionAcceptance;
				scheduledAt.value = draft.data.scheduledAt ?? null;
			}
		}
		// 削除して編集
		if (props.initialNote) {
			const init = props.initialNote;
			text.value = init.text ? init.text : '';
			useCw.value = init.cw != null;
			cw.value = init.cw ?? null;
			visibility.value = init.visibility;
			localOnly.value = init.localOnly ?? false;
			files.value = init.files ?? [];
			if (init.poll) {
				poll.value = {
					choices: init.poll.choices.map(x => x.text),
					multiple: init.poll.multiple,
					expiresAt: init.poll.expiresAt ? (new Date(init.poll.expiresAt)).getTime() : null,
					expiredAfter: null,
				};
			}
			if (init.visibleUserIds) {
				misskeyApi('users/show', { userIds: init.visibleUserIds }).then(users => {
					users.forEach(u => pushVisibleUser(u));
				});
			}
			quoteId.value = renoteTargetNote.value ? renoteTargetNote.value.id : null;
			reactionAcceptance.value = init.reactionAcceptance;
		}
		nextTick(() => watchForDraft());
	});
});
onBeforeUnmount(() => {
	uploader.abortAll();
	if (textAutocomplete) {
		textAutocomplete.detach();
	}
	if (cwAutocomplete) {
		cwAutocomplete.detach();
	}
	if (hashtagAutocomplete) {
		hashtagAutocomplete.detach();
	}
});
async function canClose() {
	if (!uploader.allItemsUploaded.value) {
		const { canceled } = await os.confirm({
			type: 'question',
			text: i18n.ts._postForm.quitInspiteOfThereAreUnuploadedFilesConfirm,
			okText: i18n.ts.yes,
			cancelText: i18n.ts.no,
		});
		if (canceled) return false;
	}
	return true;
}
__expose({
	clear,
	abortUploader: () => uploader.abortAll(),
	canClose,
})

return (_ctx: any,_cache: any) => {
  const _component_MkEllipsis = _resolveComponent("MkEllipsis")
  const _component_MkAcct = _resolveComponent("MkAcct")
  const _component_MkTime = _resolveComponent("MkTime")
  const _component_I18n = _resolveComponent("I18n")
  const _component_MkTip = _resolveComponent("MkTip")
  const _directive_click_anime = _resolveDirective("click-anime")
  const _directive_tooltip = _resolveDirective("tooltip")

  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass([_ctx.$style.root]),
      onDragover: _withModifiers(onDragover, ["stop"]),
      onDragenter: onDragenter,
      onDragleave: onDragleave,
      onDrop: _withModifiers(onDrop, ["stop"])
    }, [ _createElementVNode("header", {
        class: _normalizeClass(_ctx.$style.header)
      }, [ _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.headerLeft)
        }, [ (!_ctx.fixed) ? (_openBlock(), _createElementBlock("button", {
              key: 0,
              class: _normalizeClass(["_button", _ctx.$style.cancel]),
              onClick: cancel
            }, [ _hoisted_1 ])) : _createCommentVNode("v-if", true), _createElementVNode("button", {
            ref_key: "accountMenuEl", ref: accountMenuEl,
            class: "_button",
            onClick: openAccountMenu
          }, [ _createElementVNode("img", {
              class: _normalizeClass(_ctx.$style.avatar),
              src: (postAccount.value ?? _unref($i)).avatarUrl,
              style: "border-radius: 100%;"
            }, null, 8 /* PROPS */, ["src"]) ], 512 /* NEED_PATCH */) ]), _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.headerRight)
        }, [ (!(targetChannel.value != null && _ctx.fixed)) ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ (targetChannel.value == null) ? (_openBlock(), _createElementBlock("button", {
                  key: 0,
                  ref: "visibilityButton",
                  class: _normalizeClass(['_button', _ctx.$style.headerRightItem, _ctx.$style.visibility]),
                  onClick: setVisibility
                }, [ (visibility.value === 'public') ? (_openBlock(), _createElementBlock("span", { key: 0 }, [ _hoisted_2 ])) : _createCommentVNode("v-if", true), (visibility.value === 'home') ? (_openBlock(), _createElementBlock("span", { key: 0 }, [ _hoisted_3 ])) : _createCommentVNode("v-if", true), (visibility.value === 'followers') ? (_openBlock(), _createElementBlock("span", { key: 0 }, [ _hoisted_4 ])) : _createCommentVNode("v-if", true), (visibility.value === 'specified') ? (_openBlock(), _createElementBlock("span", { key: 0 }, [ _hoisted_5 ])) : _createCommentVNode("v-if", true), _createElementVNode("span", {
                    class: _normalizeClass(_ctx.$style.headerRightButtonText)
                  }, _toDisplayString(_unref(i18n).ts._visibility[visibility.value]), 1 /* TEXT */) ])) : (_openBlock(), _createElementBlock("button", {
                  key: 1,
                  class: _normalizeClass(["_button", [_ctx.$style.headerRightItem, _ctx.$style.visibility]]),
                  disabled: ""
                }, [ _createElementVNode("span", null, [ _hoisted_6 ]), _createElementVNode("span", {
                    class: _normalizeClass(_ctx.$style.headerRightButtonText)
                  }, _toDisplayString(targetChannel.value.name), 1 /* TEXT */) ])) ], 64 /* STABLE_FRAGMENT */)) : _createCommentVNode("v-if", true), (visibility.value !== 'specified') ? (_openBlock(), _createElementBlock("button", {
              key: 0,
              class: _normalizeClass(["_button", [_ctx.$style.headerRightItem, { [_ctx.$style.danger]: localOnly.value }]]),
              disabled: targetChannel.value != null,
              onClick: toggleLocalOnly
            }, [ (!localOnly.value) ? (_openBlock(), _createElementBlock("span", { key: 0 }, [ _hoisted_7 ])) : (_openBlock(), _createElementBlock("span", { key: 1 }, [ _hoisted_8 ])) ])) : _createCommentVNode("v-if", true), _createElementVNode("button", {
            ref_key: "otherSettingsButton", ref: otherSettingsButton,
            class: _normalizeClass(["_button", _ctx.$style.headerRightItem]),
            onClick: showOtherSettings
          }, [ _hoisted_9 ], 512 /* NEED_PATCH */), _createElementVNode("button", {
            ref_key: "submitButtonEl", ref: submitButtonEl,
            class: _normalizeClass(["_button", _ctx.$style.submit]),
            disabled: !canPost.value,
            "data-cy-open-post-form-submit": "",
            onClick: post
          }, [ _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.submitInner)
            }, [ (posted.value) ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ ], 64 /* STABLE_FRAGMENT */)) : (posting.value) ? (_openBlock(), _createBlock(_component_MkEllipsis, { key: 1 })) : (_openBlock(), _createElementBlock(_Fragment, { key: 2 }, [ _toDisplayString(submitText.value) ], 64 /* STABLE_FRAGMENT */)), _createElementVNode("i", {
                style: "margin-left: 6px;",
                class: _normalizeClass(submitIcon.value)
              }, null, 2 /* CLASS */) ]) ], 8 /* PROPS */, ["disabled"]) ]) ]), (replyTargetNote.value) ? (_openBlock(), _createBlock(MkNoteSimple, {
          key: 0,
          class: _normalizeClass(_ctx.$style.targetNote),
          note: replyTargetNote.value
        }, null, 8 /* PROPS */, ["note"])) : _createCommentVNode("v-if", true), (renoteTargetNote.value) ? (_openBlock(), _createBlock(MkNoteSimple, {
          key: 0,
          class: _normalizeClass(_ctx.$style.targetNote),
          note: renoteTargetNote.value
        }, null, 8 /* PROPS */, ["note"])) : _createCommentVNode("v-if", true), (quoteId.value) ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          class: _normalizeClass(_ctx.$style.withQuote)
        }, [ _hoisted_10, _createTextVNode(), _toDisplayString(_unref(i18n).ts.quoteAttached), _createElementVNode("button", {
            onClick: _cache[0] || (_cache[0] = ($event: any) => { quoteId.value = null; renoteTargetNote.value = null; })
          }, [ _hoisted_11 ]) ])) : _createCommentVNode("v-if", true), (visibility.value === 'specified') ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          class: _normalizeClass(_ctx.$style.toSpecified)
        }, [ _createElementVNode("span", _hoisted_12, _toDisplayString(_unref(i18n).ts.recipient), 1 /* TEXT */), _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.visibleUsers)
          }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(visibleUsers.value, (u) => {
              return (_openBlock(), _createElementBlock("span", {
                key: u.id,
                class: _normalizeClass(_ctx.$style.visibleUser)
              }, [
                _createVNode(_component_MkAcct, { user: u }, null, 8 /* PROPS */, ["user"]),
                _createElementVNode("button", {
                  class: "_button",
                  style: "padding: 4px 8px;",
                  onClick: _cache[1] || (_cache[1] = ($event: any) => (removeVisibleUser(u.id)))
                }, [
                  _hoisted_13
                ])
              ]))
            }), 128 /* KEYED_FRAGMENT */)), _createElementVNode("button", {
              class: "_buttonPrimary",
              style: "padding: 4px; border-radius: 8px;",
              onClick: addVisibleUser
            }, [ _hoisted_14 ]) ]) ])) : _createCommentVNode("v-if", true), (!_unref(store).r.tips.value.postForm) ? (_openBlock(), _createBlock(MkInfo, {
          key: 0,
          class: _normalizeClass(_ctx.$style.showHowToUse),
          closable: "",
          onClose: _cache[2] || (_cache[2] = ($event: any) => (_unref(closeTip)('postForm')))
        }, {
          default: _withCtx(() => [
            _createElementVNode("button", {
              class: "_textButton",
              onClick: showTour
            }, _toDisplayString(_unref(i18n).ts._postForm.showHowToUse), 1 /* TEXT */)
          ]),
          _: 1 /* STABLE */
        })) : _createCommentVNode("v-if", true), (scheduledAt.value != null) ? (_openBlock(), _createBlock(MkInfo, {
          key: 0,
          class: _normalizeClass(_ctx.$style.scheduledAt)
        }, {
          default: _withCtx(() => [
            _createVNode(_component_I18n, {
              src: _unref(i18n).ts.scheduleToPostOnX,
              tag: "span"
            }, {
              x: _withCtx(() => [
                _createVNode(_component_MkTime, {
                  time: scheduledAt.value,
                  mode: 'detail',
                  style: "font-weight: bold;"
                }, null, 8 /* PROPS */, ["time", "mode"])
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["src"]),
            _createTextVNode(" - "),
            _createElementVNode("button", {
              class: "_textButton",
              onClick: _cache[3] || (_cache[3] = ($event: any) => (cancelSchedule()))
            }, _toDisplayString(_unref(i18n).ts.cancel), 1 /* TEXT */)
          ]),
          _: 1 /* STABLE */
        })) : _createCommentVNode("v-if", true), (hasNotSpecifiedMentions.value) ? (_openBlock(), _createBlock(MkInfo, {
          key: 0,
          warn: "",
          class: _normalizeClass(_ctx.$style.hasNotSpecifiedMentions)
        }, {
          default: _withCtx(() => [
            _createTextVNode(_toDisplayString(_unref(i18n).ts.notSpecifiedMentionWarning), 1 /* TEXT */),
            _createTextVNode(" - "),
            _createElementVNode("button", {
              class: "_textButton",
              onClick: _cache[4] || (_cache[4] = ($event: any) => (addMissingMention()))
            }, _toDisplayString(_unref(i18n).ts.add), 1 /* TEXT */)
          ]),
          _: 1 /* STABLE */
        })) : _createCommentVNode("v-if", true), _withDirectives(_createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.cwOuter)
      }, [ _withDirectives(_createElementVNode("input", {
          ref_key: "cwInputEl", ref: cwInputEl,
          "onUpdate:modelValue": _cache[5] || (_cache[5] = ($event: any) => ((cw).value = $event)),
          class: _normalizeClass(_ctx.$style.cw),
          placeholder: _unref(i18n).ts.annotation,
          onKeydown: onKeydown,
          onKeyup: onKeyup,
          onCompositionend: onCompositionEnd
        }, null, 40 /* PROPS, NEED_HYDRATION */, ["placeholder"]), [ [_vModelText, cw.value] ]), (maxCwTextLength - cwTextLength.value < 20) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: _normalizeClass(['_acrylic', _ctx.$style.cwTextCount, { [_ctx.$style.cwTextOver]: cwTextLength.value > maxCwTextLength }])
          }, _toDisplayString(maxCwTextLength - cwTextLength.value), 1 /* TEXT */)) : _createCommentVNode("v-if", true) ], 512 /* NEED_PATCH */), [ [_vShow, useCw.value] ]), _createElementVNode("div", {
        class: _normalizeClass([_ctx.$style.textOuter, { [_ctx.$style.withCw]: useCw.value }])
      }, [ (targetChannel.value) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: _normalizeClass(_ctx.$style.colorBar),
            style: _normalizeStyle({ background: targetChannel.value.color })
          })) : _createCommentVNode("v-if", true), _withDirectives(_createElementVNode("textarea", {
          ref_key: "textareaEl", ref: textareaEl,
          "onUpdate:modelValue": _cache[6] || (_cache[6] = ($event: any) => ((text).value = $event)),
          class: _normalizeClass([_ctx.$style.text]),
          disabled: posting.value || posted.value,
          readonly: textAreaReadOnly.value,
          placeholder: placeholder.value,
          "data-cy-post-form-text": "",
          onKeydown: onKeydown,
          onKeyup: onKeyup,
          onPaste: onPaste,
          onCompositionupdate: onCompositionUpdate,
          onCompositionend: onCompositionEnd
        }, null, 40 /* PROPS, NEED_HYDRATION */, ["disabled", "readonly", "placeholder"]), [ [_vModelText, text.value] ]), (maxTextLength.value - textLength.value < 100) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: _normalizeClass(['_acrylic', _ctx.$style.textCount, { [_ctx.$style.textOver]: textLength.value > maxTextLength.value }])
          }, _toDisplayString(maxTextLength.value - textLength.value), 1 /* TEXT */)) : _createCommentVNode("v-if", true) ], 2 /* CLASS */), _withDirectives(_createElementVNode("input", {
        ref_key: "hashtagsInputEl", ref: hashtagsInputEl,
        "onUpdate:modelValue": _cache[7] || (_cache[7] = ($event: any) => ((hashtags).value = $event)),
        class: _normalizeClass(_ctx.$style.hashtags),
        placeholder: _unref(i18n).ts.hashtags,
        list: "hashtags"
      }, null, 8 /* PROPS */, ["placeholder"]), [ [_vModelText, _unref(hashtags)] ]), _createVNode(XPostFormAttaches, {
        onDetach: detachFile,
        onChangeSensitive: updateFileSensitive,
        onChangeName: updateFileName,
        modelValue: files.value,
        "onUpdate:modelValue": _cache[8] || (_cache[8] = ($event: any) => ((files).value = $event))
      }, null, 8 /* PROPS */, ["modelValue"]), (_unref(uploader).items.value.length > 0) ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          style: "padding: 12px;"
        }, [ _createVNode(_component_MkTip, { k: "postFormUploader" }, {
            default: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts._postForm.uploaderTip), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }), _createVNode(MkUploaderItems, {
            items: _unref(uploader).items.value,
            onShowMenu: _cache[9] || (_cache[9] = (item, ev) => showPerUploadItemMenu(item, ev)),
            onShowMenuViaContextmenu: _cache[10] || (_cache[10] = (item, ev) => showPerUploadItemMenuViaContextmenu(item, ev))
          }, null, 8 /* PROPS */, ["items"]) ])) : _createCommentVNode("v-if", true), (poll.value) ? (_openBlock(), _createBlock(MkPollEditor, {
          key: 0,
          onDestroyed: _cache[11] || (_cache[11] = ($event: any) => (poll.value = null)),
          modelValue: poll.value,
          "onUpdate:modelValue": _cache[12] || (_cache[12] = ($event: any) => ((poll).value = $event))
        }, null, 8 /* PROPS */, ["modelValue"])) : _createCommentVNode("v-if", true), (showPreview.value) ? (_openBlock(), _createBlock(MkNotePreview, {
          key: 0,
          class: _normalizeClass(_ctx.$style.preview),
          text: text.value,
          files: files.value,
          poll: poll.value ?? undefined,
          useCw: useCw.value,
          cw: cw.value,
          user: postAccount.value ?? _unref($i)
        }, null, 8 /* PROPS */, ["text", "files", "poll", "useCw", "cw", "user"])) : _createCommentVNode("v-if", true), (showingOptions.value) ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          style: "padding: 8px 16px;"
        })) : _createCommentVNode("v-if", true), _createElementVNode("footer", {
        ref_key: "footerEl", ref: footerEl,
        class: _normalizeClass(_ctx.$style.footer)
      }, [ _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.footerLeft)
        }, [ _createElementVNode("button", {
            class: _normalizeClass(["_button", _ctx.$style.footerButton]),
            onClick: chooseFileFromPc
          }, [ _hoisted_15 ]), _createElementVNode("button", {
            class: _normalizeClass(["_button", _ctx.$style.footerButton]),
            onClick: chooseFileFromDrive
          }, [ _hoisted_16 ]), _createElementVNode("button", {
            class: _normalizeClass(["_button", [_ctx.$style.footerButton, { [_ctx.$style.footerButtonActive]: poll.value }]]),
            onClick: togglePoll
          }, [ _hoisted_17 ], 2 /* CLASS */), _createElementVNode("button", {
            class: _normalizeClass(["_button", [_ctx.$style.footerButton, { [_ctx.$style.footerButtonActive]: useCw.value }]]),
            onClick: _cache[13] || (_cache[13] = ($event: any) => (useCw.value = !useCw.value))
          }, [ _hoisted_18 ], 2 /* CLASS */), _createElementVNode("button", {
            class: _normalizeClass(["_button", [_ctx.$style.footerButton, { [_ctx.$style.footerButtonActive]: _unref(withHashtags) }]]),
            onClick: _cache[14] || (_cache[14] = ($event: any) => (withHashtags.value = !_unref(withHashtags)))
          }, [ _hoisted_19 ], 2 /* CLASS */), _createElementVNode("button", {
            class: _normalizeClass(["_button", _ctx.$style.footerButton]),
            onClick: insertMention
          }, [ _hoisted_20 ]), (showAddMfmFunction.value) ? (_openBlock(), _createElementBlock("button", {
              key: 0,
              class: _normalizeClass(['_button', _ctx.$style.footerButton]),
              onClick: insertMfmFunction
            }, [ _hoisted_21 ])) : _createCommentVNode("v-if", true), (_unref(postFormActions).length > 0) ? (_openBlock(), _createElementBlock("button", {
              key: 0,
              class: _normalizeClass(["_button", _ctx.$style.footerButton]),
              onClick: showActions
            }, [ _hoisted_22 ])) : _createCommentVNode("v-if", true) ]), _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.footerRight)
        }, [ _createElementVNode("button", {
            class: _normalizeClass(['_button', _ctx.$style.footerButton]),
            onClick: insertEmoji
          }, [ _hoisted_23 ]) ]) ], 512 /* NEED_PATCH */), _createElementVNode("datalist", { id: "hashtags" }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(recentHashtags.value, (hashtag) => {
          return (_openBlock(), _createElementBlock("option", {
            key: hashtag,
            value: hashtag
          }, 8 /* PROPS */, ["value"]))
        }), 128 /* KEYED_FRAGMENT */)) ]) ], 32 /* NEED_HYDRATION */))
}
}

})
