import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDynamicComponent as _resolveDynamicComponent, resolveDirective as _resolveDirective, withDirectives as _withDirectives, renderList as _renderList, renderSlot as _renderSlot, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, vShow as _vShow } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-chevron-up" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("span", { style: "height: 1em; width: 1px; background: var(--MI_THEME-divider);" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-chevron-down" })
import { onUnmounted, onMounted, computed, useTemplateRef, TransitionGroup, markRaw, watch } from 'vue'
import * as Misskey from 'misskey-js'
import { notificationTypes } from 'misskey-js'
import { useInterval } from '@@/js/use-interval.js'
import { useDocumentVisibility } from '@@/js/use-document-visibility.js'
import { getScrollContainer, scrollToTop } from '@@/js/scroll.js'
import XNotification from '@/components/MkNotification.vue'
import MkNote from '@/components/MkNote.vue'
import { useStream } from '@/stream.js'
import { i18n } from '@/i18n.js'
import MkPullToRefresh from '@/components/MkPullToRefresh.vue'
import { prefer } from '@/preferences.js'
import { store } from '@/store.js'
import { isSeparatorNeeded, getSeparatorInfo } from '@/utility/timeline-date-separate.js'
import { Paginator } from '@/utility/paginator.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkStreamingNotificationsTimeline',
  props: {
    excludeTypes: { type: Array, required: false }
  },
  setup(__props: any, { expose: __expose }) {

const props = __props
const rootEl = useTemplateRef('rootEl');
const paginator = prefer.s.useGroupedNotifications ? markRaw(new Paginator('i/notifications-grouped', {
	limit: 20,
	computedParams: computed(() => ({
		excludeTypes: props.excludeTypes ?? undefined,
	})),
})) : markRaw(new Paginator('i/notifications', {
	limit: 20,
	computedParams: computed(() => ({
		excludeTypes: props.excludeTypes ?? undefined,
	})),
}));
const MIN_POLLING_INTERVAL = 1000 * 10;
const POLLING_INTERVAL =
	prefer.s.pollingInterval === 1 ? MIN_POLLING_INTERVAL * 1.5 * 1.5 :
	prefer.s.pollingInterval === 2 ? MIN_POLLING_INTERVAL * 1.5 :
	prefer.s.pollingInterval === 3 ? MIN_POLLING_INTERVAL :
	MIN_POLLING_INTERVAL;
if (!store.s.realtimeMode) {
	useInterval(async () => {
		paginator.fetchNewer({
			toQueue: false,
		});
	}, POLLING_INTERVAL, {
		immediate: false,
		afterMounted: true,
	});
}
function isTop() {
	if (scrollContainer == null) return true;
	if (rootEl.value == null) return true;
	const scrollTop = scrollContainer.scrollTop;
	const tlTop = rootEl.value.offsetTop - scrollContainer.offsetTop;
	return scrollTop <= tlTop;
}
function releaseQueue() {
	paginator.releaseQueue();
	scrollToTop(rootEl.value!);
}
let scrollContainer: HTMLElement | null = null;
function onScrollContainerScroll() {
	if (isTop()) {
		paginator.releaseQueue();
	}
}
watch(rootEl, (el) => {
	if (el && scrollContainer == null) {
		scrollContainer = getScrollContainer(el);
		if (scrollContainer == null) return;
		scrollContainer.addEventListener('scroll', onScrollContainerScroll, { passive: true }); // ほんとはscrollendにしたいけどiosが非対応
	}
}, { immediate: true });
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
function onNotification(notification: Misskey.entities.Notification) {
	const isMuted = props.excludeTypes ? props.excludeTypes.includes(notification.type as typeof notificationTypes[number]) : false;
	if (isMuted || window.document.visibilityState === 'visible') {
		if (store.s.realtimeMode) {
			useStream().send('readNotification');
		}
	}
	if (!isMuted) {
		if (isTop() && !isPausingUpdate) {
			paginator.prepend(notification);
		} else {
			paginator.enqueue(notification);
		}
	}
}
function reload() {
	return paginator.reload();
}
let connection: Misskey.IChannelConnection<Misskey.Channels['main']> | null = null;
onMounted(() => {
	paginator.init();
	if (paginator.computedParams) {
		watch(paginator.computedParams, () => {
			paginator.reload();
		}, { immediate: false, deep: true });
	}
	if (store.s.realtimeMode) {
		connection = useStream().useChannel('main');
		connection.on('notification', onNotification);
		connection.on('notificationFlushed', reload);
	}
});
onUnmounted(() => {
	if (connection) connection.dispose();
});
__expose({
	reload,
})

return (_ctx: any,_cache: any) => {
  const _component_MkLoading = _resolveComponent("MkLoading")
  const _component_MkError = _resolveComponent("MkError")
  const _component_MkResult = _resolveComponent("MkResult")
  const _directive_appear = _resolveDirective("appear")

  return (_openBlock(), _createBlock(_resolveDynamicComponent(_unref(prefer).s.enablePullToRefresh ? MkPullToRefresh : 'div'), { refresher: () => reload() }, {
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
                  text: _unref(i18n).ts.noNotifications
                }, null, 8 /* PROPS */, ["text"])
              ])
            ]))
          : (_openBlock(), _createElementBlock("div", {
            key: 3,
            ref: "rootEl"
          }, [
            _createVNode(_resolveDynamicComponent(_unref(prefer).s.animation ? _unref(TransitionGroup) : 'div'), {
              class: _normalizeClass([_ctx.$style.notifications]),
              enterActiveClass: _ctx.$style.transition_x_enterActive,
              leaveActiveClass: _ctx.$style.transition_x_leaveActive,
              enterFromClass: _ctx.$style.transition_x_enterFrom,
              leaveToClass: _ctx.$style.transition_x_leaveTo,
              moveClass: _ctx.$style.transition_x_move,
              tag: "div"
            }, {
              default: _withCtx(() => [
                (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(paginator).items.value, (notification, i) => {
                  return (_openBlock(), _createElementBlock("div", {
                    key: notification.id,
                    "data-scroll-anchor": notification.id,
                    class: _normalizeClass(_ctx.$style.item)
                  }, [
                    (i > 0 && _unref(isSeparatorNeeded)(_unref(paginator).items.value[i -1].createdAt, notification.createdAt))
                      ? (_openBlock(), _createElementBlock("div", {
                        key: 0,
                        class: _normalizeClass(_ctx.$style.date)
                      }, [
                        _createElementVNode("span", null, [
                          _hoisted_1,
                          _createTextVNode(" " + _toDisplayString(_unref(getSeparatorInfo)(_unref(paginator).items.value[i -1].createdAt, notification.createdAt)?.prevText), 1 /* TEXT */)
                        ]),
                        _hoisted_2,
                        _createElementVNode("span", null, [
                          _createTextVNode(_toDisplayString(_unref(getSeparatorInfo)(_unref(paginator).items.value[i -1].createdAt, notification.createdAt)?.nextText) + " ", 1 /* TEXT */),
                          _hoisted_3
                        ])
                      ]))
                      : _createCommentVNode("v-if", true),
                    (['reply', 'quote', 'mention'].includes(notification.type) && 'note' in notification)
                      ? (_openBlock(), _createBlock(MkNote, {
                        key: 0,
                        class: _normalizeClass(_ctx.$style.content),
                        note: notification.note,
                        withHardMute: true
                      }, null, 8 /* PROPS */, ["note", "withHardMute"]))
                      : (_openBlock(), _createBlock(XNotification, {
                        key: 1,
                        class: _normalizeClass(_ctx.$style.content),
                        notification: notification,
                        withTime: true,
                        full: true
                      }, null, 8 /* PROPS */, ["notification", "withTime", "full"]))
                  ], 8 /* PROPS */, ["data-scroll-anchor"]))
                }), 128 /* KEYED_FRAGMENT */))
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["enterActiveClass", "leaveActiveClass", "enterFromClass", "leaveToClass", "moveClass"]),
            _withDirectives(_createElementVNode("button", {
              key: "_more_",
              disabled: _unref(paginator).fetchingOlder.value,
              class: _normalizeClass(["_button", _ctx.$style.more]),
              onClick: _cache[1] || (_cache[1] = ($event: any) => (_unref(paginator).fetchOlder))
            }, [
              (!_unref(paginator).fetchingOlder.value)
                ? (_openBlock(), _createElementBlock("div", { key: 0 }, _toDisplayString(_unref(i18n).ts.loadMore), 1 /* TEXT */))
                : (_openBlock(), _createBlock(_component_MkLoading, { key: 1 }))
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
