import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDynamicComponent as _resolveDynamicComponent, resolveDirective as _resolveDirective, withDirectives as _withDirectives, renderList as _renderList, renderSlot as _renderSlot, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, vShow as _vShow } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-circle-arrow-up" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-chevron-up" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("span", { style: "height: 1em; width: 1px; background: var(--MI_THEME-divider);" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-chevron-down" })
import { computed, watch, onUnmounted, provide, useTemplateRef, TransitionGroup, onMounted, shallowRef, ref, markRaw } from 'vue'
import * as Misskey from 'misskey-js'
import { useInterval } from '@@/js/use-interval.js'
import { useDocumentVisibility } from '@@/js/use-document-visibility.js'
import { getScrollContainer, scrollToTop } from '@@/js/scroll.js'
import type { BasicTimelineType } from '@/timelines.js'
import type { SoundStore } from '@/preferences/def.js'
import type { IPaginator, MisskeyEntity } from '@/utility/paginator.js'
import MkPullToRefresh from '@/components/MkPullToRefresh.vue'
import { useStream } from '@/stream.js'
import * as sound from '@/utility/sound.js'
import { $i } from '@/i.js'
import { instance } from '@/instance.js'
import { prefer } from '@/preferences.js'
import { store } from '@/store.js'
import MkNote from '@/components/MkNote.vue'
import MkButton from '@/components/MkButton.vue'
import { i18n } from '@/i18n.js'
import { globalEvents, useGlobalEvent } from '@/events.js'
import { isSeparatorNeeded, getSeparatorInfo } from '@/utility/timeline-date-separate.js'
import { Paginator } from '@/utility/paginator.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkStreamingNotesTimeline',
  props: {
    src: { type: [null, String], required: true },
    list: { type: String, required: false },
    antenna: { type: String, required: false },
    channel: { type: String, required: false },
    role: { type: String, required: false },
    sound: { type: Boolean, required: false, default: false },
    customSound: { type: null, required: false, default: null },
    withRenotes: { type: Boolean, required: false, default: true },
    withReplies: { type: Boolean, required: false, default: false },
    withSensitive: { type: Boolean, required: false, default: true },
    onlyFiles: { type: Boolean, required: false, default: false }
  },
  setup(__props: any, { expose: __expose }) {

const props = __props
provide('inTimeline', true);
provide('tl_withSensitive', computed(() => props.withSensitive));
provide('inChannel', computed(() => props.src === 'channel'));
let paginator: IPaginator<Misskey.entities.Note>;
if (props.src === 'antenna') {
	paginator = markRaw(new Paginator('antennas/notes', {
		computedParams: computed(() => ({
			antennaId: props.antenna!,
		})),
		useShallowRef: true,
	}));
} else if (props.src === 'home') {
	paginator = markRaw(new Paginator('notes/timeline', {
		computedParams: computed(() => ({
			withRenotes: props.withRenotes,
			withFiles: props.onlyFiles ? true : undefined,
		})),
		useShallowRef: true,
	}));
} else if (props.src === 'local') {
	paginator = markRaw(new Paginator('notes/local-timeline', {
		computedParams: computed(() => ({
			withRenotes: props.withRenotes,
			withReplies: props.withReplies,
			withFiles: props.onlyFiles ? true : undefined,
		})),
		useShallowRef: true,
	}));
} else if (props.src === 'social') {
	paginator = markRaw(new Paginator('notes/hybrid-timeline', {
		computedParams: computed(() => ({
			withRenotes: props.withRenotes,
			withReplies: props.withReplies,
			withFiles: props.onlyFiles ? true : undefined,
		})),
		useShallowRef: true,
	}));
} else if (props.src === 'global') {
	paginator = markRaw(new Paginator('notes/global-timeline', {
		computedParams: computed(() => ({
			withRenotes: props.withRenotes,
			withFiles: props.onlyFiles ? true : undefined,
		})),
		useShallowRef: true,
	}));
} else if (props.src === 'mentions') {
	paginator = markRaw(new Paginator('notes/mentions', {
		useShallowRef: true,
	}));
} else if (props.src === 'directs') {
	paginator = markRaw(new Paginator('notes/mentions', {
		params: {
			visibility: 'specified',
		},
		useShallowRef: true,
	}));
} else if (props.src === 'list') {
	paginator = markRaw(new Paginator('notes/user-list-timeline', {
		computedParams: computed(() => ({
			withRenotes: props.withRenotes,
			withFiles: props.onlyFiles ? true : undefined,
			listId: props.list!,
		})),
		useShallowRef: true,
	}));
} else if (props.src === 'channel') {
	paginator = markRaw(new Paginator('channels/timeline', {
		computedParams: computed(() => ({
			channelId: props.channel!,
		})),
		useShallowRef: true,
	}));
} else if (props.src === 'role') {
	paginator = markRaw(new Paginator('roles/notes', {
		computedParams: computed(() => ({
			roleId: props.role!,
		})),
		useShallowRef: true,
	}));
} else {
	throw new Error('Unrecognized timeline type: ' + props.src);
}
onMounted(() => {
	paginator.init();
	if (paginator.computedParams) {
		watch(paginator.computedParams, () => {
			paginator.reload();
		}, { immediate: false, deep: true });
	}
});
function isTop() {
	if (scrollContainer == null) return true;
	if (rootEl.value == null) return true;
	const scrollTop = scrollContainer.scrollTop;
	const tlTop = rootEl.value.offsetTop - scrollContainer.offsetTop;
	return scrollTop <= tlTop;
}
let scrollContainer: HTMLElement | null = null;
function onScrollContainerScroll() {
	if (isTop()) {
		paginator.releaseQueue();
	}
}
const rootEl = useTemplateRef('rootEl');
watch(rootEl, (el) => {
	if (el && scrollContainer == null) {
		scrollContainer = getScrollContainer(el);
		if (scrollContainer == null) return;
		scrollContainer.addEventListener('scroll', onScrollContainerScroll, { passive: true }); // ほんとはscrollendにしたいけどiosが非対応
	}
}, { immediate: true });
onUnmounted(() => {
	if (scrollContainer) {
		scrollContainer.removeEventListener('scroll', onScrollContainerScroll);
	}
});
const visibility = useDocumentVisibility();
let isPausingUpdate = false;
watch(visibility, () => {
	if (visibility.value === 'hidden') {
		isPausingUpdate = true;
	} else { // 'visible'
		isPausingUpdate = false;
		if (isTop()) {
			releaseQueue();
		}
	}
});
let adInsertionCounter = 0;
const MIN_POLLING_INTERVAL = 1000 * 10;
const POLLING_INTERVAL =
	prefer.s.pollingInterval === 1 ? MIN_POLLING_INTERVAL * 1.5 * 1.5 :
	prefer.s.pollingInterval === 2 ? MIN_POLLING_INTERVAL * 1.5 :
	prefer.s.pollingInterval === 3 ? MIN_POLLING_INTERVAL :
	MIN_POLLING_INTERVAL;
if (!store.s.realtimeMode) {
	// TODO: 先頭のノートの作成日時が1日以上前であれば流速が遅いTLと見做してインターバルを通常より延ばす
	useInterval(async () => {
		paginator.fetchNewer({
			toQueue: !isTop() || isPausingUpdate,
		});
	}, POLLING_INTERVAL, {
		immediate: false,
		afterMounted: true,
	});
	useGlobalEvent('notePosted', (note) => {
		paginator.fetchNewer({
			toQueue: !isTop() || isPausingUpdate,
		});
	});
}
useGlobalEvent('noteDeleted', (noteId) => {
	paginator.removeItem(noteId);
});
function releaseQueue() {
	paginator.releaseQueue();
	scrollToTop(rootEl.value!);
}
function prepend(note: Misskey.entities.Note & MisskeyEntity) {
	adInsertionCounter++;
	if (instance.notesPerOneAd > 0 && adInsertionCounter % instance.notesPerOneAd === 0) {
		note._shouldInsertAd_ = true;
	}
	if (isTop() && !isPausingUpdate) {
		paginator.prepend(note);
	} else {
		paginator.enqueue(note);
	}
	if (props.sound) {
		if (props.customSound) {
			sound.playMisskeySfxFile(props.customSound);
		} else {
			sound.playMisskeySfx($i && (note.userId === $i.id) ? 'noteMy' : 'note');
		}
	}
}
const stream = store.s.realtimeMode ? useStream() : null;
const connections = {
	antenna: null as Misskey.IChannelConnection<Misskey.Channels['antenna']> | null,
	homeTimeline: null as Misskey.IChannelConnection<Misskey.Channels['homeTimeline']> | null,
	localTimeline: null as Misskey.IChannelConnection<Misskey.Channels['localTimeline']> | null,
	hybridTimeline: null as Misskey.IChannelConnection<Misskey.Channels['hybridTimeline']> | null,
	globalTimeline: null as Misskey.IChannelConnection<Misskey.Channels['globalTimeline']> | null,
	main: null as Misskey.IChannelConnection<Misskey.Channels['main']> | null,
	userList: null as Misskey.IChannelConnection<Misskey.Channels['userList']> | null,
	channel: null as Misskey.IChannelConnection<Misskey.Channels['channel']> | null,
	roleTimeline: null as Misskey.IChannelConnection<Misskey.Channels['roleTimeline']> | null,
};
function connectChannel() {
	if (stream == null) return;
	if (props.src === 'antenna') {
		if (props.antenna == null) return;
		connections.antenna = stream.useChannel('antenna', {
			antennaId: props.antenna,
		});
		connections.antenna.on('note', prepend);
	} else if (props.src === 'home') {
		connections.homeTimeline = stream.useChannel('homeTimeline', {
			withRenotes: props.withRenotes,
			withFiles: props.onlyFiles ? true : undefined,
		});
		connections.main = stream.useChannel('main');
		connections.homeTimeline.on('note', prepend);
	} else if (props.src === 'local') {
		connections.localTimeline = stream.useChannel('localTimeline', {
			withRenotes: props.withRenotes,
			withReplies: props.withReplies,
			withFiles: props.onlyFiles ? true : undefined,
		});
		connections.localTimeline.on('note', prepend);
	} else if (props.src === 'social') {
		connections.hybridTimeline = stream.useChannel('hybridTimeline', {
			withRenotes: props.withRenotes,
			withReplies: props.withReplies,
			withFiles: props.onlyFiles ? true : undefined,
		});
		connections.hybridTimeline.on('note', prepend);
	} else if (props.src === 'global') {
		connections.globalTimeline = stream.useChannel('globalTimeline', {
			withRenotes: props.withRenotes,
			withFiles: props.onlyFiles ? true : undefined,
		});
		connections.globalTimeline.on('note', prepend);
	} else if (props.src === 'mentions') {
		connections.main = stream.useChannel('main');
		connections.main.on('mention', prepend);
	} else if (props.src === 'directs') {
		connections.main = stream.useChannel('main');
		connections.main.on('mention', note => {
			if (note.visibility === 'specified') {
				prepend(note);
			}
		});
	} else if (props.src === 'list') {
		if (props.list == null) return;
		connections.userList = stream.useChannel('userList', {
			withRenotes: props.withRenotes,
			withFiles: props.onlyFiles ? true : undefined,
			listId: props.list,
		});
		connections.userList.on('note', prepend);
	} else if (props.src === 'channel') {
		if (props.channel == null) return;
		connections.channel = stream.useChannel('channel', {
			channelId: props.channel,
		});
		connections.channel.on('note', prepend);
	} else if (props.src === 'role') {
		if (props.role == null) return;
		connections.roleTimeline = stream.useChannel('roleTimeline', {
			roleId: props.role,
		});
		connections.roleTimeline.on('note', prepend);
	}
}
function disconnectChannel() {
	for (const key in connections) {
		const conn = connections[key as keyof typeof connections];
		if (conn != null) {
			conn.dispose();
			connections[key as keyof typeof connections] = null;
		}
	}
}
if (store.s.realtimeMode) {
	connectChannel();
}
watch(() => [props.list, props.antenna, props.channel, props.role, props.withRenotes], () => {
	if (store.s.realtimeMode) {
		disconnectChannel();
		connectChannel();
	}
});
watch(() => props.withSensitive, reloadTimeline);
onUnmounted(() => {
	disconnectChannel();
});
function reloadTimeline() {
	return new Promise<void>((res) => {
		adInsertionCounter = 0;
		paginator.reload().then(() => {
			res();
		});
	});
}
__expose({
	reloadTimeline,
})

return (_ctx: any,_cache: any) => {
  const _component_MkLoading = _resolveComponent("MkLoading")
  const _component_MkError = _resolveComponent("MkError")
  const _component_MkResult = _resolveComponent("MkResult")
  const _component_MkAd = _resolveComponent("MkAd")
  const _directive_appear = _resolveDirective("appear")

  return (_openBlock(), _createBlock(_resolveDynamicComponent(_unref(prefer).s.enablePullToRefresh ? MkPullToRefresh : 'div'), { refresher: () => reloadTimeline() }, {
      default: _withCtx(() => [
        (_unref(paginator).fetching.value)
          ? (_openBlock(), _createBlock(_component_MkLoading, { key: 0 }))
          : (_unref(paginator).error.value)
            ? (_openBlock(), _createBlock(_component_MkError, {
              key: 1,
              onRetry: _cache[0] || (_cache[0] = ($event: any) => (_unref(paginator).init()))
            }))
          : (_unref(paginator).items.value.length === 0)
            ? (_openBlock(), _createElementBlock("div", { key: "_empty_" }, [
              _renderSlot(_ctx.$slots, "empty", {}, () => [
                _createVNode(_component_MkResult, {
                  type: "empty",
                  text: _unref(i18n).ts.noNotes
                }, null, 8 /* PROPS */, ["text"])
              ])
            ]))
          : (_openBlock(), _createElementBlock("div", {
            key: 3,
            ref: "rootEl"
          }, [
            (_unref(paginator).queuedAheadItemsCount.value > 0)
              ? (_openBlock(), _createElementBlock("div", {
                key: 0,
                class: _normalizeClass(_ctx.$style.new)
              }, [
                _createElementVNode("div", {
                  class: _normalizeClass(_ctx.$style.newBg1)
                }),
                _createElementVNode("div", {
                  class: _normalizeClass(_ctx.$style.newBg2)
                }),
                _createElementVNode("button", {
                  class: _normalizeClass(["_button", _ctx.$style.newButton]),
                  onClick: _cache[1] || (_cache[1] = ($event: any) => (releaseQueue()))
                }, [
                  _hoisted_1,
                  _createTextVNode(" " + _toDisplayString(_unref(i18n).ts.newNote), 1 /* TEXT */)
                ])
              ]))
              : _createCommentVNode("v-if", true),
            _createVNode(_resolveDynamicComponent(_unref(prefer).s.animation ? _unref(TransitionGroup) : 'div'), {
              class: _normalizeClass(_ctx.$style.notes),
              enterActiveClass: _ctx.$style.transition_x_enterActive,
              leaveActiveClass: _ctx.$style.transition_x_leaveActive,
              enterFromClass: _ctx.$style.transition_x_enterFrom,
              leaveToClass: _ctx.$style.transition_x_leaveTo,
              moveClass: _ctx.$style.transition_x_move,
              tag: "div"
            }, {
              default: _withCtx(() => [
                (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(paginator).items.value, (note, i) => {
                  return (_openBlock(), _createElementBlock(_Fragment, { key: note.id }, [
                    (i > 0 && _unref(isSeparatorNeeded)(_unref(paginator).items.value[i -1].createdAt, note.createdAt))
                      ? (_openBlock(), _createElementBlock("div", {
                        key: 0,
                        "data-scroll-anchor": note.id
                      }, [
                        _createElementVNode("div", {
                          class: _normalizeClass(_ctx.$style.date)
                        }, [
                          _createElementVNode("span", null, [
                            _hoisted_2,
                            _createTextVNode(" " + _toDisplayString(_unref(getSeparatorInfo)(_unref(paginator).items.value[i -1].createdAt, note.createdAt)?.prevText), 1 /* TEXT */)
                          ]),
                          _hoisted_3,
                          _createElementVNode("span", null, [
                            _createTextVNode(_toDisplayString(_unref(getSeparatorInfo)(_unref(paginator).items.value[i -1].createdAt, note.createdAt)?.nextText) + " ", 1 /* TEXT */),
                            _hoisted_4
                          ])
                        ]),
                        _createVNode(MkNote, {
                          class: _normalizeClass(_ctx.$style.note),
                          note: note,
                          withHardMute: true
                        }, null, 8 /* PROPS */, ["note", "withHardMute"])
                      ]))
                      : (note._shouldInsertAd_)
                        ? (_openBlock(), _createElementBlock("div", {
                          key: 1,
                          "data-scroll-anchor": note.id
                        }, [
                          _createVNode(MkNote, {
                            class: _normalizeClass(_ctx.$style.note),
                            note: note,
                            withHardMute: true
                          }, null, 8 /* PROPS */, ["note", "withHardMute"]),
                          _createElementVNode("div", {
                            class: _normalizeClass(_ctx.$style.ad)
                          }, [
                            _createVNode(_component_MkAd, { preferForms: ['horizontal', 'horizontal-big'] }, null, 8 /* PROPS */, ["preferForms"])
                          ])
                        ]))
                      : (_openBlock(), _createBlock(MkNote, {
                        key: 2,
                        class: _normalizeClass(_ctx.$style.note),
                        note: note,
                        withHardMute: true,
                        "data-scroll-anchor": note.id
                      }, null, 8 /* PROPS */, ["note", "withHardMute", "data-scroll-anchor"]))
                  ], 64 /* STABLE_FRAGMENT */))
                }), 128 /* KEYED_FRAGMENT */))
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["enterActiveClass", "leaveActiveClass", "enterFromClass", "leaveToClass", "moveClass"]),
            _withDirectives(_createElementVNode("button", {
              key: "_more_",
              disabled: _unref(paginator).fetchingOlder.value,
              class: _normalizeClass(["_button", _ctx.$style.more]),
              onClick: _cache[2] || (_cache[2] = ($event: any) => (_unref(paginator).fetchOlder))
            }, [
              (!_unref(paginator).fetchingOlder.value)
                ? (_openBlock(), _createElementBlock("div", { key: 0 }, _toDisplayString(_unref(i18n).ts.loadMore), 1 /* TEXT */))
                : (_openBlock(), _createBlock(_component_MkLoading, {
                  key: 1,
                  inline: true
                }, null, 8 /* PROPS */, ["inline"]))
            ], 8 /* PROPS */, ["disabled"]), [
              [_vShow, _unref(paginator).canFetchOlder.value]
            ])
          ]))
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["refresher"]))
}
}

})
