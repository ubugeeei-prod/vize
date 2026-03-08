import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, withDirectives as _withDirectives, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, vShow as _vShow } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-chart-line" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-selector" })
import { ref } from 'vue'
import { useWidgetPropsManager } from './widget.js'
import type { WidgetComponentProps, WidgetComponentEmits, WidgetComponentExpose } from './widget.js'
import XCalendar from './WidgetActivity.calendar.vue'
import XChart from './WidgetActivity.chart.vue'
import type { FormWithDefault, GetFormResultType } from '@/utility/form.js'
import { misskeyApiGet } from '@/utility/misskey-api.js'
import MkContainer from '@/components/MkContainer.vue'
import { ensureSignin } from '@/i.js'
import { i18n } from '@/i18n.js'

type WidgetProps = GetFormResultType<typeof widgetPropsDef>;
const name = 'activity';

export default /*@__PURE__*/_defineComponent({
  __name: 'WidgetActivity',
  setup(__props: any, { expose: __expose, emit: __emit }) {

const emit = __emit
const props = __props
const $i = ensureSignin();
const widgetPropsDef = {
	showHeader: {
		type: 'boolean',
		label: i18n.ts._widgetOptions.showHeader,
		default: true,
	},
	transparent: {
		type: 'boolean',
		label: i18n.ts._widgetOptions.transparent,
		default: false,
	},
	view: {
		type: 'number',
		default: 0,
		hidden: true,
	},
} satisfies FormWithDefault;
const { widgetProps, configure, save } = useWidgetPropsManager(name,
	widgetPropsDef,
	props,
	emit,
);
const activity = ref<{
	total: number;
	notes: number;
	replies: number;
	renotes: number;
}[] | null>(null);
const fetching = ref(true);
const toggleView = () => {
	if (widgetProps.view === 1) {
		widgetProps.view = 0;
	} else {
		widgetProps.view++;
	}
	save();
};
misskeyApiGet('charts/user/notes', {
	userId: $i.id,
	span: 'day',
	limit: 7 * 21,
}).then(res => {
	activity.value = res.diffs.normal.map((_, i) => ({
		total: res.diffs.normal[i] + res.diffs.reply[i] + res.diffs.renote[i],
		notes: res.diffs.normal[i],
		replies: res.diffs.reply[i],
		renotes: res.diffs.renote[i],
	}));
	fetching.value = false;
});
__expose({
	name,
	configure,
	id: props.widget ? props.widget.id : null,
})

return (_ctx: any,_cache: any) => {
  const _component_MkLoading = _resolveComponent("MkLoading")

  return (_openBlock(), _createBlock(MkContainer, {
      showHeader: _unref(widgetProps).showHeader,
      naked: _unref(widgetProps).transparent,
      "data-cy-mkw-activity": "",
      class: "mkw-activity"
    }, {
      icon: _withCtx(() => [
        _hoisted_1
      ]),
      header: _withCtx(() => [
        _createTextVNode(_toDisplayString(_unref(i18n).ts._widgets.activity), 1 /* TEXT */)
      ]),
      func: _withCtx(({ buttonStyleClass }) => [
        _createElementVNode("button", {
          class: _normalizeClass(["_button", buttonStyleClass]),
          onClick: _cache[0] || (_cache[0] = ($event: any) => (toggleView()))
        }, [
          _hoisted_2
        ])
      ]),
      default: _withCtx(() => [
        _createElementVNode("div", null, [
          (fetching.value)
            ? (_openBlock(), _createBlock(_component_MkLoading, { key: 0 }))
            : (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [
              _withDirectives(_createVNode(XCalendar, { activity: activity.value ?? [] }, null, 8 /* PROPS */, ["activity"]), [
                [_vShow, _unref(widgetProps).view === 0]
              ]),
              _withDirectives(_createVNode(XChart, { activity: activity.value ?? [] }, null, 8 /* PROPS */, ["activity"]), [
                [_vShow, _unref(widgetProps).view === 1]
              ])
            ], 64 /* STABLE_FRAGMENT */))
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["showHeader", "naked"]))
}
}

})
