import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, Transition as _Transition, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveDirective as _resolveDirective, renderList as _renderList, renderSlot as _renderSlot, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-maximize" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-minimize" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-picture-in-picture" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-rectangle" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-x" })
import { onBeforeUnmount, onMounted, provide, useTemplateRef, ref } from 'vue'
import type { MenuItem } from '@/types/menu.js'
import { elementContains } from '@/utility/element-contains.js'
import * as os from '@/os.js'
import { i18n } from '@/i18n.js'
import { prefer } from '@/preferences.js'

type WindowButton = {
	title: string;
	icon: string;
	onClick: () => void;
	highlighted?: boolean;
};
const minHeight = 50;
const minWidth = 250;

export default /*@__PURE__*/_defineComponent({
  __name: 'MkWindow',
  props: {
    initialWidth: { type: Number, required: true, default: 400 },
    initialHeight: { type: Number, required: true, default: null },
    canResize: { type: Boolean, required: false, default: false },
    closeButton: { type: Boolean, required: false, default: true },
    mini: { type: Boolean, required: false, default: false },
    front: { type: Boolean, required: false, default: false },
    contextmenu: { type: Array, required: false, default: null },
    buttonsLeft: { type: Array, required: false, default: () => [] },
    buttonsRight: { type: Array, required: false, default: () => [] }
  },
  emits: ["closed"],
  setup(__props: any, { expose: __expose, emit: __emit }) {

const emit = __emit
const props = __props
function dragListen(fn: (ev: PointerEvent) => void) {
	window.addEventListener('pointermove', fn);
	const clear = () => {
		dragClear(fn);
	};
	window.addEventListener('pointerup', clear, { once: true });
	window.addEventListener('pointercancel', clear, { once: true });
	window.addEventListener('blur', clear, { once: true });
}
function dragClear(fn: (ev: PointerEvent) => void) {
	window.removeEventListener('pointermove', fn);
}
function capturePointer(evt: PointerEvent) {
	const target = evt.currentTarget;
	if (!(target instanceof HTMLElement)) return;
	if (!target.setPointerCapture) return;
	try {
		target.setPointerCapture(evt.pointerId);
	} catch {
		return;
	}
	const release = () => {
		if (target.hasPointerCapture(evt.pointerId)) {
			target.releasePointerCapture(evt.pointerId);
		}
	};
	window.addEventListener('pointerup', release, { once: true });
	window.addEventListener('pointercancel', release, { once: true });
}
provide('inWindow', true);
const rootEl = useTemplateRef('rootEl');
const showing = ref(true);
let beforeClickedAt = 0;
const maximized = ref(false);
const minimized = ref(false);
let unResizedTop = '';
let unResizedLeft = '';
let unResizedWidth = '';
let unResizedHeight = '';
function close() {
	showing.value = false;
}
function onKeydown(evt: KeyboardEvent) {
	if (evt.which === 27) { // Esc
		evt.preventDefault();
		evt.stopPropagation();
		close();
	}
}
function onContextmenu(ev: PointerEvent) {
	if (props.contextmenu) {
		os.contextMenu(props.contextmenu, ev);
	}
}
// 最前面へ移動
function top() {
	if (rootEl.value) {
		rootEl.value.style.zIndex = os.claimZIndex(props.front ? 'middle' : 'low').toString();
	}
}
function maximize() {
	if (rootEl.value == null) return;
	maximized.value = true;
	unResizedTop = rootEl.value.style.top;
	unResizedLeft = rootEl.value.style.left;
	unResizedWidth = rootEl.value.style.width;
	unResizedHeight = rootEl.value.style.height;
	rootEl.value.style.top = '0';
	rootEl.value.style.left = '0';
	rootEl.value.style.width = '100%';
	rootEl.value.style.height = '100%';
}
function unMaximize() {
	if (rootEl.value == null) return;
	maximized.value = false;
	rootEl.value.style.top = unResizedTop;
	rootEl.value.style.left = unResizedLeft;
	rootEl.value.style.width = unResizedWidth;
	rootEl.value.style.height = unResizedHeight;
}
function minimize() {
	if (rootEl.value == null) return;
	minimized.value = true;
	unResizedWidth = rootEl.value.style.width;
	unResizedHeight = rootEl.value.style.height;
	rootEl.value.style.width = minWidth + 'px';
	rootEl.value.style.height = props.mini ? '32px' : '39px';
}
function unMinimize() {
	if (rootEl.value == null) return;
	const main = rootEl.value;
	minimized.value = false;
	rootEl.value.style.width = unResizedWidth;
	rootEl.value.style.height = unResizedHeight;
	const browserWidth = window.innerWidth;
	const browserHeight = window.innerHeight;
	const windowWidth = main.offsetWidth;
	const windowHeight = main.offsetHeight;
	const position = main.getBoundingClientRect();
	if (position.top + windowHeight > browserHeight) main.style.top = browserHeight - windowHeight + 'px';
	if (position.left + windowWidth > browserWidth) main.style.left = browserWidth - windowWidth + 'px';
}
function onBodyMousedown() {
	top();
}
function onDblClick() {
	if (minimized.value) {
		unMinimize();
	} else {
		maximize();
	}
}
function getPositionX(event: PointerEvent) {
	return event.clientX;
}
function getPositionY(event: PointerEvent) {
	return event.clientY;
}
function onHeaderPointerdown(evt: PointerEvent) {
	capturePointer(evt);
	// 右クリックはコンテキストメニューを開こうとした可能性が高いため無視
	if ('button' in evt && evt.button === 2) return;
	let beforeMaximized = false;
	if (maximized.value) {
		beforeMaximized = true;
		unMaximize();
	}
	// ダブルクリック判定
	if (Date.now() - beforeClickedAt < 300) {
		beforeClickedAt = Date.now();
		onDblClick();
		return;
	}
	beforeClickedAt = Date.now();
	const main = rootEl.value;
	if (main == null) return;
	if (!elementContains(main, window.document.activeElement)) main.focus();
	const position = main.getBoundingClientRect();
	const clickX = getPositionX(evt);
	const clickY = getPositionY(evt);
	const moveBaseX = beforeMaximized ? parseInt(unResizedWidth, 10) / 2 : clickX - position.left; // TODO: parseIntやめる
	const moveBaseY = beforeMaximized ? 20 : clickY - position.top;
	const browserWidth = window.innerWidth;
	const browserHeight = window.innerHeight;
	const windowWidth = main.offsetWidth;
	const windowHeight = main.offsetHeight;
	function move(x: number, y: number) {
		let moveLeft = x - moveBaseX;
		let moveTop = y - moveBaseY;
		// 下はみ出し
		if (moveTop + windowHeight > browserHeight) moveTop = browserHeight - windowHeight;
		// 左はみ出し
		if (moveLeft < 0) moveLeft = 0;
		// 上はみ出し
		if (moveTop < 0) moveTop = 0;
		// 右はみ出し
		if (moveLeft + windowWidth > browserWidth) moveLeft = browserWidth - windowWidth;
		if (rootEl.value) {
			rootEl.value.style.left = moveLeft + 'px';
			rootEl.value.style.top = moveTop + 'px';
		}
	}
	if (beforeMaximized) {
		move(clickX, clickY);
	}
	// 動かした時
	dragListen(me => {
		const x = getPositionX(me);
		const y = getPositionY(me);
		move(x, y);
	});
}
// 上ハンドル掴み時
function onTopHandlePointerdown(evt: PointerEvent) {
	capturePointer(evt);
	const main = rootEl.value;
	// どういうわけかnullになることがある
	if (main == null) return;
	const base = getPositionY(evt);
	const height = parseInt(getComputedStyle(main, '').height, 10);
	const top = parseInt(getComputedStyle(main, '').top, 10);
	// 動かした時
	dragListen(me => {
		const move = getPositionY(me) - base;
		if (top + move > 0) {
			if (height + -move > minHeight) {
				applyTransformHeight(height + -move);
				applyTransformTop(top + move);
			} else { // 最小の高さより小さくなろうとした時
				applyTransformHeight(minHeight);
				applyTransformTop(top + (height - minHeight));
			}
		} else { // 上のはみ出し時
			applyTransformHeight(top + height);
			applyTransformTop(0);
		}
	});
}
// 右ハンドル掴み時
function onRightHandlePointerdown(evt: PointerEvent) {
	capturePointer(evt);
	const main = rootEl.value;
	if (main == null) return;
	const base = getPositionX(evt);
	const width = parseInt(getComputedStyle(main, '').width, 10);
	const left = parseInt(getComputedStyle(main, '').left, 10);
	const browserWidth = window.innerWidth;
	// 動かした時
	dragListen(me => {
		const move = getPositionX(me) - base;
		if (left + width + move < browserWidth) {
			if (width + move > minWidth) {
				applyTransformWidth(width + move);
			} else { // 最小の幅より小さくなろうとした時
				applyTransformWidth(minWidth);
			}
		} else { // 右のはみ出し時
			applyTransformWidth(browserWidth - left);
		}
	});
}
// 下ハンドル掴み時
function onBottomHandlePointerdown(evt: PointerEvent) {
	capturePointer(evt);
	const main = rootEl.value;
	if (main == null) return;
	const base = getPositionY(evt);
	const height = parseInt(getComputedStyle(main, '').height, 10);
	const top = parseInt(getComputedStyle(main, '').top, 10);
	const browserHeight = window.innerHeight;
	// 動かした時
	dragListen(me => {
		const move = getPositionY(me) - base;
		if (top + height + move < browserHeight) {
			if (height + move > minHeight) {
				applyTransformHeight(height + move);
			} else { // 最小の高さより小さくなろうとした時
				applyTransformHeight(minHeight);
			}
		} else { // 下のはみ出し時
			applyTransformHeight(browserHeight - top);
		}
	});
}
// 左ハンドル掴み時
function onLeftHandlePointerdown(evt: PointerEvent) {
	capturePointer(evt);
	const main = rootEl.value;
	if (main == null) return;
	const base = getPositionX(evt);
	const width = parseInt(getComputedStyle(main, '').width, 10);
	const left = parseInt(getComputedStyle(main, '').left, 10);
	// 動かした時
	dragListen(me => {
		const move = getPositionX(me) - base;
		if (left + move > 0) {
			if (width + -move > minWidth) {
				applyTransformWidth(width + -move);
				applyTransformLeft(left + move);
			} else { // 最小の幅より小さくなろうとした時
				applyTransformWidth(minWidth);
				applyTransformLeft(left + (width - minWidth));
			}
		} else { // 左のはみ出し時
			applyTransformWidth(left + width);
			applyTransformLeft(0);
		}
	});
}
// 左上ハンドル掴み時
function onTopLeftHandlePointerdown(evt: PointerEvent) {
	onTopHandlePointerdown(evt);
	onLeftHandlePointerdown(evt);
}
// 右上ハンドル掴み時
function onTopRightHandlePointerdown(evt: PointerEvent) {
	onTopHandlePointerdown(evt);
	onRightHandlePointerdown(evt);
}
// 右下ハンドル掴み時
function onBottomRightHandlePointerdown(evt: PointerEvent) {
	onBottomHandlePointerdown(evt);
	onRightHandlePointerdown(evt);
}
// 左下ハンドル掴み時
function onBottomLeftHandlePointerdown(evt: PointerEvent) {
	onBottomHandlePointerdown(evt);
	onLeftHandlePointerdown(evt);
}
// 高さを適用
function applyTransformHeight(height: number) {
	if (height > window.innerHeight) height = window.innerHeight;
	if (rootEl.value) rootEl.value.style.height = height + 'px';
}
// 幅を適用
function applyTransformWidth(width: number) {
	if (width > window.innerWidth) width = window.innerWidth;
	if (rootEl.value) rootEl.value.style.width = width + 'px';
}
// Y座標を適用
function applyTransformTop(top: number) {
	if (rootEl.value) rootEl.value.style.top = top + 'px';
}
// X座標を適用
function applyTransformLeft(left: number) {
	if (rootEl.value) rootEl.value.style.left = left + 'px';
}
function onBrowserResize() {
	const main = rootEl.value;
	if (main == null) return;
	const position = main.getBoundingClientRect();
	const browserWidth = window.innerWidth;
	const browserHeight = window.innerHeight;
	const windowWidth = main.offsetWidth;
	const windowHeight = main.offsetHeight;
	if (position.left < 0) main.style.left = '0'; // 左はみ出し
	if (position.top + windowHeight > browserHeight) main.style.top = browserHeight - windowHeight + 'px'; // 下はみ出し
	if (position.left + windowWidth > browserWidth) main.style.left = browserWidth - windowWidth + 'px'; // 右はみ出し
	if (position.top < 0) main.style.top = '0'; // 上はみ出し
}
onMounted(() => {
	applyTransformWidth(props.initialWidth);
	if (props.initialHeight) applyTransformHeight(props.initialHeight);
	if (rootEl.value) {
		applyTransformTop((window.innerHeight / 2) - (rootEl.value.offsetHeight / 2));
		applyTransformLeft((window.innerWidth / 2) - (rootEl.value.offsetWidth / 2));
	}
	// 他のウィンドウ内のボタンなどを押してこのウィンドウが開かれた場合、親が最前面になろうとするのでそれに隠されないようにする
	top();
	window.addEventListener('resize', onBrowserResize);
});
onBeforeUnmount(() => {
	window.removeEventListener('resize', onBrowserResize);
});
__expose({
	close,
})

return (_ctx: any,_cache: any) => {
  const _directive_tooltip = _resolveDirective("tooltip")

  return (_openBlock(), _createBlock(_Transition, {
      enterActiveClass: _unref(prefer).s.animation ? _ctx.$style.transition_window_enterActive : '',
      leaveActiveClass: _unref(prefer).s.animation ? _ctx.$style.transition_window_leaveActive : '',
      enterFromClass: _unref(prefer).s.animation ? _ctx.$style.transition_window_enterFrom : '',
      leaveToClass: _unref(prefer).s.animation ? _ctx.$style.transition_window_leaveTo : '',
      appear: "",
      onAfterLeave: _cache[0] || (_cache[0] = ($event: any) => (emit('closed')))
    }, {
      default: _withCtx(() => [
        (showing.value)
          ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            ref: "rootEl",
            class: _normalizeClass([_ctx.$style.root, { [_ctx.$style.maximized]: maximized.value }])
          }, [
            _createElementVNode("div", {
              class: _normalizeClass(["_shadow", _ctx.$style.body]),
              onMousedown: onBodyMousedown,
              onKeydown: onKeydown
            }, [
              _createElementVNode("div", {
                class: _normalizeClass([_ctx.$style.header, { [_ctx.$style.mini]: __props.mini }]),
                onContextmenu: _withModifiers(onContextmenu, ["prevent","stop"])
              }, [
                _createElementVNode("span", {
                  class: _normalizeClass(_ctx.$style.headerLeft)
                }, [
                  (!minimized.value)
                    ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
                      (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(__props.buttonsLeft, (button) => {
                        return (_openBlock(), _createElementBlock("button", { class: _normalizeClass(["_button", [_ctx.$style.headerButton, { [_ctx.$style.highlighted]: button.highlighted }]]), onClick: _cache[1] || (_cache[1] = ($event: any) => (button.onClick)) }, [
                          _createElementVNode("i", {
                            class: _normalizeClass(button.icon)
                          }, null, 2 /* CLASS */)
                        ], 2 /* CLASS */))
                      }), 256 /* UNKEYED_FRAGMENT */))
                    ], 64 /* STABLE_FRAGMENT */))
                    : _createCommentVNode("v-if", true)
                ]),
                _createElementVNode("span", {
                  class: _normalizeClass(_ctx.$style.headerTitle),
                  onPointerdown: _withModifiers(onHeaderPointerdown, ["prevent"])
                }, [
                  _renderSlot(_ctx.$slots, "header")
                ], 32 /* NEED_HYDRATION */),
                _createElementVNode("span", {
                  class: _normalizeClass(_ctx.$style.headerRight)
                }, [
                  (!minimized.value)
                    ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
                      (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(__props.buttonsRight, (button) => {
                        return (_openBlock(), _createElementBlock("button", { class: _normalizeClass(["_button", [_ctx.$style.headerButton, { [_ctx.$style.highlighted]: button.highlighted }]]), onClick: _cache[2] || (_cache[2] = ($event: any) => (button.onClick)) }, [
                          _createElementVNode("i", {
                            class: _normalizeClass(button.icon)
                          }, null, 2 /* CLASS */)
                        ], 2 /* CLASS */))
                      }), 256 /* UNKEYED_FRAGMENT */))
                    ], 64 /* STABLE_FRAGMENT */))
                    : _createCommentVNode("v-if", true),
                  (__props.canResize && minimized.value)
                    ? (_openBlock(), _createElementBlock("button", {
                      key: 0,
                      class: _normalizeClass(["_button", _ctx.$style.headerButton]),
                      onClick: _cache[3] || (_cache[3] = ($event: any) => (unMinimize()))
                    }, [
                      _hoisted_1
                    ]))
                    : (__props.canResize && !maximized.value)
                      ? (_openBlock(), _createElementBlock("button", {
                        key: 1,
                        class: _normalizeClass(["_button", _ctx.$style.headerButton]),
                        onClick: _cache[4] || (_cache[4] = ($event: any) => (minimize()))
                      }, [
                        _hoisted_2
                      ]))
                    : _createCommentVNode("v-if", true),
                  (__props.canResize && maximized.value)
                    ? (_openBlock(), _createElementBlock("button", {
                      key: 0,
                      class: _normalizeClass(["_button", _ctx.$style.headerButton]),
                      onClick: _cache[5] || (_cache[5] = ($event: any) => (unMaximize()))
                    }, [
                      _hoisted_3
                    ]))
                    : (__props.canResize && !maximized.value && !minimized.value)
                      ? (_openBlock(), _createElementBlock("button", {
                        key: 1,
                        class: _normalizeClass(["_button", _ctx.$style.headerButton]),
                        onClick: _cache[6] || (_cache[6] = ($event: any) => (maximize()))
                      }, [
                        _hoisted_4
                      ]))
                    : _createCommentVNode("v-if", true),
                  (__props.closeButton)
                    ? (_openBlock(), _createElementBlock("button", {
                      key: 0,
                      class: _normalizeClass(["_button", _ctx.$style.headerButton]),
                      onClick: _cache[7] || (_cache[7] = ($event: any) => (close()))
                    }, [
                      _hoisted_5
                    ]))
                    : _createCommentVNode("v-if", true)
                ])
              ], 34 /* CLASS, NEED_HYDRATION */),
              _createElementVNode("div", {
                class: _normalizeClass(_ctx.$style.content)
              }, [
                _renderSlot(_ctx.$slots, "default")
              ])
            ], 32 /* NEED_HYDRATION */),
            (__props.canResize && !minimized.value)
              ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
                _createElementVNode("div", {
                  class: _normalizeClass(_ctx.$style.handleTop),
                  onPointerdown: _withModifiers(onTopHandlePointerdown, ["prevent"])
                }, null, 32 /* NEED_HYDRATION */),
                _createElementVNode("div", {
                  class: _normalizeClass(_ctx.$style.handleRight),
                  onPointerdown: _withModifiers(onRightHandlePointerdown, ["prevent"])
                }, null, 32 /* NEED_HYDRATION */),
                _createElementVNode("div", {
                  class: _normalizeClass(_ctx.$style.handleBottom),
                  onPointerdown: _withModifiers(onBottomHandlePointerdown, ["prevent"])
                }, null, 32 /* NEED_HYDRATION */),
                _createElementVNode("div", {
                  class: _normalizeClass(_ctx.$style.handleLeft),
                  onPointerdown: _withModifiers(onLeftHandlePointerdown, ["prevent"])
                }, null, 32 /* NEED_HYDRATION */),
                _createElementVNode("div", {
                  class: _normalizeClass(_ctx.$style.handleTopLeft),
                  onPointerdown: _withModifiers(onTopLeftHandlePointerdown, ["prevent"])
                }, null, 32 /* NEED_HYDRATION */),
                _createElementVNode("div", {
                  class: _normalizeClass(_ctx.$style.handleTopRight),
                  onPointerdown: _withModifiers(onTopRightHandlePointerdown, ["prevent"])
                }, null, 32 /* NEED_HYDRATION */),
                _createElementVNode("div", {
                  class: _normalizeClass(_ctx.$style.handleBottomRight),
                  onPointerdown: _withModifiers(onBottomRightHandlePointerdown, ["prevent"])
                }, null, 32 /* NEED_HYDRATION */),
                _createElementVNode("div", {
                  class: _normalizeClass(_ctx.$style.handleBottomLeft),
                  onPointerdown: _withModifiers(onBottomLeftHandlePointerdown, ["prevent"])
                }, null, 32 /* NEED_HYDRATION */)
              ], 64 /* STABLE_FRAGMENT */))
              : _createCommentVNode("v-if", true)
          ]))
          : _createCommentVNode("v-if", true)
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["enterActiveClass", "leaveActiveClass", "enterFromClass", "leaveToClass"]))
}
}

})
