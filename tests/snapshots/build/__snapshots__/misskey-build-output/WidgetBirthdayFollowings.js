import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-cake" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-refresh" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-chevron-up" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("span", { style: "height: 1em; width: 1px; background: var(--MI_THEME-divider);" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-chevron-down" })
import { computed, markRaw, ref, watch } from 'vue'
import { useLowresTime } from '@/composables/use-lowres-time.js'
import { isSeparatorNeeded, getSeparatorInfo } from '@/utility/timeline-date-separate.js'
import { useWidgetPropsManager } from './widget.js'
import type { WidgetComponentEmits, WidgetComponentExpose, WidgetComponentProps } from './widget.js'
import type { FormWithDefault, GetFormResultType } from '@/utility/form.js'
import MkContainer from '@/components/MkContainer.vue'
import MkPagination from '@/components/MkPagination.vue'
import XUser from './WidgetBirthdayFollowings.user.vue'
import { i18n } from '@/i18n.js'
import { Paginator } from '@/utility/paginator.js'

type WidgetProps = GetFormResultType<typeof widgetPropsDef>;
const name = 'birthdayFollowings';

export default /*@__PURE__*/_defineComponent({
  __name: 'WidgetBirthdayFollowings',
  setup(__props: any, { expose: __expose, emit: __emit }) {

const emit = __emit
const props = __props
const widgetPropsDef = {
	showHeader: {
		type: 'boolean',
		label: i18n.ts._widgetOptions.showHeader,
		default: true,
	},
	height: {
		type: 'number' as const,
		label: i18n.ts._widgetOptions.height,
		default: 300,
	},
	period: {
		type: 'radio' as const,
		label: i18n.ts._widgetOptions._birthdayFollowings.period,
		default: '3day',
		options: [{
			value: 'today' as const,
			label: i18n.ts.today,
		}, {
			value: '3day' as const,
			label: i18n.tsx.dayX({ day: 3 }),
		}, {
			value: 'week' as const,
			label: i18n.ts.oneWeek,
		}, {
			value: 'month' as const,
			label: i18n.ts.oneMonth,
		}],
	},
} satisfies FormWithDefault;
const { widgetProps, configure } = useWidgetPropsManager(
	name,
	widgetPropsDef,
	props,
	emit,
);
const now = useLowresTime();
const nextDay = new Date();
nextDay.setHours(24, 0, 0, 0);
let nextDayMidnightTime = nextDay.getTime();
const begin = ref<Date>(new Date());
const end = computed(() => {
	switch (widgetProps.period) {
		case '3day':
			return new Date(begin.value.getTime() + 1000 * 60 * 60 * 24 * 3);
		case 'week':
			return new Date(begin.value.getTime() + 1000 * 60 * 60 * 24 * 7);
		case 'month':
			return new Date(begin.value.getTime() + 1000 * 60 * 60 * 24 * 30);
		default:
			return begin.value;
	}
});
const birthdayUsersPaginator = markRaw(new Paginator('users/get-following-birthday-users', {
	limit: 18,
	offsetMode: true,
	computedParams: computed(() => {
		if (widgetProps.period === 'today') {
			return {
				birthday: {
					month: begin.value.getMonth() + 1,
					day: begin.value.getDate(),
				},
			};
		} else {
			return {
				birthday: {
					begin: {
						month: begin.value.getMonth() + 1,
						day: begin.value.getDate(),
					},
					end: {
						month: end.value.getMonth() + 1,
						day: end.value.getDate(),
					},
				},
			};
		}
	}),
}));
function fetch() {
	const now = new Date();
	begin.value = now;
}
const UPDATE_INTERVAL = 1000 * 60;
let nextDayTimer: number | null = null;
watch(now, (to) => {
	// 次回更新までに日付が変わる場合、日付が変わった直後に強制的に更新するタイマーをセットする
	if (nextDayMidnightTime - to <= UPDATE_INTERVAL) {
		if (nextDayTimer != null) {
			window.clearTimeout(nextDayTimer);
			nextDayTimer = null;
		}
		nextDayTimer = window.setTimeout(() => {
			fetch();
			nextDay.setHours(24, 0, 0, 0);
			nextDayMidnightTime = nextDay.getTime();
			nextDayTimer = null;
		}, nextDayMidnightTime - to);
	}
}, { immediate: true });
__expose({
	name,
	configure,
	id: props.widget ? props.widget.id : null,
})

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createBlock(MkContainer, {
      style: _normalizeStyle(`height: ${_unref(widgetProps).height}px;`),
      showHeader: _unref(widgetProps).showHeader,
      scrollable: true,
      class: "mkw-bdayfollowings"
    }, {
      icon: _withCtx(() => [
        _hoisted_1
      ]),
      header: _withCtx(() => [
        _createTextVNode(_toDisplayString(_unref(i18n).ts._widgets.birthdayFollowings), 1 /* TEXT */)
      ]),
      func: _withCtx(({ buttonStyleClass }) => [
        _createElementVNode("button", {
          class: _normalizeClass(["_button", buttonStyleClass]),
          onClick: fetch
        }, [
          _hoisted_2
        ])
      ]),
      default: _withCtx(() => [
        _createVNode(MkPagination, { paginator: _unref(birthdayUsersPaginator) }, {
          default: _withCtx(({ items }) => [
            _createElementVNode("div", null, [
              (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (user, i) => {
                return (_openBlock(), _createElementBlock(_Fragment, { key: user.id }, [
                  (i > 0 && _unref(isSeparatorNeeded)(_unref(birthdayUsersPaginator).items.value[i - 1].birthday, user.birthday))
                    ? (_openBlock(), _createElementBlock("div", { key: 0 }, [
                      _createElementVNode("div", {
                        class: _normalizeClass(_ctx.$style.date)
                      }, [
                        _createElementVNode("span", null, [
                          _hoisted_3,
                          _createTextVNode(" " + _toDisplayString(_unref(getSeparatorInfo)(_unref(birthdayUsersPaginator).items.value[i - 1].birthday, user.birthday)?.prevText), 1 /* TEXT */)
                        ]),
                        _hoisted_4,
                        _createElementVNode("span", null, [
                          _createTextVNode(_toDisplayString(_unref(getSeparatorInfo)(_unref(birthdayUsersPaginator).items.value[i - 1].birthday, user.birthday)?.nextText) + " ", 1 /* TEXT */),
                          _hoisted_5
                        ])
                      ]),
                      _createVNode(XUser, {
                        class: _normalizeClass(_ctx.$style.user),
                        item: user
                      }, null, 8 /* PROPS */, ["item"])
                    ]))
                    : (_openBlock(), _createBlock(XUser, {
                      key: 1,
                      class: _normalizeClass(_ctx.$style.user),
                      item: user
                    }, null, 8 /* PROPS */, ["item"]))
                ], 64 /* STABLE_FRAGMENT */))
              }), 128 /* KEYED_FRAGMENT */))
            ])
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["paginator"])
      ]),
      _: 1 /* STABLE */
    }, 12 /* STYLE, PROPS */, ["showHeader", "scrollable"]))
}
}

})
