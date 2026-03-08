import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, renderSlot as _renderSlot, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, unref as _unref } from "vue"

import { onMounted, onUnmounted, ref, useTemplateRef } from 'vue'
import { getScrollContainer } from '@@/js/scroll.js'
import { i18n } from '@/i18n.js'
import { isHorizontalSwipeSwiping } from '@/utility/touch.js'
import { haptic } from '@/utility/haptic.js'
const SCROLL_STOP = 10;
const FIRE_THRESHOLD = 200;
const RELEASE_TRANSITION_DURATION = 200;
const PULL_BRAKE_BASE = 1.5;
const PULL_BRAKE_FACTOR = 170;

export default /*@__PURE__*/_defineComponent({
  __name: 'MkPullToRefresh',
  props: {
    refresher: { type: Function, required: true, default: () => Promise.resolve() }
  },
  emits: ["refresh"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const MAX_PULL_DISTANCE = Infinity;
const isPulling = ref(false);
const isPulledEnough = ref(false);
const isRefreshing = ref(false);
const pullDistance = ref(0);
let startScreenY: number | null = null;
const rootEl = useTemplateRef('rootEl');
let scrollEl: HTMLElement | null = null;
function getScreenY(event: TouchEvent | MouseEvent | PointerEvent): number {
	if (('touches' in event) && event.touches[0] && event.touches[0].screenY != null) {
		return event.touches[0].screenY;
	} else if ('screenY' in event) {
		return event.screenY;
	} else {
		return 0; // TSを黙らせるため
	}
}
// When at the top of the page, disable vertical overscroll so passive touch listeners can take over.
function lockDownScroll() {
	if (scrollEl == null) return;
	scrollEl.style.touchAction = 'pan-x pan-down pinch-zoom';
	scrollEl.style.overscrollBehavior = 'auto none';
}
function unlockDownScroll() {
	if (scrollEl == null) return;
	scrollEl.style.touchAction = 'auto';
	scrollEl.style.overscrollBehavior = 'auto contain';
}
function moveStartByMouse(event: MouseEvent) {
	if (event.button !== 1) return;
	if (isRefreshing.value) return;
	const scrollPos = scrollEl!.scrollTop;
	if (scrollPos !== 0) {
		unlockDownScroll();
		return;
	}
	lockDownScroll();
	event.preventDefault(); // 中クリックによるスクロール、テキスト選択などを防ぐ
	isPulling.value = true;
	startScreenY = getScreenY(event);
	pullDistance.value = 0;
	window.addEventListener('mousemove', moving, { passive: true });
	window.addEventListener('mouseup', () => {
		window.removeEventListener('mousemove', moving);
		onPullRelease();
	}, { passive: true, once: true });
}
function moveStartByTouch(event: TouchEvent) {
	if (isRefreshing.value) return;
	const scrollPos = scrollEl!.scrollTop;
	if (scrollPos !== 0) {
		unlockDownScroll();
		return;
	}
	lockDownScroll();
	isPulling.value = true;
	startScreenY = getScreenY(event);
	pullDistance.value = 0;
	window.addEventListener('touchmove', moving, { passive: true });
	window.addEventListener('touchend', () => {
		window.removeEventListener('touchmove', moving);
		onPullRelease();
	}, { passive: true, once: true });
}
function moveBySystem(to: number): Promise<void> {
	return new Promise(r => {
		const startHeight = pullDistance.value;
		const overHeight = pullDistance.value - to;
		if (overHeight < 1) {
			r();
			return;
		}
		const startTime = Date.now();
		let intervalId = window.setInterval(() => {
			const time = Date.now() - startTime;
			if (time > RELEASE_TRANSITION_DURATION) {
				pullDistance.value = to;
				window.clearInterval(intervalId);
				r();
				return;
			}
			const nextHeight = startHeight - (overHeight / RELEASE_TRANSITION_DURATION) * time;
			if (pullDistance.value < nextHeight) return;
			pullDistance.value = nextHeight;
		}, 1);
	});
}
async function fixOverContent() {
	if (pullDistance.value > FIRE_THRESHOLD) {
		await moveBySystem(FIRE_THRESHOLD);
	}
}
async function closeContent() {
	if (pullDistance.value > 0) {
		await moveBySystem(0);
	}
}
function onPullRelease() {
	startScreenY = null;
	if (isPulledEnough.value) {
		isPulledEnough.value = false;
		isRefreshing.value = true;
		fixOverContent().then(() => {
			emit('refresh');
			props.refresher().then(() => {
				refreshFinished();
			});
		});
	} else {
		closeContent().then(() => isPulling.value = false);
	}
}
function toggleScrollLockOnTouchEnd() {
	const scrollPos = scrollEl!.scrollTop;
	if (scrollPos === 0) {
		lockDownScroll();
	} else {
		unlockDownScroll();
	}
}
function moving(event: MouseEvent | TouchEvent) {
	if ((scrollEl?.scrollTop ?? 0) > SCROLL_STOP + pullDistance.value || isHorizontalSwipeSwiping.value) {
		pullDistance.value = 0;
		isPulledEnough.value = false;
		onPullRelease();
		return;
	}
	if (startScreenY === null) {
		startScreenY = getScreenY(event);
	}
	const moveScreenY = getScreenY(event);
	const moveHeight = moveScreenY - startScreenY!;
	pullDistance.value = Math.min(Math.max(moveHeight, 0), MAX_PULL_DISTANCE);
	isPulledEnough.value = pullDistance.value >= FIRE_THRESHOLD;
	if (isPulledEnough.value) haptic();
}
/**
 * emit(refresh)が完了したことを知らせる関数
 *
 * タイムアウトがないのでこれを最終的に実行しないと出たままになる
 */
function refreshFinished() {
	closeContent().then(() => {
		isPulling.value = false;
		isRefreshing.value = false;
	});
}
onMounted(() => {
	if (rootEl.value == null) return;
	scrollEl = getScrollContainer(rootEl.value);
	lockDownScroll();
	rootEl.value.addEventListener('mousedown', moveStartByMouse, { passive: false }); // preventDefaultするため
	rootEl.value.addEventListener('touchstart', moveStartByTouch, { passive: true });
	rootEl.value.addEventListener('touchend', toggleScrollLockOnTouchEnd, { passive: true });
});
onUnmounted(() => {
	unlockDownScroll();
	if (rootEl.value) rootEl.value.removeEventListener('mousedown', moveStartByMouse);
	if (rootEl.value) rootEl.value.removeEventListener('touchstart', moveStartByTouch);
	if (rootEl.value) rootEl.value.removeEventListener('touchend', toggleScrollLockOnTouchEnd);
});

return (_ctx: any,_cache: any) => {
  const _component_MkLoading = _resolveComponent("MkLoading")

  return (_openBlock(), _createElementBlock("div", {
      ref_key: "rootEl", ref: rootEl,
      class: _normalizeClass(isPulling.value ? _ctx.$style.isPulling : null)
    }, [ (isPulling.value) ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          class: _normalizeClass(_ctx.$style.frame),
          style: _normalizeStyle(`--frame-min-height: ${Math.round(pullDistance.value / (PULL_BRAKE_BASE + (pullDistance.value / PULL_BRAKE_FACTOR)))}px;`)
        }, [ _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.frameContent)
          }, [ (isRefreshing.value) ? (_openBlock(), _createBlock(_component_MkLoading, {
                key: 0,
                class: _normalizeClass(_ctx.$style.loader),
                em: true
              }, null, 8 /* PROPS */, ["em"])) : (_openBlock(), _createElementBlock("i", {
                key: 1,
                class: _normalizeClass(["ti ti-arrow-bar-to-down", [_ctx.$style.icon, { [_ctx.$style.refresh]: isPulledEnough.value }]])
              })), _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.text)
            }, [ (isPulledEnough.value) ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ _toDisplayString(_unref(i18n).ts.releaseToRefresh) ], 64 /* STABLE_FRAGMENT */)) : (isRefreshing.value) ? (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [ _toDisplayString(_unref(i18n).ts.refreshing) ], 64 /* STABLE_FRAGMENT */)) : (_openBlock(), _createElementBlock(_Fragment, { key: 2 }, [ _toDisplayString(_unref(i18n).ts.pullDownToRefresh) ], 64 /* STABLE_FRAGMENT */)) ]) ]) ])) : _createCommentVNode("v-if", true), _renderSlot(_ctx.$slots, "default") ], 2 /* CLASS */))
}
}

})
