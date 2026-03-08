import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, unref as _unref } from "vue"

import { computed } from 'vue'
import { useWidgetPropsManager } from './widget.js'
import type { WidgetComponentEmits, WidgetComponentExpose, WidgetComponentProps } from './widget.js'
import type { FormWithDefault, GetFormResultType } from '@/utility/form.js'
import { timezones } from '@/utility/timezones.js'
import { i18n } from '@/i18n.js'
import MkDigitalClock from '@/components/MkDigitalClock.vue'

type WidgetProps = GetFormResultType<typeof widgetPropsDef>;
const name = 'digitalClock';

export default /*@__PURE__*/_defineComponent({
  __name: 'WidgetDigitalClock',
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
	timezone: {
		type: 'enum',
		label: i18n.ts._widgetOptions._clock.timezone,
		default: null,
		enum: [...timezones.map((tz) => ({
			label: tz.name,
			value: tz.name.toLowerCase(),
		})), {
			label: i18n.ts.auto,
			value: null,
		}],
	},
} satisfies FormWithDefault;
const { widgetProps, configure } = useWidgetPropsManager(name,
	widgetPropsDef,
	props,
	emit,
);
const tzAbbrev = computed(() => (widgetProps.timezone === null
	? timezones.find((tz) => tz.name.toLowerCase() === Intl.DateTimeFormat().resolvedOptions().timeZone.toLowerCase())?.abbrev
	: timezones.find((tz) => tz.name.toLowerCase() === widgetProps.timezone)?.abbrev) ?? '?');
const tzOffset = computed(() => widgetProps.timezone === null
	? 0 - new Date().getTimezoneOffset()
	: timezones.find((tz) => tz.name.toLowerCase() === widgetProps.timezone)?.offset ?? 0);
const tzOffsetLabel = computed(() => (tzOffset.value >= 0 ? '+' : '-') + Math.floor(tzOffset.value / 60).toString().padStart(2, '0') + ':' + (tzOffset.value % 60).toString().padStart(2, '0'));
__expose({
	name,
	configure,
	id: props.widget ? props.widget.id : null,
})

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      "data-cy-mkw-digitalClock": "",
      class: _normalizeClass(["_monospace", [_ctx.$style.root, { _panel: !_unref(widgetProps).transparent }]]),
      style: _normalizeStyle({ fontSize: `${_unref(widgetProps).fontSize}em` })
    }, [ (_unref(widgetProps).showLabel) ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          class: _normalizeClass(_ctx.$style.label)
        }, _toDisplayString(tzAbbrev.value), 1 /* TEXT */)) : _createCommentVNode("v-if", true), _createElementVNode("div", null, [ _createVNode(MkDigitalClock, {
          showMs: _unref(widgetProps).showMs,
          offset: tzOffset.value
        }, null, 8 /* PROPS */, ["showMs", "offset"]) ]), (_unref(widgetProps).showLabel) ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          class: _normalizeClass(_ctx.$style.label)
        }, _toDisplayString(tzOffsetLabel.value), 1 /* TEXT */)) : _createCommentVNode("v-if", true) ], 6 /* CLASS, STYLE */))
}
}

})
