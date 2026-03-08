import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, Transition as _Transition, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, withDirectives as _withDirectives, renderSlot as _renderSlot, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref, vShow as _vShow } from "vue"

import { nextTick, onMounted, onUnmounted, useTemplateRef } from 'vue'
import * as os from '@/os.js'
import { calcPopupPosition } from '@/utility/popup-position.js'
import { prefer } from '@/preferences.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkTooltip',
  props: {
    showing: { type: Boolean, required: true },
    anchorElement: { type: null, required: false },
    x: { type: Number, required: false },
    y: { type: Number, required: false },
    text: { type: String, required: false },
    asMfm: { type: Boolean, required: false },
    maxWidth: { type: Number, required: false, default: 250 },
    direction: { type: String, required: false, default: 'top' },
    innerMargin: { type: Number, required: false, default: 0 }
  },
  emits: ["closed"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
// タイミングによっては最初から showing = false な場合があり、その場合に closed 扱いにしないと永久にDOMに残ることになる
if (!props.showing) emit('closed');
const el = useTemplateRef('el');
const zIndex = os.claimZIndex('high');
function setPosition() {
	if (el.value == null) return;
	const data = calcPopupPosition(el.value, {
		anchorElement: props.anchorElement,
		direction: props.direction,
		align: 'center',
		innerMargin: props.innerMargin,
		x: props.x,
		y: props.y,
	});
	el.value.style.transformOrigin = data.transformOrigin;
	el.value.style.left = data.left + 'px';
	el.value.style.top = data.top + 'px';
}
let loopHandler: number | null = null;
onMounted(() => {
	nextTick(() => {
		setPosition();
		const loop = () => {
			setPosition();
			loopHandler = window.requestAnimationFrame(loop);
		};
		loop();
	});
});
onUnmounted(() => {
	if (loopHandler != null) window.cancelAnimationFrame(loopHandler);
});

return (_ctx: any,_cache: any) => {
  const _component_Mfm = _resolveComponent("Mfm")

  return (_openBlock(), _createBlock(_Transition, {
      enterActiveClass: _unref(prefer).s.animation ? _ctx.$style.transition_tooltip_enterActive : '',
      leaveActiveClass: _unref(prefer).s.animation ? _ctx.$style.transition_tooltip_leaveActive : '',
      enterFromClass: _unref(prefer).s.animation ? _ctx.$style.transition_tooltip_enterFrom : '',
      leaveToClass: _unref(prefer).s.animation ? _ctx.$style.transition_tooltip_leaveTo : '',
      appear: "",
      css: _unref(prefer).s.animation,
      onAfterLeave: _cache[0] || (_cache[0] = ($event: any) => (emit('closed')))
    }, {
      default: _withCtx(() => [
        _withDirectives(_createElementVNode("div", {
          ref_key: "el", ref: el,
          class: _normalizeClass(["_acrylic _shadow", _ctx.$style.root]),
          style: _normalizeStyle({ zIndex: _unref(zIndex), maxWidth: __props.maxWidth + 'px' })
        }, [
          _renderSlot(_ctx.$slots, "default", {}, () => [
            (__props.text)
              ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
                (__props.asMfm)
                  ? (_openBlock(), _createBlock(_component_Mfm, {
                    key: 0,
                    text: __props.text
                  }, null, 8 /* PROPS */, ["text"]))
                  : (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(__props.text), 1 /* TEXT */))
              ], 64 /* STABLE_FRAGMENT */))
              : _createCommentVNode("v-if", true)
          ])
        ], 4 /* STYLE */), [
          [_vShow, __props.showing]
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["enterActiveClass", "leaveActiveClass", "enterFromClass", "leaveToClass", "css"]))
}
}

})
