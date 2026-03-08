import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass } from "vue"

import { onMounted, onUnmounted, ref, watch } from 'vue'
import { defaultIdlingRenderScheduler } from '@/utility/idle-render.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkDigitalClock',
  props: {
    showS: { type: Boolean, required: false, default: true },
    showMs: { type: Boolean, required: false, default: false },
    offset: { type: Number, required: false, default: 0 - new Date().getTimezoneOffset() },
    now: { type: Function, required: false, default: () => new Date() }
  },
  setup(__props: any) {

const props = __props
const hh = ref('');
const mm = ref('');
const ss = ref('');
const ms = ref('');
const showColon = ref(false);
let prevSec: number | null = null;
watch(showColon, (v) => {
	if (v) {
		window.setTimeout(() => {
			showColon.value = false;
		}, 30);
	}
});
const tick = (): void => {
	const now = props.now();
	now.setMinutes(now.getMinutes() + now.getTimezoneOffset() + props.offset);
	hh.value = now.getHours().toString().padStart(2, '0');
	mm.value = now.getMinutes().toString().padStart(2, '0');
	ss.value = now.getSeconds().toString().padStart(2, '0');
	ms.value = Math.floor(now.getMilliseconds() / 10).toString().padStart(2, '0');
	if (now.getSeconds() !== prevSec) showColon.value = true;
	prevSec = now.getSeconds();
};
tick();
onMounted(() => {
	defaultIdlingRenderScheduler.add(tick);
});
onUnmounted(() => {
	defaultIdlingRenderScheduler.delete(tick);
});

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("span", null, [ _createElementVNode("span", {
        textContent: _toDisplayString(hh.value)
      }, null, 8 /* PROPS */, ["textContent"]), _createElementVNode("span", {
        class: _normalizeClass([_ctx.$style.colon, { [_ctx.$style.showColon]: showColon.value }])
      }, ":", 2 /* CLASS */), _createElementVNode("span", {
        textContent: _toDisplayString(mm.value)
      }, null, 8 /* PROPS */, ["textContent"]), (__props.showS) ? (_openBlock(), _createElementBlock("span", {
          key: 0,
          class: _normalizeClass([_ctx.$style.colon, { [_ctx.$style.showColon]: showColon.value }])
        }, ":")) : _createCommentVNode("v-if", true), (__props.showS) ? (_openBlock(), _createElementBlock("span", {
          key: 0,
          textContent: _toDisplayString(ss.value)
        })) : _createCommentVNode("v-if", true), (__props.showMs) ? (_openBlock(), _createElementBlock("span", {
          key: 0,
          class: _normalizeClass([_ctx.$style.colon, { [_ctx.$style.showColon]: showColon.value }])
        }, ":")) : _createCommentVNode("v-if", true), (__props.showMs) ? (_openBlock(), _createElementBlock("span", {
          key: 0,
          textContent: _toDisplayString(ms.value)
        })) : _createCommentVNode("v-if", true) ]))
}
}

})
