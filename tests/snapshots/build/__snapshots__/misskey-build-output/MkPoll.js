import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-check", style: "margin-right: 4px; color: var(--MI_THEME-accent);" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("span", null, " · ")
import { computed, ref, watch } from 'vue'
import * as Misskey from 'misskey-js'
import { host } from '@@/js/config.js'
import type { OpenOnRemoteOptions } from '@/utility/please-login.js'
import { sum } from '@/utility/array.js'
import { pleaseLogin } from '@/utility/please-login.js'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { i18n } from '@/i18n.js'
import { useLowresTime } from '@/composables/use-lowres-time.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkPoll',
  props: {
    noteId: { type: String, required: true },
    multiple: { type: null, required: true },
    expiresAt: { type: null, required: true },
    choices: { type: null, required: true },
    readOnly: { type: Boolean, required: false },
    emojiUrls: { type: null, required: false },
    author: { type: null, required: false }
  },
  setup(__props: any) {

const props = __props
const now = useLowresTime();
const expiresAtTime = computed(() => props.expiresAt ? new Date(props.expiresAt).getTime() : null);
const remaining = computed(() => {
	if (expiresAtTime.value == null) return -1;
	return Math.floor(Math.max(expiresAtTime.value - now.value, 0) / 1000);
});
const total = computed(() => sum(props.choices.map(x => x.votes)));
const closed = computed(() => props.expiresAt != null && remaining.value <= 0);
const isVoted = computed(() => !props.multiple && props.choices.some(c => c.isVoted));
const timer = computed(() => i18n.tsx._poll[
	remaining.value >= 86400 ? 'remainingDays' :
	remaining.value >= 3600 ? 'remainingHours' :
	remaining.value >= 60 ? 'remainingMinutes' : 'remainingSeconds'
]({
	s: Math.floor(remaining.value % 60),
	m: Math.floor(remaining.value / 60) % 60,
	h: Math.floor(remaining.value / 3600) % 24,
	d: Math.floor(remaining.value / 86400),
}));
const showResult = ref(props.readOnly || isVoted.value || closed.value);
if (!closed.value) {
	const closedWatchStop = watch(closed, (isNowClosed) => {
		if (isNowClosed) {
			showResult.value = true;
			closedWatchStop();
		}
	});
}
const pleaseLoginContext = computed<OpenOnRemoteOptions>(() => ({
	type: 'lookup',
	url: `https://${host}/notes/${props.noteId}`,
}));
const vote = async (id: number) => {
	if (props.readOnly || closed.value || isVoted.value) return;

	const isLoggedIn = await pleaseLogin({ openOnRemote: pleaseLoginContext.value });
	if (!isLoggedIn) return;

	const { canceled } = await os.confirm({
		type: 'question',
		text: i18n.tsx.voteConfirm({ choice: props.choices[id].text }),
	});
	if (canceled) return;

	await misskeyApi('notes/polls/vote', {
		noteId: props.noteId,
		choice: id,
	});
	if (!showResult.value) showResult.value = !props.multiple;
};

return (_ctx: any,_cache: any) => {
  const _component_Mfm = _resolveComponent("Mfm")

  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass({ [_ctx.$style.done]: closed.value || isVoted.value })
    }, [ _createElementVNode("ul", {
        class: _normalizeClass(_ctx.$style.choices)
      }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(__props.choices, (choice, i) => {
          return (_openBlock(), _createElementBlock("li", {
            key: i,
            class: _normalizeClass(_ctx.$style.choice),
            onClick: _cache[0] || (_cache[0] = ($event: any) => (vote(i)))
          }, [
            _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.bg),
              style: _normalizeStyle({ 'width': `${showResult.value ? (choice.votes / total.value * 100) : 0}%` })
            }, null, 4 /* STYLE */),
            _createElementVNode("span", {
              class: _normalizeClass(_ctx.$style.fg)
            }, [
              (choice.isVoted)
                ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
                  _hoisted_1
                ], 64 /* STABLE_FRAGMENT */))
                : _createCommentVNode("v-if", true),
              _createVNode(_component_Mfm, {
                text: choice.text,
                plain: true,
                author: __props.author,
                emojiUrls: __props.emojiUrls
              }, null, 8 /* PROPS */, ["text", "plain", "author", "emojiUrls"]),
              (showResult.value)
                ? (_openBlock(), _createElementBlock("span", {
                  key: 0,
                  style: "margin-left: 4px; opacity: 0.7;"
                }, "(" + _toDisplayString(_unref(i18n).tsx._poll.votesCount({ n: choice.votes })) + ")", 1 /* TEXT */))
                : _createCommentVNode("v-if", true)
            ])
          ]))
        }), 128 /* KEYED_FRAGMENT */)) ]), (!__props.readOnly) ? (_openBlock(), _createElementBlock("p", {
          key: 0,
          class: _normalizeClass(_ctx.$style.info)
        }, [ _createElementVNode("span", null, _toDisplayString(_unref(i18n).tsx._poll.totalVotes({ n: total.value })), 1 /* TEXT */), _hoisted_2, (!closed.value && !isVoted.value) ? (_openBlock(), _createElementBlock("a", {
              key: 0,
              style: "color: inherit;",
              onClick: _cache[1] || (_cache[1] = ($event: any) => (showResult.value = !showResult.value))
            }, _toDisplayString(showResult.value ? _unref(i18n).ts._poll.vote : _unref(i18n).ts._poll.showResult), 1 /* TEXT */)) : _createCommentVNode("v-if", true), (isVoted.value) ? (_openBlock(), _createElementBlock("span", { key: 0 }, _toDisplayString(_unref(i18n).ts._poll.voted), 1 /* TEXT */)) : (closed.value) ? (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(_unref(i18n).ts._poll.closed), 1 /* TEXT */)) : _createCommentVNode("v-if", true), (remaining.value > 0) ? (_openBlock(), _createElementBlock("span", { key: 0 }, " · " + _toDisplayString(timer.value), 1 /* TEXT */)) : _createCommentVNode("v-if", true) ])) : _createCommentVNode("v-if", true) ], 2 /* CLASS */))
}
}

})
