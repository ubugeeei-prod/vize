import { defineComponent as _defineComponent } from 'vue'
import { Transition as _Transition, openBlock as _openBlock, createBlock as _createBlock, createElementVNode as _createElementVNode, resolveDirective as _resolveDirective, withDirectives as _withDirectives, renderSlot as _renderSlot, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref, vShow as _vShow, withModifiers as _withModifiers } from "vue"

import { nextTick, normalizeClass, onMounted, onUnmounted, provide, watch, ref, useTemplateRef, computed } from 'vue'
import type { Keymap } from '@/utility/hotkey.js'
import * as os from '@/os.js'
import { isTouchUsing } from '@/utility/touch.js'
import { deviceKind } from '@/utility/device-kind.js'
import { focusTrap } from '@/utility/focus-trap.js'
import { focusParent } from '@/utility/focus.js'
import { prefer } from '@/preferences.js'
import { DI } from '@/di.js'

type ModalTypes = 'popup' | 'dialog' | 'drawer';
const MARGIN = 16;
const SCROLLBAR_THICKNESS = 16;

export default /*@__PURE__*/_defineComponent({
  __name: 'MkModal',
  props: {
    manualShowing: { type: Boolean, required: false, default: null },
    anchor: { type: Object, required: false, default: () => ({ x: 'center', y: 'bottom' }) },
    anchorElement: { type: null, required: false, default: null },
    preferType: { type: [null, String], required: false, default: 'auto' },
    zPriority: { type: String, required: false, default: 'low' },
    noOverlap: { type: Boolean, required: false, default: true },
    transparentBg: { type: Boolean, required: false, default: false },
    hasInteractionWithOtherFocusTrappedEls: { type: Boolean, required: false, default: false },
    returnFocusTo: { type: null, required: false, default: null }
  },
  emits: ["opening", "opened", "click", "esc", "close", "closed"],
  setup(__props: any, { expose: __expose, emit: __emit }) {

const emit = __emit
const props = __props
function getFixedContainer(el: Element | null): Element | null {
	if (el == null || el.tagName === 'BODY') return null;
	const position = window.getComputedStyle(el).getPropertyValue('position');
	if (position === 'fixed') {
		return el;
	} else {
		return getFixedContainer(el.parentElement);
	}
}
provide(DI.inModal, true);
const maxHeight = ref<number>();
const fixed = ref(false);
const transformOrigin = ref('center');
const showing = ref(true);
const modalRootEl = useTemplateRef('modalRootEl');
const content = useTemplateRef('content');
const zIndex = os.claimZIndex(props.zPriority);
const useSendAnime = ref(false);
const type = computed<ModalTypes>(() => {
	if (props.preferType === 'auto') {
		if ((prefer.s.menuStyle === 'drawer') || (prefer.s.menuStyle === 'auto' && isTouchUsing && deviceKind === 'smartphone')) {
			return 'drawer';
		} else {
			return props.anchorElement != null ? 'popup' : 'dialog';
		}
	} else {
		return props.preferType!;
	}
});
const isEnableBgTransparent = computed(() => props.transparentBg && (type.value === 'popup'));
const transitionName = computed((() =>
	prefer.s.animation
		? useSendAnime.value
			? 'send'
			: type.value === 'drawer'
				? 'modal-drawer'
				: type.value === 'popup'
					? 'modal-popup'
					: 'modal'
		: ''
));
const transitionDuration = computed((() =>
	transitionName.value === 'send'
		? 400
		: transitionName.value === 'modal-popup'
			? 100
			: transitionName.value === 'modal'
				? 200
				: transitionName.value === 'modal-drawer'
					? 200
					: 0
));
let releaseFocusTrap: (() => void) | null = null;
let contentClicking = false;
function close(opts: { useSendAnimation?: boolean } = {}) {
	if (opts.useSendAnimation) {
		useSendAnime.value = true;
	}
	if (props.anchorElement) props.anchorElement.style.pointerEvents = 'auto';
	showing.value = false;
	emit('close');
}
function onBgClick() {
	if (contentClicking) return;
	emit('click');
}
if (type.value === 'drawer') {
	maxHeight.value = window.innerHeight / 1.5;
}
const keymap = {
	'esc': {
		allowRepeat: true,
		callback: () => emit('esc'),
	},
} as const satisfies Keymap;
const align = () => {
	if (props.anchorElement == null) return;
	if (type.value === 'drawer') return;
	if (type.value === 'dialog') return;

	if (content.value == null) return;

	const anchorRect = props.anchorElement.getBoundingClientRect();

	const width = content.value!.offsetWidth;
	const height = content.value!.offsetHeight;

	let left = 0;
	let top = 0;

	const x = anchorRect.left + (fixed.value ? 0 : window.scrollX);
	const y = anchorRect.top + (fixed.value ? 0 : window.scrollY);

	if (props.anchor.x === 'center') {
		left = x + (props.anchorElement.offsetWidth / 2) - (width / 2);
	} else if (props.anchor.x === 'left') {
		// TODO
	} else if (props.anchor.x === 'right') {
		left = x + props.anchorElement.offsetWidth;
	}

	if (props.anchor.y === 'center') {
		top = (y - (height / 2));
	} else if (props.anchor.y === 'top') {
		// TODO
	} else if (props.anchor.y === 'bottom') {
		top = y + props.anchorElement.offsetHeight;
	}

	if (fixed.value) {
		// 画面から横にはみ出る場合
		if (left + width > (window.innerWidth - SCROLLBAR_THICKNESS)) {
			left = (window.innerWidth - SCROLLBAR_THICKNESS) - width;
		}

		const underSpace = ((window.innerHeight - SCROLLBAR_THICKNESS) - MARGIN) - top;
		const upperSpace = (anchorRect.top - MARGIN);

		// 画面から縦にはみ出る場合
		if (top + height > ((window.innerHeight - SCROLLBAR_THICKNESS) - MARGIN)) {
			if (props.noOverlap && props.anchor.x === 'center') {
				if (underSpace >= (upperSpace / 3)) {
					maxHeight.value = underSpace;
				} else {
					maxHeight.value = upperSpace;
					top = (upperSpace + MARGIN) - height;
				}
			} else {
				top = ((window.innerHeight - SCROLLBAR_THICKNESS) - MARGIN) - height;
			}
		} else {
			maxHeight.value = underSpace;
		}
	} else {
		// 画面から横にはみ出る場合
		if (left + width - window.scrollX > (window.innerWidth - SCROLLBAR_THICKNESS)) {
			left = (window.innerWidth - SCROLLBAR_THICKNESS) - width + window.scrollX - 1;
		}

		const underSpace = ((window.innerHeight - SCROLLBAR_THICKNESS) - MARGIN) - (top - window.scrollY);
		const upperSpace = (anchorRect.top - MARGIN);

		// 画面から縦にはみ出る場合
		if (top + height - window.scrollY > ((window.innerHeight - SCROLLBAR_THICKNESS) - MARGIN)) {
			if (props.noOverlap && props.anchor.x === 'center') {
				if (underSpace >= (upperSpace / 3)) {
					maxHeight.value = underSpace;
				} else {
					maxHeight.value = upperSpace;
					top = window.scrollY + ((upperSpace + MARGIN) - height);
				}
			} else {
				top = ((window.innerHeight - SCROLLBAR_THICKNESS) - MARGIN) - height + window.scrollY - 1;
			}
		} else {
			maxHeight.value = underSpace;
		}
	}

	if (top < 0) {
		top = MARGIN;
	}

	if (left < 0) {
		left = 0;
	}

	let transformOriginX = 'center';
	let transformOriginY = 'center';

	if (top >= anchorRect.top + props.anchorElement.offsetHeight + (fixed.value ? 0 : window.scrollY)) {
		transformOriginY = 'top';
	} else if ((top + height) <= anchorRect.top + (fixed.value ? 0 : window.scrollY)) {
		transformOriginY = 'bottom';
	}

	if (left >= anchorRect.left + props.anchorElement.offsetWidth + (fixed.value ? 0 : window.scrollX)) {
		transformOriginX = 'left';
	} else if ((left + width) <= anchorRect.left + (fixed.value ? 0 : window.scrollX)) {
		transformOriginX = 'right';
	}

	transformOrigin.value = `${transformOriginX} ${transformOriginY}`;

	content.value.style.left = left + 'px';
	content.value.style.top = top + 'px';
};
const onOpened = () => {
	emit('opened');

	// contentの子要素にアクセスするためレンダリングの完了を待つ必要がある（nextTickが必要）
	nextTick(() => {
		// NOTE: Chromatic テストの際に undefined になる場合がある
		if (content.value == null) return;

		// モーダルコンテンツにマウスボタンが押され、コンテンツ外でマウスボタンが離されたときにモーダルバックグラウンドクリックと判定させないためにマウスイベントを監視しフラグ管理する
		const el = content.value.children[0];
		el.addEventListener('mousedown', ev => {
			contentClicking = true;
			window.addEventListener('mouseup', ev => {
				// click イベントより先に mouseup イベントが発生するかもしれないのでちょっと待つ
				window.setTimeout(() => {
					contentClicking = false;
				}, 100);
			}, { passive: true, once: true });
		}, { passive: true });
	});
};
const onClosed = () => {
	emit('closed');
};
const alignObserver = new ResizeObserver((entries, observer) => {
	align();
});
onMounted(() => {
	watch(() => props.anchorElement, async () => {
		if (props.anchorElement) {
			props.anchorElement.style.pointerEvents = 'none';
		}
		fixed.value = (type.value === 'drawer') || (getFixedContainer(props.anchorElement) != null);
		await nextTick();
		align();
	}, { immediate: true });
	watch([showing, () => props.manualShowing], ([showing, manualShowing]) => {
		if (manualShowing === true || (manualShowing == null && showing === true)) {
			if (modalRootEl.value != null) {
				const { release } = focusTrap(modalRootEl.value, props.hasInteractionWithOtherFocusTrappedEls);
				releaseFocusTrap = release;
				modalRootEl.value.focus();
			}
		} else {
			releaseFocusTrap?.();
			focusParent(props.returnFocusTo ?? props.anchorElement, true, false);
		}
	}, { immediate: true });
	nextTick(() => {
		alignObserver.observe(content.value!);
	});
});
onUnmounted(() => {
	alignObserver.disconnect();
});
__expose({
	close,
})

return (_ctx: any,_cache: any) => {
  const _directive_hotkey = _resolveDirective("hotkey")

  return (_openBlock(), _createBlock(_Transition, {
      name: transitionName.value,
      enterActiveClass: _unref(normalizeClass)({
  		[_ctx.$style.transition_modalDrawer_enterActive]: transitionName.value === 'modal-drawer',
  		[_ctx.$style.transition_modalPopup_enterActive]: transitionName.value === 'modal-popup',
  		[_ctx.$style.transition_modal_enterActive]: transitionName.value === 'modal',
  		[_ctx.$style.transition_send_enterActive]: transitionName.value === 'send',
  	}),
      leaveActiveClass: _unref(normalizeClass)({
  		[_ctx.$style.transition_modalDrawer_leaveActive]: transitionName.value === 'modal-drawer',
  		[_ctx.$style.transition_modalPopup_leaveActive]: transitionName.value === 'modal-popup',
  		[_ctx.$style.transition_modal_leaveActive]: transitionName.value === 'modal',
  		[_ctx.$style.transition_send_leaveActive]: transitionName.value === 'send',
  	}),
      enterFromClass: _unref(normalizeClass)({
  		[_ctx.$style.transition_modalDrawer_enterFrom]: transitionName.value === 'modal-drawer',
  		[_ctx.$style.transition_modalPopup_enterFrom]: transitionName.value === 'modal-popup',
  		[_ctx.$style.transition_modal_enterFrom]: transitionName.value === 'modal',
  		[_ctx.$style.transition_send_enterFrom]: transitionName.value === 'send',
  	}),
      leaveToClass: _unref(normalizeClass)({
  		[_ctx.$style.transition_modalDrawer_leaveTo]: transitionName.value === 'modal-drawer',
  		[_ctx.$style.transition_modalPopup_leaveTo]: transitionName.value === 'modal-popup',
  		[_ctx.$style.transition_modal_leaveTo]: transitionName.value === 'modal',
  		[_ctx.$style.transition_send_leaveTo]: transitionName.value === 'send',
  	}),
      duration: transitionDuration.value,
      appear: "",
      onAfterLeave: onClosed,
      onEnter: _cache[0] || (_cache[0] = ($event: any) => (emit('opening'))),
      onAfterEnter: onOpened
    }, {
      default: _withCtx(() => [
        _withDirectives(_createElementVNode("div", {
          ref_key: "modalRootEl", ref: modalRootEl,
          class: _normalizeClass([_ctx.$style.root, { [_ctx.$style.drawer]: type.value === 'drawer', [_ctx.$style.dialog]: type.value === 'dialog', [_ctx.$style.popup]: type.value === 'popup' }]),
          style: _normalizeStyle({ zIndex: _unref(zIndex), pointerEvents: (__props.manualShowing != null ? __props.manualShowing : showing.value) ? 'auto' : 'none', '--transformOrigin': transformOrigin.value })
        }, [
          _createElementVNode("div", {
            "data-cy-bg": "",
            "data-cy-transparent": isEnableBgTransparent.value,
            class: _normalizeClass(["_modalBg", [_ctx.$style.bg, { [_ctx.$style.bgTransparent]: isEnableBgTransparent.value }]]),
            style: _normalizeStyle({ zIndex: _unref(zIndex) }),
            onClick: onBgClick,
            onMousedown: onBgClick,
            onContextmenu: _cache[1] || (_cache[1] = _withModifiers(() => {}, ["prevent","stop"]))
          }, null, 46 /* CLASS, STYLE, PROPS, NEED_HYDRATION */, ["data-cy-transparent"]),
          _createElementVNode("div", {
            ref_key: "content", ref: content,
            class: _normalizeClass([_ctx.$style.content, { [_ctx.$style.fixed]: fixed.value }]),
            style: _normalizeStyle({ zIndex: _unref(zIndex) }),
            onClick: _withModifiers(onBgClick, ["self"])
          }, [
            _renderSlot(_ctx.$slots, "default", {
              "max-height": maxHeight.value,
              type: type.value
            })
          ], 6 /* CLASS, STYLE */)
        ], 6 /* CLASS, STYLE */), [
          [_vShow, __props.manualShowing != null ? __props.manualShowing : showing.value]
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["name", "enterActiveClass", "leaveActiveClass", "enterFromClass", "leaveToClass", "duration"]))
}
}

})
