import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, TransitionGroup as _TransitionGroup, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-whirl" })
import { ref } from 'vue'
import * as Misskey from 'misskey-js'
import { useInterval } from '@@/js/use-interval.js'
import { useWidgetPropsManager } from './widget.js'
import type { WidgetComponentEmits, WidgetComponentExpose, WidgetComponentProps } from './widget.js'
import type { FormWithDefault, GetFormResultType } from '@/utility/form.js'
import MkContainer from '@/components/MkContainer.vue'
import MkMiniChart from '@/components/MkMiniChart.vue'
import { misskeyApi, misskeyApiGet } from '@/utility/misskey-api.js'
import { i18n } from '@/i18n.js'
import { getProxiedImageUrlNullable } from '@/utility/media-proxy.js'
import { prefer } from '@/preferences.js'

type WidgetProps = GetFormResultType<typeof widgetPropsDef>;
const name = 'federation';

export default /*@__PURE__*/_defineComponent({
  __name: 'WidgetFederation',
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
const instances = ref<Misskey.entities.FederationInstance[]>([]);
const charts = ref<Misskey.entities.ChartsInstanceResponse[]>([]);
const fetching = ref(true);
async function fetchInstances() {
	const fetchedInstances = await misskeyApi('federation/instances', {
		sort: '+latestRequestReceivedAt',
		limit: 5,
	});
	const fetchedCharts = await Promise.all(fetchedInstances.map(i => misskeyApiGet('charts/instance', { host: i.host, limit: 16, span: 'hour' })));
	instances.value = fetchedInstances;
	charts.value = fetchedCharts;
	fetching.value = false;
}
useInterval(fetchInstances, 1000 * 60, {
	immediate: true,
	afterMounted: true,
});
function getInstanceIcon(instance: Misskey.entities.FederationInstance): string {
	return getProxiedImageUrlNullable(instance.iconUrl, 'preview') ?? getProxiedImageUrlNullable(instance.faviconUrl, 'preview') ?? '/client-assets/dummy.png';
}
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
      "data-cy-mkw-federation": "",
      class: "mkw-federation"
    }, {
      icon: _withCtx(() => [
        _hoisted_1
      ]),
      header: _withCtx(() => [
        _createTextVNode(_toDisplayString(_unref(i18n).ts._widgets.federation), 1 /* TEXT */)
      ]),
      default: _withCtx(() => [
        _createElementVNode("div", { class: "wbrkwalb" }, [
          (fetching.value)
            ? (_openBlock(), _createBlock(_component_MkLoading, { key: 0 }))
            : (_openBlock(), _createBlock(_TransitionGroup, {
              key: 1,
              tag: "div",
              name: _unref(prefer).s.animation ? 'chart' : '',
              class: "instances"
            }, {
              default: _withCtx(() => [
                (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(instances.value, (instance, i) => {
                  return (_openBlock(), _createElementBlock("div", {
                    key: instance.id,
                    class: "instance"
                  }, [
                    _createElementVNode("img", {
                      src: getInstanceIcon(instance),
                      alt: ""
                    }, null, 8 /* PROPS */, ["src"]),
                    _createElementVNode("div", { class: "body" }, [
                      _createVNode(_component_MkA, {
                        class: "a",
                        to: `/instance-info/${instance.host}`,
                        behavior: "window",
                        title: instance.host
                      }, {
                        default: _withCtx(() => [
                          _createTextVNode(_toDisplayString(instance.host), 1 /* TEXT */)
                        ]),
                        _: 2 /* DYNAMIC */
                      }, 8 /* PROPS */, ["to", "title"]),
                      _createElementVNode("p", null, _toDisplayString(instance.softwareName || '?') + " " + _toDisplayString(instance.softwareVersion), 1 /* TEXT */)
                    ]),
                    _createVNode(MkMiniChart, {
                      class: "chart",
                      src: charts.value[i].requests.received
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
