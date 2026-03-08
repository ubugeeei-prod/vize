import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"

import { computed } from 'vue'
import { useWidgetPropsManager } from './widget.js'
import type { WidgetComponentEmits, WidgetComponentExpose, WidgetComponentProps } from './widget.js'
import type { FormWithDefault, GetFormResultType } from '@/utility/form.js'
import MkContainer from '@/components/MkContainer.vue'
import MkAnalogClock from '@/components/MkAnalogClock.vue'
import MkDigitalClock from '@/components/MkDigitalClock.vue'
import { timezones } from '@/utility/timezones.js'
import { i18n } from '@/i18n.js'

type WidgetProps = GetFormResultType<typeof widgetPropsDef>;
const name = 'clock';

export default /*@__PURE__*/_defineComponent({
  __name: 'WidgetClock',
  setup(__props: any, { expose: __expose, emit: __emit }) {

const emit = __emit
const props = __props
const widgetPropsDef = {
	transparent: {
		type: 'boolean',
		label: i18n.ts._widgetOptions.transparent,
		default: false,
	},
	size: {
		type: 'radio',
		label: i18n.ts._widgetOptions._clock.size,
		default: 'medium',
		options: [{
			value: 'small' as const,
			label: i18n.ts.small,
		}, {
			value: 'medium' as const,
			label: i18n.ts.medium,
		}, {
			value: 'large' as const,
			label: i18n.ts.large,
		}],
	},
	thickness: {
		type: 'radio',
		label: i18n.ts._widgetOptions._clock.thickness,
		default: 0.2,
		options: [{
			value: 0.1 as const,
			label: i18n.ts._widgetOptions._clock.thicknessThin,
		}, {
			value: 0.2 as const,
			label: i18n.ts._widgetOptions._clock.thicknessMedium,
		}, {
			value: 0.3 as const,
			label: i18n.ts._widgetOptions._clock.thicknessThick,
		}],
	},
	graduations: {
		type: 'radio',
		label: i18n.ts._widgetOptions._clock.graduations,
		default: 'numbers',
		options: [{
			value: 'none' as const,
			label: i18n.ts.none,
		}, {
			value: 'dots' as const,
			label: i18n.ts._widgetOptions._clock.graduationDots,
		}, {
			value: 'numbers' as const,
			label: i18n.ts._widgetOptions._clock.graduationArabic,
		}, /*, {
			value: 'roman' as const,
			label: i18n.ts._widgetOptions._clock.graduationRoman,
		}*/],
	},
	fadeGraduations: {
		type: 'boolean',
		label: i18n.ts._widgetOptions._clock.fadeGraduations,
		default: true,
	},
	sAnimation: {
		type: 'radio',
		label: i18n.ts._widgetOptions._clock.sAnimation,
		default: 'elastic',
		options: [{
			value: 'none' as const,
			label: i18n.ts.none,
		}, {
			value: 'elastic' as const,
			label: i18n.ts._widgetOptions._clock.sAnimationElastic,
		}, {
			value: 'easeOut' as const,
			label: i18n.ts._widgetOptions._clock.sAnimationEaseOut,
		}],
	},
	twentyFour: {
		type: 'boolean',
		label: i18n.ts._widgetOptions._clock.twentyFour,
		default: false,
	},
	label: {
		type: 'radio',
		label: i18n.ts.label,
		default: 'none',
		options: [{
			value: 'none' as const,
			label: i18n.ts.none,
		}, {
			value: 'time' as const,
			label: i18n.ts._widgetOptions._clock.labelTime,
		}, {
			value: 'tz' as const,
			label: i18n.ts._widgetOptions._clock.labelTz,
		}, {
			value: 'timeAndTz' as const,
			label: i18n.ts._widgetOptions._clock.labelTimeAndTz,
		}],
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
  return (_openBlock(), _createBlock(MkContainer, {
      naked: _unref(widgetProps).transparent,
      showHeader: false,
      "data-cy-mkw-clock": ""
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: _normalizeClass([_ctx.$style.root, {
  			[_ctx.$style.small]: _unref(widgetProps).size === 'small',
  			[_ctx.$style.medium]: _unref(widgetProps).size === 'medium',
  			[_ctx.$style.large]: _unref(widgetProps).size === 'large',
  		}])
        }, [
          (_unref(widgetProps).label === 'tz' || _unref(widgetProps).label === 'timeAndTz')
            ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: _normalizeClass(["_monospace", [_ctx.$style.label, _ctx.$style.a]])
            }, _toDisplayString(tzAbbrev.value), 1 /* TEXT */))
            : _createCommentVNode("v-if", true),
          _createVNode(MkAnalogClock, {
            class: _normalizeClass(_ctx.$style.clock),
            thickness: _unref(widgetProps).thickness,
            offset: tzOffset.value,
            graduations: _unref(widgetProps).graduations,
            fadeGraduations: _unref(widgetProps).fadeGraduations,
            twentyfour: _unref(widgetProps).twentyFour,
            sAnimation: _unref(widgetProps).sAnimation
          }, null, 8 /* PROPS */, ["thickness", "offset", "graduations", "fadeGraduations", "twentyfour", "sAnimation"]),
          (_unref(widgetProps).label === 'time' || _unref(widgetProps).label === 'timeAndTz')
            ? (_openBlock(), _createBlock(MkDigitalClock, {
              key: 0,
              class: _normalizeClass(["_monospace", [_ctx.$style.label, _ctx.$style.c]]),
              showS: false,
              offset: tzOffset.value
            }, null, 8 /* PROPS */, ["showS", "offset"]))
            : _createCommentVNode("v-if", true),
          (_unref(widgetProps).label === 'tz' || _unref(widgetProps).label === 'timeAndTz')
            ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: _normalizeClass(["_monospace", [_ctx.$style.label, _ctx.$style.d]])
            }, _toDisplayString(tzOffsetLabel.value), 1 /* TEXT */))
            : _createCommentVNode("v-if", true)
        ], 2 /* CLASS */)
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["naked", "showHeader"]))
}
}

})
