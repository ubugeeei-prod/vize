import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, TransitionGroup as _TransitionGroup, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-hash" })
import { ref } from 'vue'
import * as Misskey from 'misskey-js'
import { useInterval } from '@@/js/use-interval.js'
import { useWidgetPropsManager } from './widget.js'
import type { WidgetComponentEmits, WidgetComponentExpose, WidgetComponentProps } from './widget.js'
import type { FormWithDefault, GetFormResultType } from '@/utility/form.js'
import MkContainer from '@/components/MkContainer.vue'
import MkMiniChart from '@/components/MkMiniChart.vue'
import { misskeyApiGet } from '@/utility/misskey-api.js'
import { i18n } from '@/i18n.js'
import { prefer } from '@/preferences.js'

type WidgetProps = GetFormResultType<typeof widgetPropsDef>;
const name = 'trends';

export default /*@__PURE__*/_defineComponent({
  __name: 'WidgetTrends',
  setup(__props: any, { expose: __expose, emit: __emit }) {

const emit = __emit
const props = __props
const widgetPropsDef = {
	showHeader: {
		type: 'boolean',
		label: i18n.ts._widgetOptions.showHeader,
		default: true,
	},
} satisfies FormWithDefault;
const { widgetProps, configure } = useWidgetPropsManager(name,
	widgetPropsDef,
	props,
	emit,
);
const stats = ref<Misskey.entities.HashtagsTrendResponse>([]);
const fetching = ref(true);
const fetch = () => {
	misskeyApiGet('hashtags/trend').then(res => {
		stats.value = res;
		fetching.value = false;
	});
};
useInterval(fetch, 1000 * 60, {
	immediate: true,
	afterMounted: true,
});
__expose({
	name,
	configure,
	id: props.widget ? props.widget.id : null,
})

return (_ctx: any,_cache: any) => {
  const _component_MkLoading = _resolveComponent("MkLoading")
  const _component_MkA = _resolveComponent("MkA")

  return (_openBlock(), _createBlock(MkContainer, {
      showHeader: _unref(widgetProps).showHeader,
      "data-cy-mkw-trends": "",
      class: "mkw-trends"
    }, {
      icon: _withCtx(() => [
        _hoisted_1
      ]),
      header: _withCtx(() => [
        _createTextVNode(_toDisplayString(_unref(i18n).ts._widgets.trends), 1 /* TEXT */)
      ]),
      default: _withCtx(() => [
        _createElementVNode("div", { class: "wbrkwala" }, [
          (fetching.value)
            ? (_openBlock(), _createBlock(_component_MkLoading, { key: 0 }))
            : (_openBlock(), _createBlock(_TransitionGroup, {
              key: 1,
              tag: "div",
              name: _unref(prefer).s.animation ? 'chart' : '',
              class: "tags"
            }, {
              default: _withCtx(() => [
                (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(stats.value, (stat) => {
                  return (_openBlock(), _createElementBlock("div", { key: stat.tag }, [
                    _createElementVNode("div", { class: "tag" }, [
                      _createVNode(_component_MkA, {
                        class: "a",
                        to: `/tags/${ encodeURIComponent(stat.tag) }`,
                        title: stat.tag
                      }, {
                        default: _withCtx(() => [
                          _createTextVNode("#"),
                          _createTextVNode(_toDisplayString(stat.tag), 1 /* TEXT */)
                        ]),
                        _: 2 /* DYNAMIC */
                      }, 8 /* PROPS */, ["to", "title"]),
                      _createElementVNode("p", null, _toDisplayString(_unref(i18n).tsx.nUsersMentioned({ n: stat.usersCount })), 1 /* TEXT */)
                    ]),
                    _createVNode(MkMiniChart, {
                      class: "chart",
                      src: stat.chart
                    }, null, 8 /* PROPS */, ["src"])
                  ]))
                }), 128 /* KEYED_FRAGMENT */))
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["name"]))
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["showHeader"]))
}
}

})
