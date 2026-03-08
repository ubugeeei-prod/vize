import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, toDisplayString as _toDisplayString } from "vue"

import { reactive, watch } from 'vue'
import number from '@/filters/number.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkNumber',
  props: {
    value: { type: Number, required: true }
  },
  setup(__props: any) {

const props = __props
const tweened = reactive({
	number: 0,
});
watch(() => props.value, (to, from) => {
	// requestAnimationFrameを利用して、500msでfromからtoまでを1次関数的に変化させる
	let start: number | null = null;
	function step(timestamp: number) {
		if (start === null) {
			start = timestamp;
		}
		const elapsed = timestamp - start;
		tweened.number = (from ?? 0) + (to - (from ?? 0)) * elapsed / 500;
		if (elapsed < 500) {
			window.requestAnimationFrame(step);
		} else {
			tweened.number = to;
		}
	}
	window.requestAnimationFrame(step);
}, {
	immediate: true,
});

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("span", null, _toDisplayString(number(Math.floor(tweened.number))), 1 /* TEXT */))
}
}

})
