import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, withDirectives as _withDirectives, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, unref as _unref, vShow as _vShow } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("div", { class: "title" }, "Sub")
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("div", { class: "subTitle" }, "Top 10")
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("div", { class: "title" }, "Pub")
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("div", { class: "subTitle" }, "Top 10")
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-world-download" })
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("div", { class: "label" }, "Sub")
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-world-upload" })
const _hoisted_8 = /*#__PURE__*/ _createElementVNode("div", { class: "label" }, "Pub")
import { onMounted, ref } from 'vue'
import XPie from './overview.pie.vue'
import type { InstanceForPie } from './overview.pie.vue'
import * as os from '@/os.js'
import { misskeyApiGet } from '@/utility/misskey-api.js'
import number from '@/filters/number.js'
import MkNumberDiff from '@/components/MkNumberDiff.vue'
import { i18n } from '@/i18n.js'
import { useChartTooltip } from '@/composables/use-chart-tooltip.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'overview.federation',
  setup(__props) {

const topSubInstancesForPie = ref<InstanceForPie[] | null>(null);
const topPubInstancesForPie = ref<InstanceForPie[] | null>(null);
const federationPubActive = ref<number | null>(null);
const federationPubActiveDiff = ref<number | null>(null);
const federationSubActive = ref<number | null>(null);
const federationSubActiveDiff = ref<number | null>(null);
const fetching = ref(true);
const { handler: externalTooltipHandler } = useChartTooltip();
onMounted(async () => {
	const chart = await misskeyApiGet('charts/federation', { limit: 2, span: 'day' });
	federationPubActive.value = chart.pubActive[0];
	federationPubActiveDiff.value = chart.pubActive[0] - chart.pubActive[1];
	federationSubActive.value = chart.subActive[0];
	federationSubActiveDiff.value = chart.subActive[0] - chart.subActive[1];
	misskeyApiGet('federation/stats', { limit: 10 }).then(res => {
		topSubInstancesForPie.value = [
			...res.topSubInstances.map(x => ({
				name: x.host,
				color: x.themeColor,
				value: x.followersCount,
				onClick: () => {
					os.pageWindow(`/instance-info/${x.host}`);
				},
			})),
			{ name: '(other)', color: '#80808080', value: res.otherFollowersCount },
		];
		topPubInstancesForPie.value = [
			...res.topPubInstances.map(x => ({
				name: x.host,
				color: x.themeColor,
				value: x.followingCount,
				onClick: () => {
					os.pageWindow(`/instance-info/${x.host}`);
				},
			})),
			{ name: '(other)', color: '#80808080', value: res.otherFollowingCount },
		];
	});
	fetching.value = false;
});

return (_ctx: any,_cache: any) => {
  const _component_MkLoading = _resolveComponent("MkLoading")
  const _directive_tooltip = _resolveDirective("tooltip")

  return (_openBlock(), _createElementBlock("div", null, [ (fetching.value) ? (_openBlock(), _createBlock(_component_MkLoading, { key: 0 })) : _createCommentVNode("v-if", true), _withDirectives(_createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.root)
      }, [ (topSubInstancesForPie.value && topPubInstancesForPie.value) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: "pies"
          }, [ _createElementVNode("div", { class: "pie deliver _panel" }, [ _hoisted_1, _createVNode(XPie, {
                data: topSubInstancesForPie.value,
                class: "chart"
              }, null, 8 /* PROPS */, ["data"]), _hoisted_2 ]), _createElementVNode("div", { class: "pie inbox _panel" }, [ _hoisted_3, _createVNode(XPie, {
                data: topPubInstancesForPie.value,
                class: "chart"
              }, null, 8 /* PROPS */, ["data"]), _hoisted_4 ]) ])) : _createCommentVNode("v-if", true), (!fetching.value) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: "items"
          }, [ _createElementVNode("div", { class: "item _panel sub" }, [ _createElementVNode("div", { class: "icon" }, [ _hoisted_5 ]), _createElementVNode("div", { class: "body" }, [ (federationSubActive.value != null) ? (_openBlock(), _createElementBlock("div", {
                    key: 0,
                    class: "value"
                  }, [ _toDisplayString(number(federationSubActive.value)), _createTextVNode("\n\t\t\t\t\t\t"), (federationSubActiveDiff.value != null) ? (_openBlock(), _createBlock(MkNumberDiff, {
                        key: 0,
                        class: "diff",
                        value: federationSubActiveDiff.value
                      }, null, 8 /* PROPS */, ["value"])) : _createCommentVNode("v-if", true) ])) : _createCommentVNode("v-if", true), _hoisted_6 ]) ]), _createElementVNode("div", { class: "item _panel pub" }, [ _createElementVNode("div", { class: "icon" }, [ _hoisted_7 ]), _createElementVNode("div", { class: "body" }, [ (federationPubActive.value != null) ? (_openBlock(), _createElementBlock("div", {
                    key: 0,
                    class: "value"
                  }, [ _toDisplayString(number(federationPubActive.value)), _createTextVNode("\n\t\t\t\t\t\t"), (federationPubActiveDiff.value != null) ? (_openBlock(), _createBlock(MkNumberDiff, {
                        key: 0,
                        class: "diff",
                        value: federationPubActiveDiff.value
                      }, null, 8 /* PROPS */, ["value"])) : _createCommentVNode("v-if", true) ])) : _createCommentVNode("v-if", true), _hoisted_8 ]) ]) ])) : _createCommentVNode("v-if", true) ], 512 /* NEED_PATCH */), [ [_vShow, !fetching.value] ]) ]))
}
}

})
