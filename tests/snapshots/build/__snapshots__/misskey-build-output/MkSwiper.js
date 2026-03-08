import { useModel as _useModel } from 'vue'
import { defineComponent as _defineComponent } from 'vue'
import { Transition as _Transition, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, renderSlot as _renderSlot, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx } from "vue"

import { ref, useTemplateRef, computed, nextTick, watch } from 'vue'
import type { Tab } from '@/components/global/MkPageHeader.tabs.vue'
import { isHorizontalSwipeSwiping as isSwiping } from '@/utility/touch.js'
import { prefer } from '@/preferences.js'
const MIN_SWIPE_DISTANCE = 20;
const SWIPE_DISTANCE_THRESHOLD = 70;
const MAX_SWIPE_DISTANCE = 120;
const SWIPE_DIRECTION_ANGLE_THRESHOLD = 50;

export default /*@__PURE__*/_defineComponent({
  __name: 'MkSwiper',
  props: {
    tabs: { type: Array, required: true },
    "tab": {}
  },
  emits: ["swiped", "left", "right", "update:tab"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const tabModel = _useModel(__props, "tab")
const rootEl = useTemplateRef('rootEl');
const shouldAnimate = computed(() => prefer.r.enableHorizontalSwipe.value || prefer.r.animation.value);
// ▼ しきい値 ▼ //
// スワイプと判定される最小の距離
// スワイプ時の動作を発火する最小の距離
// スワイプできる最大の距離
// スワイプ方向を判定する角度の許容範囲（度数）
// ▲ しきい値 ▲ //
let startScreenX: number | null = null;
let startScreenY: number | null = null;
const currentTabIndex = computed(() => props.tabs.findIndex(tab => tab.key === tabModel.value));
const pullDistance = ref(0);
const isSwipingForClass = ref(false);
let swipeAborted = false;
let swipeDirectionLocked: 'horizontal' | 'vertical' | null = null;
function touchStart(event: TouchEvent) {
	if (!prefer.r.enableHorizontalSwipe.value) return;
	if (event.touches.length !== 1) return;
	if (hasSomethingToDoWithXSwipe(event.target as HTMLElement)) return;
	startScreenX = event.touches[0].screenX;
	startScreenY = event.touches[0].screenY;
	swipeDirectionLocked = null; // スワイプ方向をリセット
}
function touchMove(event: TouchEvent) {
	if (!prefer.r.enableHorizontalSwipe.value) return;
	if (event.touches.length !== 1) return;
	if (startScreenX == null || startScreenY == null) return;
	if (swipeAborted) return;
	if (hasSomethingToDoWithXSwipe(event.target as HTMLElement)) return;
	let distanceX = event.touches[0].screenX - startScreenX;
	let distanceY = event.touches[0].screenY - startScreenY;
	// スワイプ方向をロック
	if (!swipeDirectionLocked) {
		const angle = Math.abs(Math.atan2(distanceY, distanceX) * (180 / Math.PI));
		if (angle > 90 - SWIPE_DIRECTION_ANGLE_THRESHOLD && angle < 90 + SWIPE_DIRECTION_ANGLE_THRESHOLD) {
			swipeDirectionLocked = 'vertical';
		} else {
			swipeDirectionLocked = 'horizontal';
		}
	}
	// 縦方向のスワイプの場合は中断
	if (swipeDirectionLocked === 'vertical') {
		swipeAborted = true;
		pullDistance.value = 0;
		isSwiping.value = false;
		window.setTimeout(() => {
			isSwipingForClass.value = false;
		}, 400);
		return;
	}
	if (Math.abs(distanceX) < MIN_SWIPE_DISTANCE) return;
	if (Math.abs(distanceX) > MAX_SWIPE_DISTANCE) return;
	if (currentTabIndex.value === 0 || props.tabs[currentTabIndex.value - 1].onClick) {
		distanceX = Math.min(distanceX, 0);
	}
	if (currentTabIndex.value === props.tabs.length - 1 || props.tabs[currentTabIndex.value + 1].onClick) {
		distanceX = Math.max(distanceX, 0);
	}
	if (distanceX === 0) return;
	isSwiping.value = true;
	isSwipingForClass.value = true;
	nextTick(() => {
		// グリッチを控えるため、1.5px以上の差がないと更新しない
		if (Math.abs(distanceX - pullDistance.value) < 1.5) return;
		pullDistance.value = distanceX;
	});
}
function touchEnd(event: TouchEvent) {
	if (swipeAborted) {
		swipeAborted = false;
		return;
	}
	if (!prefer.r.enableHorizontalSwipe.value) return;
	if (event.touches.length !== 0) return;
	if (startScreenX == null) return;
	if (!isSwiping.value) return;
	if (hasSomethingToDoWithXSwipe(event.target as HTMLElement)) return;
	const distance = event.changedTouches[0].screenX - startScreenX;
	if (Math.abs(distance) > SWIPE_DISTANCE_THRESHOLD) {
		if (distance > 0) {
			if (props.tabs[currentTabIndex.value - 1] && !props.tabs[currentTabIndex.value - 1].onClick) {
				tabModel.value = props.tabs[currentTabIndex.value - 1].key;
				emit('swiped', props.tabs[currentTabIndex.value - 1].key, 'right');
			}
		} else {
			if (props.tabs[currentTabIndex.value + 1] && !props.tabs[currentTabIndex.value + 1].onClick) {
				tabModel.value = props.tabs[currentTabIndex.value + 1].key;
				emit('swiped', props.tabs[currentTabIndex.value + 1].key, 'left');
			}
		}
	}
	pullDistance.value = 0;
	isSwiping.value = false;
	window.setTimeout(() => {
		isSwipingForClass.value = false;
	}, 400);
	swipeDirectionLocked = null; // スワイプ方向をリセット
}
/** 横スワイプに関与する可能性のある要素を調べる */
function hasSomethingToDoWithXSwipe(el: HTMLElement) {
	if (['INPUT', 'TEXTAREA'].includes(el.tagName)) return true;
	if (el.isContentEditable) return true;
	if (el.scrollWidth > el.clientWidth) return true;
	const style = window.getComputedStyle(el);
	if (['absolute', 'fixed', 'sticky'].includes(style.position)) return true;
	if (['scroll', 'auto'].includes(style.overflowX)) return true;
	if (style.touchAction === 'pan-x') return true;
	if (el.parentElement && el.parentElement !== rootEl.value) {
		return hasSomethingToDoWithXSwipe(el.parentElement);
	} else {
		return false;
	}
}
const transitionName = ref<'swipeAnimationLeft' | 'swipeAnimationRight' | undefined>(undefined);
watch(tabModel, (newTab, oldTab) => {
	const newIndex = props.tabs.findIndex(tab => tab.key === newTab);
	const oldIndex = props.tabs.findIndex(tab => tab.key === oldTab);
	if (oldIndex >= 0 && newIndex >= 0 && oldIndex < newIndex) {
		transitionName.value = 'swipeAnimationLeft';
	} else {
		transitionName.value = 'swipeAnimationRight';
	}
	window.setTimeout(() => {
		transitionName.value = undefined;
	}, 400);
});

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      ref_key: "rootEl", ref: rootEl,
      class: _normalizeClass([_ctx.$style.transitionRoot, { [_ctx.$style.enableAnimation]: shouldAnimate.value }]),
      onTouchstartPassive: touchStart,
      onTouchmovePassive: touchMove,
      onTouchendPassive: touchEnd
    }, [ _createVNode(_Transition, {
        class: _normalizeClass([_ctx.$style.transitionChildren, { [_ctx.$style.swiping]: isSwipingForClass.value }]),
        enterActiveClass: _ctx.$style.swipeAnimation_enterActive,
        leaveActiveClass: _ctx.$style.swipeAnimation_leaveActive,
        enterFromClass: transitionName.value === 'swipeAnimationLeft' ? _ctx.$style.swipeAnimationLeft_enterFrom : _ctx.$style.swipeAnimationRight_enterFrom,
        leaveToClass: transitionName.value === 'swipeAnimationLeft' ? _ctx.$style.swipeAnimationLeft_leaveTo : _ctx.$style.swipeAnimationRight_leaveTo,
        style: _normalizeStyle(`--swipe: ${pullDistance.value}px;`)
      }, {
        default: _withCtx(() => [
          _createElementVNode("div", { key: tabModel.value }, [
            _renderSlot(_ctx.$slots, "default")
          ])
        ]),
        _: 1 /* STABLE */
      }, 14 /* CLASS, STYLE, PROPS */, ["enterActiveClass", "leaveActiveClass", "enterFromClass", "leaveToClass"]) ], 34 /* CLASS, NEED_HYDRATION */))
}
}

})
