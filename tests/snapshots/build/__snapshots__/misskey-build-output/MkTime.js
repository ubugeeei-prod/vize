import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, unref as _unref } from "vue"

import isChromatic from 'chromatic/isChromatic'
import { computed } from 'vue'
import { i18n } from '@/i18n.js'
import { dateTimeFormat } from '@@/js/intl-const.js'
import { useLowresTime } from '@/composables/use-lowres-time.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkTime',
  props: {
    time: { type: [Date, String, Number], required: true },
    origin: { type: Date, required: false, default: isChromatic() ? () => new Date('2023-04-01T00:00:00Z') : null },
    mode: { type: String, required: false, default: 'relative' },
    colored: { type: Boolean, required: false }
  },
  setup(__props: any) {

const props = __props
function getDateSafe(n: Date | string | number) {
	try {
		if (n instanceof Date) {
			return n;
		}
		return new Date(n);
	} catch (err) {
		return {
			getTime: () => NaN,
		};
	}
}
// eslint-disable-next-line vue/no-setup-props-reactivity-loss
const _time = props.time == null ? NaN : getDateSafe(props.time).getTime();
const invalid = Number.isNaN(_time);
const absolute = !invalid ? dateTimeFormat.format(_time) : i18n.ts._ago.invalid;
const actualNow = useLowresTime();
const now = computed(() => (props.origin ? props.origin.getTime() : actualNow.value));
// eslint-disable-next-line vue/no-setup-props-reactivity-loss
const ago = computed(() => (now.value - _time) / 1000/*ms*/);
const relative = computed<string>(() => {
	if (props.mode === 'absolute') return ''; // absoluteではrelativeを使わないので計算しない
	if (invalid) return i18n.ts._ago.invalid;

	return (
		ago.value >= 31536000 ? i18n.tsx._ago.yearsAgo({ n: Math.round(ago.value / 31536000).toString() }) :
		ago.value >= 2592000 ? i18n.tsx._ago.monthsAgo({ n: Math.round(ago.value / 2592000).toString() }) :
		ago.value >= 604800 ? i18n.tsx._ago.weeksAgo({ n: Math.round(ago.value / 604800).toString() }) :
		ago.value >= 86400 ? i18n.tsx._ago.daysAgo({ n: Math.round(ago.value / 86400).toString() }) :
		ago.value >= 3600 ? i18n.tsx._ago.hoursAgo({ n: Math.round(ago.value / 3600).toString() }) :
		ago.value >= 60 ? i18n.tsx._ago.minutesAgo({ n: (~~(ago.value / 60)).toString() }) :
		ago.value >= 10 ? i18n.tsx._ago.secondsAgo({ n: (~~(ago.value % 60)).toString() }) :
		ago.value >= -3 ? i18n.ts._ago.justNow :
		ago.value < -31536000 ? i18n.tsx._timeIn.years({ n: Math.round(-ago.value / 31536000).toString() }) :
		ago.value < -2592000 ? i18n.tsx._timeIn.months({ n: Math.round(-ago.value / 2592000).toString() }) :
		ago.value < -604800 ? i18n.tsx._timeIn.weeks({ n: Math.round(-ago.value / 604800).toString() }) :
		ago.value < -86400 ? i18n.tsx._timeIn.days({ n: Math.round(-ago.value / 86400).toString() }) :
		ago.value < -3600 ? i18n.tsx._timeIn.hours({ n: Math.round(-ago.value / 3600).toString() }) :
		ago.value < -60 ? i18n.tsx._timeIn.minutes({ n: (~~(-ago.value / 60)).toString() }) :
		i18n.tsx._timeIn.seconds({ n: (~~(-ago.value % 60)).toString() })
	);
});

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("time", {
      title: _unref(absolute),
      class: _normalizeClass({ [_ctx.$style.old1]: __props.colored && (ago.value > 60 * 60 * 24 * 90), [_ctx.$style.old2]: __props.colored && (ago.value > 60 * 60 * 24 * 180) })
    }, [ (_unref(invalid)) ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ _toDisplayString(_unref(i18n).ts._ago.invalid) ], 64 /* STABLE_FRAGMENT */)) : (__props.mode === 'relative') ? (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [ _toDisplayString(relative.value) ], 64 /* STABLE_FRAGMENT */)) : (__props.mode === 'absolute') ? (_openBlock(), _createElementBlock(_Fragment, { key: 2 }, [ _toDisplayString(_unref(absolute)) ], 64 /* STABLE_FRAGMENT */)) : (__props.mode === 'detail') ? (_openBlock(), _createElementBlock(_Fragment, { key: 3 }, [ _toDisplayString(_unref(absolute)), _createTextVNode(" ("), _toDisplayString(relative.value), _createTextVNode(")") ], 64 /* STABLE_FRAGMENT */)) : _createCommentVNode("v-if", true) ], 10 /* CLASS, PROPS */, ["title"]))
}
}

})
