import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, Transition as _Transition, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-chevron-up" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-device-tv" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-chevron-up" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-user" })
const _hoisted_5 = { style: "font-weight: bold; padding: 12px;" }
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-chevron-down" })
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-device-tv" })
const _hoisted_8 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-chevron-down" })
const _hoisted_9 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-user" })
import { computed, watch, ref, markRaw } from 'vue'
import * as Misskey from 'misskey-js'
import { host } from '@@/js/config.js'
import MkNoteDetailed from '@/components/MkNoteDetailed.vue'
import MkNotesTimeline from '@/components/MkNotesTimeline.vue'
import MkRemoteCaution from '@/components/MkRemoteCaution.vue'
import MkButton from '@/components/MkButton.vue'
import { misskeyApi } from '@/utility/misskey-api.js'
import { definePage } from '@/page.js'
import { i18n } from '@/i18n.js'
import { dateString } from '@/filters/date.js'
import MkClipPreview from '@/components/MkClipPreview.vue'
import { prefer } from '@/preferences.js'
import { pleaseLogin } from '@/utility/please-login.js'
import { getAppearNote } from '@/utility/get-appear-note.js'
import { serverContext, assertServerContext } from '@/server-context.js'
import { $i } from '@/i.js'
import { Paginator } from '@/utility/paginator.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'note',
  props: {
    noteId: { type: String, required: true },
    initialTab: { type: String, required: false }
  },
  setup(__props: any) {

const props = __props
// contextは非ログイン状態の情報しかないためログイン時は利用できない
const CTX_NOTE = !$i && assertServerContext(serverContext, 'note') ? serverContext.note : null;
const note = ref<null | Misskey.entities.Note>(CTX_NOTE);
const clips = ref<Misskey.entities.Clip[]>();
const showPrev = ref<'user' | 'channel' | false>(false);
const showNext = ref<'user' | 'channel' | false>(false);
const error = ref();
const prevUserPaginator = markRaw(new Paginator('users/notes', {
	limit: 10,
	initialId: props.noteId,
	computedParams: computed(() => note.value ? ({
		userId: note.value.userId,
	}) : undefined),
}));
const nextUserPaginator = markRaw(new Paginator('users/notes', {
	limit: 10,
	initialId: props.noteId,
	initialDirection: 'newer',
	computedParams: computed(() => note.value ? ({
		userId: note.value.userId,
	}) : undefined),
}));
const prevChannelPaginator = markRaw(new Paginator('channels/timeline', {
	limit: 10,
	initialId: props.noteId,
	computedParams: computed(() => note.value && note.value.channelId != null ? ({
		channelId: note.value.channelId,
	}) : undefined),
}));
const nextChannelPaginator = markRaw(new Paginator('channels/timeline', {
	limit: 10,
	initialId: props.noteId,
	initialDirection: 'newer',
	computedParams: computed(() => note.value && note.value.channelId != null ? ({
		channelId: note.value.channelId,
	}) : undefined),
}));
function fetchNote() {
	showPrev.value = false;
	showNext.value = false;
	note.value = null;
	if (CTX_NOTE && CTX_NOTE.id === props.noteId) {
		note.value = CTX_NOTE;
		return;
	}
	misskeyApi('notes/show', {
		noteId: props.noteId,
	}).then(res => {
		note.value = res;
		const appearNote = getAppearNote(res) ?? res;
		// 古いノートは被クリップ数をカウントしていないので、2023-10-01以前のものは強制的にnotes/clipsを叩く
		if ((appearNote.clippedCount ?? 0) > 0 || new Date(appearNote.createdAt).getTime() < new Date('2023-10-01').getTime()) {
			misskeyApi('notes/clips', {
				noteId: appearNote.id,
			}).then((_clips) => {
				clips.value = _clips;
			});
		}
	}).catch(err => {
		if (['fbcc002d-37d9-4944-a6b0-d9e29f2d33ab', '145f88d2-b03d-4087-8143-a78928883c4b'].includes(err.id)) {
			pleaseLogin({
				path: '/',
				message: err.id === 'fbcc002d-37d9-4944-a6b0-d9e29f2d33ab' ? i18n.ts.thisContentsAreMarkedAsSigninRequiredByAuthor : i18n.ts.signinOrContinueOnRemote,
				openOnRemote: {
					type: 'lookup',
					url: `https://${host}/notes/${props.noteId}`,
				},
			});
		}
		error.value = err;
	});
}
watch(() => props.noteId, fetchNote, {
	immediate: true,
});
const headerActions = computed(() => []);
const headerTabs = computed(() => []);
definePage(() => ({
	title: i18n.ts.note,
	...note.value ? {
		subtitle: dateString(note.value.createdAt),
		avatar: note.value.user,
		path: `/notes/${note.value.id}`,
		share: {
			title: i18n.tsx.noteOf({ user: note.value.user.name ?? note.value.user.username }),
			text: note.value.text,
		},
	} : {},
}));

return (_ctx: any,_cache: any) => {
  const _component_MkError = _resolveComponent("MkError")
  const _component_MkLoading = _resolveComponent("MkLoading")
  const _component_PageWithHeader = _resolveComponent("PageWithHeader")

  return (_openBlock(), _createBlock(_component_PageWithHeader, {
      actions: headerActions.value,
      tabs: headerTabs.value
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-w: 800px;"
        }, [
          _createVNode(_Transition, {
            name: _unref(prefer).s.animation ? 'fade' : '',
            mode: "out-in"
          }, {
            default: _withCtx(() => [
              (note.value)
                ? (_openBlock(), _createElementBlock("div", { key: 0 }, [
                  (showNext.value)
                    ? (_openBlock(), _createElementBlock("div", {
                      key: 0,
                      class: "_margin"
                    }, [
                      _createVNode(MkNotesTimeline, {
                        direction: "up",
                        withControl: false,
                        pullToRefresh: false,
                        class: "",
                        paginator: showNext.value === 'channel' ? _unref(nextChannelPaginator) : _unref(nextUserPaginator),
                        noGap: true,
                        forceDisableInfiniteScroll: true
                      }, null, 8 /* PROPS */, ["withControl", "pullToRefresh", "paginator", "noGap", "forceDisableInfiniteScroll"])
                    ]))
                    : _createCommentVNode("v-if", true),
                  _createElementVNode("div", { class: "_margin" }, [
                    (!showNext.value)
                      ? (_openBlock(), _createElementBlock("div", {
                        key: 0,
                        class: _normalizeClass(["_buttons", _ctx.$style.loadNext])
                      }, [
                        (note.value.channelId)
                          ? (_openBlock(), _createBlock(MkButton, {
                            key: 0,
                            rounded: "",
                            class: _normalizeClass(_ctx.$style.loadButton),
                            onClick: _cache[0] || (_cache[0] = ($event: any) => (showNext.value = 'channel'))
                          }, {
                            default: _withCtx(() => [
                              _hoisted_1,
                              _createTextVNode(" "),
                              _hoisted_2
                            ]),
                            _: 1 /* STABLE */
                          }))
                          : _createCommentVNode("v-if", true),
                        _createVNode(MkButton, {
                          rounded: "",
                          class: _normalizeClass(_ctx.$style.loadButton),
                          onClick: _cache[1] || (_cache[1] = ($event: any) => (showNext.value = 'user'))
                        }, {
                          default: _withCtx(() => [
                            _hoisted_3,
                            _createTextVNode(" "),
                            _hoisted_4
                          ]),
                          _: 1 /* STABLE */
                        })
                      ]))
                      : _createCommentVNode("v-if", true),
                    _createElementVNode("div", { class: "_margin _gaps_s" }, [
                      (note.value.user.host != null)
                        ? (_openBlock(), _createBlock(MkRemoteCaution, {
                          key: 0,
                          href: note.value.url ?? note.value.uri
                        }, null, 8 /* PROPS */, ["href"]))
                        : _createCommentVNode("v-if", true),
                      _createVNode(MkNoteDetailed, {
                        key: note.value.id,
                        initialTab: __props.initialTab,
                        class: _normalizeClass(_ctx.$style.note),
                        note: note.value,
                        "onUpdate:note": _cache[2] || (_cache[2] = ($event: any) => ((note).value = $event))
                      }, null, 8 /* PROPS */, ["initialTab", "note"])
                    ]),
                    (clips.value && clips.value.length > 0)
                      ? (_openBlock(), _createElementBlock("div", {
                        key: 0,
                        class: "_margin"
                      }, [
                        _createElementVNode("div", _hoisted_5, _toDisplayString(_unref(i18n).ts.clip), 1 /* TEXT */),
                        _createElementVNode("div", { class: "_gaps" }, [
                          (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(clips.value, (item) => {
                            return (_openBlock(), _createBlock(MkClipPreview, {
                              key: item.id,
                              clip: item
                            }, null, 8 /* PROPS */, ["clip"]))
                          }), 128 /* KEYED_FRAGMENT */))
                        ])
                      ]))
                      : _createCommentVNode("v-if", true),
                    (!showPrev.value)
                      ? (_openBlock(), _createElementBlock("div", {
                        key: 0,
                        class: _normalizeClass(["_buttons", _ctx.$style.loadPrev])
                      }, [
                        (note.value.channelId)
                          ? (_openBlock(), _createBlock(MkButton, {
                            key: 0,
                            rounded: "",
                            class: _normalizeClass(_ctx.$style.loadButton),
                            onClick: _cache[3] || (_cache[3] = ($event: any) => (showPrev.value = 'channel'))
                          }, {
                            default: _withCtx(() => [
                              _hoisted_6,
                              _createTextVNode(" "),
                              _hoisted_7
                            ]),
                            _: 1 /* STABLE */
                          }))
                          : _createCommentVNode("v-if", true),
                        _createVNode(MkButton, {
                          rounded: "",
                          class: _normalizeClass(_ctx.$style.loadButton),
                          onClick: _cache[4] || (_cache[4] = ($event: any) => (showPrev.value = 'user'))
                        }, {
                          default: _withCtx(() => [
                            _hoisted_8,
                            _createTextVNode(" "),
                            _hoisted_9
                          ]),
                          _: 1 /* STABLE */
                        })
                      ]))
                      : _createCommentVNode("v-if", true)
                  ]),
                  (showPrev.value)
                    ? (_openBlock(), _createElementBlock("div", {
                      key: 0,
                      class: "_margin"
                    }, [
                      _createVNode(MkNotesTimeline, {
                        withControl: false,
                        pullToRefresh: false,
                        class: "",
                        paginator: showPrev.value === 'channel' ? _unref(prevChannelPaginator) : _unref(prevUserPaginator),
                        noGap: true
                      }, null, 8 /* PROPS */, ["withControl", "pullToRefresh", "paginator", "noGap"])
                    ]))
                    : _createCommentVNode("v-if", true)
                ]))
                : (error.value)
                  ? (_openBlock(), _createBlock(_component_MkError, {
                    key: 1,
                    onRetry: _cache[5] || (_cache[5] = ($event: any) => (fetchNote()))
                  }))
                : (_openBlock(), _createBlock(_component_MkLoading, { key: 2 }))
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["name"])
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["actions", "tabs"]))
}
}

})
