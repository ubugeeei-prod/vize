import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, unref as _unref } from "vue"

import { onMounted, ref } from 'vue'
import * as os from '@/os.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkPlusOneEffect',
  props: {
    x: { type: Number, required: true },
    y: { type: Number, required: true },
    value: { type: [Number, String], required: false, default: 1 }
  },
  emits: ["end"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const up = ref(false);
const zIndex = os.claimZIndex('middle');
const angle = (45 - (Math.random() * 90)) + 'deg';
onMounted(() => {
	window.setTimeout(() => {
		up.value = true;
	}, 10);
	window.setTimeout(() => {
		emit('end');
	}, 1100);
});

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(_ctx.$style.root),
      style: _normalizeStyle({ zIndex: _unref(zIndex), top: `${__props.y - 64}px`, left: `${__props.x - 64}px` })
    }, [ _createElementVNode("span", {
        class: _normalizeClass(["text", { up: up.value }])
      }, "+" + _toDisplayString(__props.value), 3 /* TEXT, CLASS */) ], 4 /* STYLE */))
}
}

})
