import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, Transition as _Transition, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span")
import { ref } from 'vue'
import * as Misskey from 'misskey-js'
import { useInterval } from '@@/js/use-interval.js'
import MkMarqueeText from '@/components/MkMarqueeText.vue'
import { misskeyApi } from '@/utility/misskey-api.js'
import { getProxiedImageUrlNullable } from '@/utility/media-proxy.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'statusbar-federation',
  props: {
    display: { type: String, required: false },
    colored: { type: Boolean, required: false },
    marqueeDuration: { type: Number, required: false },
    marqueeReverse: { type: Boolean, required: false },
    oneByOneInterval: { type: Number, required: false },
    refreshIntervalSec: { type: Number, required: true }
  },
  setup(__props: any) {

const props = __props
const instances = ref<Misskey.entities.FederationInstance[]>([]);
const fetching = ref(true);
const key = ref(0);
const tick = () => {
	misskeyApi('federation/instances', {
		sort: '+latestRequestReceivedAt',
		limit: 30,
	}).then(res => {
		instances.value = res;
		fetching.value = false;
		key.value++;
	});
};
useInterval(tick, Math.max(5000, props.refreshIntervalSec * 1000), {
	immediate: true,
	afterMounted: true,
});
function getInstanceIcon(instance: Misskey.entities.FederationInstance): string {
	return getProxiedImageUrlNullable(instance.iconUrl, 'preview') ?? getProxiedImageUrlNullable(instance.faviconUrl, 'preview') ?? '/client-assets/dummy.png';
}

return (_ctx: any,_cache: any) => {
  const _component_MkA = _resolveComponent("MkA")

  return (!fetching.value)
      ? (_openBlock(), _createElementBlock("span", {
        key: 0,
        class: _normalizeClass(_ctx.$style.root)
      }, [ (__props.display === 'marquee') ? (_openBlock(), _createBlock(_Transition, {
            key: 0,
            enterActiveClass: _ctx.$style.transition_change_enterActive,
            leaveActiveClass: _ctx.$style.transition_change_leaveActive,
            enterFromClass: _ctx.$style.transition_change_enterFrom,
            leaveToClass: _ctx.$style.transition_change_leaveTo,
            mode: "default"
          }, {
            default: _withCtx(() => [
              _createVNode(MkMarqueeText, {
                key: key.value,
                duration: __props.marqueeDuration,
                reverse: __props.marqueeReverse
              }, {
                default: _withCtx(() => [
                  (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(instances.value, (instance) => {
                    return (_openBlock(), _createElementBlock("span", {
                      key: instance.id,
                      class: _normalizeClass([_ctx.$style.item, { [_ctx.$style.colored]: __props.colored }]),
                      style: _normalizeStyle({ background: __props.colored ? instance.themeColor ?? '' : '' })
                    }, [
                      _createElementVNode("img", {
                        class: _normalizeClass(_ctx.$style.icon),
                        src: getInstanceIcon(instance),
                        alt: ""
                      }, null, 8 /* PROPS */, ["src"]),
                      _createVNode(_component_MkA, {
                        to: `/instance-info/${instance.host}`,
                        class: _normalizeClass(["_monospace", _ctx.$style.host])
                      }, {
                        default: _withCtx(() => [
                          _createTextVNode(_toDisplayString(instance.host), 1 /* TEXT */)
                        ]),
                        _: 2 /* DYNAMIC */
                      }, 8 /* PROPS */, ["to"]),
                      _hoisted_1
                    ], 6 /* CLASS, STYLE */))
                  }), 128 /* KEYED_FRAGMENT */))
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["duration", "reverse"])
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["enterActiveClass", "leaveActiveClass", "enterFromClass", "leaveToClass"])) : (__props.display === 'oneByOne') ? (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [ ], 64 /* STABLE_FRAGMENT */)) : _createCommentVNode("v-if", true) ]))
      : _createCommentVNode("v-if", true)
}
}

})
