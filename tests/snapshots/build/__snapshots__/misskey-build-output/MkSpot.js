import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-left" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-right" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-check" })
import { nextTick, onMounted, onUnmounted, ref, useTemplateRef } from 'vue'
import { calcPopupPosition } from '@/utility/popup-position.js'
import * as os from '@/os.js'
import MkButton from '@/components/MkButton.vue'
import { i18n } from '@/i18n.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkSpot',
  props: {
    title: { type: String, required: true },
    description: { type: String, required: true },
    anchorElement: { type: null, required: false },
    x: { type: Number, required: false },
    y: { type: Number, required: false },
    direction: { type: String, required: false, default: 'top' },
    hasPrev: { type: Boolean, required: true },
    hasNext: { type: Boolean, required: true }
  },
  emits: ["prev", "next"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
function prev() {
	emit('prev');
}
function next() {
	emit('next');
}
const rootEl = useTemplateRef('rootEl');
const bodyEl = useTemplateRef('bodyEl');
const spotEl = useTemplateRef('spotEl');
const zIndex = os.claimZIndex('high');
const spotX = ref(0);
const spotY = ref(0);
const spotWidth = ref(0);
const spotHeight = ref(0);
function setPosition() {
	if (spotEl.value == null) return;
	if (bodyEl.value == null) return;
	if (props.anchorElement == null) return;
	const rect = props.anchorElement.getBoundingClientRect();
	spotX.value = rect.left;
	spotY.value = rect.top;
	spotWidth.value = rect.width;
	spotHeight.value = rect.height;
	const data = calcPopupPosition(bodyEl.value, {
		anchorElement: props.anchorElement,
		direction: props.direction,
		align: 'center',
		innerMargin: 16,
		x: props.x,
		y: props.y,
	});
	bodyEl.value.style.transformOrigin = data.transformOrigin;
	bodyEl.value.style.left = data.left + 'px';
	bodyEl.value.style.top = data.top + 'px';
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
  return (_openBlock(), _createElementBlock("div", {
      ref_key: "rootEl", ref: rootEl,
      class: _normalizeClass(_ctx.$style.root),
      style: _normalizeStyle({ zIndex: _unref(zIndex) })
    }, [ _createElementVNode("div", {
        class: _normalizeClass([_ctx.$style.bg])
      }), _createElementVNode("div", {
        ref_key: "spotEl", ref: spotEl,
        class: _normalizeClass(_ctx.$style.spot)
      }, null, 512 /* NEED_PATCH */), _createElementVNode("div", {
        ref_key: "bodyEl", ref: bodyEl,
        class: _normalizeClass(["_panel _shadow", _ctx.$style.body])
      }, [ _createElementVNode("div", { class: "_gaps_s" }, [ _createElementVNode("div", null, [ _createElementVNode("b", null, _toDisplayString(__props.title), 1 /* TEXT */) ]), _createElementVNode("div", null, _toDisplayString(__props.description), 1 /* TEXT */), _createElementVNode("div", { class: "_buttons" }, [ (__props.hasPrev) ? (_openBlock(), _createBlock(MkButton, {
                key: 0,
                small: "",
                onClick: prev
              }, {
                default: _withCtx(() => [
                  _hoisted_1,
                  _createTextVNode(" "),
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.goBack), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              })) : _createCommentVNode("v-if", true), (__props.hasNext) ? (_openBlock(), _createBlock(MkButton, {
                key: 0,
                small: "",
                primary: "",
                onClick: next
              }, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.next), 1 /* TEXT */),
                  _createTextVNode(" "),
                  _hoisted_2
                ]),
                _: 1 /* STABLE */
              })) : (_openBlock(), _createBlock(MkButton, {
                key: 1,
                small: "",
                primary: "",
                onClick: next
              }, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.done), 1 /* TEXT */),
                  _createTextVNode(" "),
                  _hoisted_3
                ]),
                _: 1 /* STABLE */
              })) ]) ]) ], 512 /* NEED_PATCH */) ], 4 /* STYLE */))
}
}

})
