import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass } from "vue"


const _hoisted_1 = { class: "host" }
import { ref } from 'vue'
import * as Misskey from 'misskey-js'
import MkMiniChart from '@/components/MkMiniChart.vue'
import { misskeyApiGet } from '@/utility/misskey-api.js'
import { getProxiedImageUrlNullable } from '@/utility/media-proxy.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkInstanceCardMini',
  props: {
    instance: { type: null, required: true }
  },
  setup(__props: any) {

const props = __props
const chartValues = ref<number[] | null>(null);
misskeyApiGet('charts/instance', { host: props.instance.host, limit: 16 + 1, span: 'day' }).then(res => {
	// 今日のぶんの値はまだ途中の値であり、それも含めると大抵の場合前日よりも下降しているようなグラフになってしまうため今日は弾く
	res.requests.received.splice(0, 1);
	chartValues.value = res.requests.received;
});
function getInstanceIcon(instance: Misskey.entities.FederationInstance): string {
	return getProxiedImageUrlNullable(instance.iconUrl, 'preview') ?? getProxiedImageUrlNullable(instance.faviconUrl, 'preview') ?? '/client-assets/dummy.png';
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass([_ctx.$style.root, { yellow: __props.instance.isNotResponding, red: __props.instance.isBlocked, gray: __props.instance.isSuspended, blue: __props.instance.isSilenced }])
    }, [ _createElementVNode("img", {
        class: "icon",
        src: getInstanceIcon(__props.instance),
        alt: "",
        loading: "lazy"
      }, null, 8 /* PROPS */, ["src"]), _createElementVNode("div", { class: "body" }, [ _createElementVNode("span", _hoisted_1, _toDisplayString(__props.instance.name ?? __props.instance.host), 1 /* TEXT */), _createElementVNode("span", { class: "sub _monospace" }, [ _createElementVNode("b", null, _toDisplayString(__props.instance.host), 1 /* TEXT */), _createTextVNode(" / " + _toDisplayString(__props.instance.softwareName || '?') + " " + _toDisplayString(__props.instance.softwareVersion), 1 /* TEXT */) ]) ]), (chartValues.value) ? (_openBlock(), _createBlock(MkMiniChart, {
          key: 0,
          class: "chart",
          src: chartValues.value
        }, null, 8 /* PROPS */, ["src"])) : _createCommentVNode("v-if", true) ], 2 /* CLASS */))
}
}

})
