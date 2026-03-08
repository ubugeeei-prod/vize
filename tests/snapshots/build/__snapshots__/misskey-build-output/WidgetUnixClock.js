import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, unref as _unref } from "vue"

import { onUnmounted, ref, watch } from 'vue'
import { useWidgetPropsManager } from './widget.js'
import { i18n } from '@/i18n.js'
import type { WidgetComponentEmits, WidgetComponentExpose, WidgetComponentProps } from './widget.js'
import type { FormWithDefault, GetFormResultType } from '@/utility/form.js'

type WidgetProps = GetFormResultType<typeof widgetPropsDef>;
const name = 'unixClock';

export default /*@__PURE__*/_defineComponent({
  __name: 'WidgetUnixClock',
  setup(__props: any, { expose: __expose, emit: __emit }) {

const emit = __emit
const props = __props
const widgetPropsDef = {
	transparent: {
		type: 'boolean',
		label: i18n.ts._widgetOptions.transparent,
		default: false,
	},
	fontSize: {
		type: 'number',
		label: i18n.ts.fontSize,
		default: 1.5,
		step: 0.1,
	},
	showMs: {
		type: 'boolean',
		label: i18n.ts._widgetOptions._clock.showMs,
		default: true,
	},
	showLabel: {
		type: 'boolean',
		label: i18n.ts._widgetOptions._clock.showLabel,
		default: true,
	},
} satisfies FormWithDefault;
const { widgetProps, configure } = useWidgetPropsManager(name,
	widgetPropsDef,
	props,
	emit,
);
let intervalId: number | null = null;
const ss = ref('');
const ms = ref('');
const showColon = ref(false);
let prevSec: string | null = null;
watch(showColon, (v) => {
	if (v) {
		window.setTimeout(() => {
			showColon.value = false;
		}, 30);
	}
});
const tick = () => {
	const now = Date.now();
	ss.value = Math.floor(now / 1000).toString();
	ms.value = Math.floor(now % 1000 / 10).toString().padStart(2, '0');
	if (ss.value !== prevSec) showColon.value = true;
	prevSec = ss.value;
};
tick();
watch(() => widgetProps.showMs, () => {
	if (intervalId) window.clearInterval(intervalId);
	intervalId = window.setInterval(tick, widgetProps.showMs ? 10 : 1000);
}, { immediate: true });
onUnmounted(() => {
	if (intervalId) {
		window.clearInterval(intervalId);
		intervalId = null;
	}
});
__expose({
	name,
	configure,
	id: props.widget ? props.widget.id : null,
})

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(["mkw-unixClock _monospace", { _panel: !_unref(widgetProps).transparent }]),
      style: _normalizeStyle({ fontSize: `${_unref(widgetProps).fontSize}em` })
    }, [ (_unref(widgetProps).showLabel) ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          class: "label"
        }, "UNIX Epoch")) : _createCommentVNode("v-if", true), _createElementVNode("div", { class: "time" }, [ _createElementVNode("span", {
          textContent: _toDisplayString(ss.value)
        }, null, 8 /* PROPS */, ["textContent"]), (_unref(widgetProps).showMs) ? (_openBlock(), _createElementBlock("span", {
            key: 0,
            class: _normalizeClass(["colon", { showColon: showColon.value }])
          }, ":")) : _createCommentVNode("v-if", true), (_unref(widgetProps).showMs) ? (_openBlock(), _createElementBlock("span", {
            key: 0,
            textContent: _toDisplayString(ms.value)
          })) : _createCommentVNode("v-if", true) ]), (_unref(widgetProps).showLabel) ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          class: "label"
        }, "UTC")) : _createCommentVNode("v-if", true) ], 6 /* CLASS, STYLE */))
}
}

})
