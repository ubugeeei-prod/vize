import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, Transition as _Transition, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, renderList as _renderList, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"

import { ref } from 'vue'
import * as Misskey from 'misskey-js'
import { useInterval } from '@@/js/use-interval.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import MkInstanceCardMini from '@/components/MkInstanceCardMini.vue'
import { prefer } from '@/preferences.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'overview.instances',
  setup(__props) {

const instances = ref<Misskey.entities.FederationInstance[]>([]);
const fetching = ref(true);
const fetch = async () => {
	const fetchedInstances = await misskeyApi('federation/instances', {
		sort: '+latestRequestReceivedAt',
		limit: 6,
	});
	instances.value = fetchedInstances;
	fetching.value = false;
};
useInterval(fetch, 1000 * 60, {
	immediate: true,
	afterMounted: true,
});

return (_ctx: any,_cache: any) => {
  const _component_MkLoading = _resolveComponent("MkLoading")
  const _component_MkA = _resolveComponent("MkA")
  const _directive_tooltip = _resolveDirective("tooltip")

  return (_openBlock(), _createElementBlock("div", null, [ _createVNode(_Transition, {
        name: _unref(prefer).s.animation ? '_transition_zoom' : '',
        mode: "out-in"
      }, {
        default: _withCtx(() => [
          (fetching.value)
            ? (_openBlock(), _createBlock(_component_MkLoading, { key: 0 }))
            : (_openBlock(), _createElementBlock("div", {
              key: 1,
              class: _normalizeClass(_ctx.$style.instances)
            }, [
              (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(instances.value, (instance, i) => {
                return (_openBlock(), _createBlock(_component_MkA, {
                  key: instance.id,
                  to: `/instance-info/${instance.host}`,
                  class: _normalizeClass(_ctx.$style.instance)
                }, {
                  default: _withCtx(() => [
                    _createVNode(MkInstanceCardMini, { instance: instance }, null, 8 /* PROPS */, ["instance"])
                  ]),
                  _: 2 /* DYNAMIC */
                }, 1032 /* PROPS, DYNAMIC_SLOTS */, ["to"]))
              }), 128 /* KEYED_FRAGMENT */))
            ]))
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["name"]) ]))
}
}

})
