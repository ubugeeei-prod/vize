import { Fragment as _Fragment, Transition as _Transition, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, withDirectives as _withDirectives, renderList as _renderList, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue";
import { ref } from "vue";
import { useInterval } from "@@/js/use-interval.js";
import { misskeyApi } from "@/utility/misskey-api.js";
import MkInstanceCardMini from "@/components/MkInstanceCardMini.vue";
import { prefer } from "@/preferences.js";
export default {
  __name: "overview.instances",
  setup(__props) {
    const instances = ref([]);
    const fetching = ref(true);
    const fetch = async () => {
      const fetchedInstances = await misskeyApi("federation/instances", {
        sort: "+latestRequestReceivedAt",
        limit: 6
      });
      instances.value = fetchedInstances;
      fetching.value = false;
    };
    useInterval(fetch, 1e3 * 60, {
      immediate: true,
      afterMounted: true
    });
    return (_ctx, _cache) => {
      const _component_MkLoading = _resolveComponent("MkLoading");
      const _component_MkA = _resolveComponent("MkA");
      const _directive_tooltip = _resolveDirective("tooltip");
      return _openBlock(), _createElementBlock("div", null, [_createVNode(_Transition, {
        name: _unref(prefer).s.animation ? "_transition_zoom" : "",
        mode: "out-in"
      }, {
        default: _withCtx(() => [fetching.value ? (_openBlock(), _createBlock(_component_MkLoading, { key: 0 })) : (_openBlock(), _createElementBlock(
          "div",
          {
            key: 1,
            class: _normalizeClass(_ctx.$style.instances)
          },
          [(_openBlock(true), _createElementBlock(
            _Fragment,
            null,
            _renderList(instances.value, (instance, i) => {
              return _withDirectives((_openBlock(), _createBlock(_component_MkA, {
                key: instance.id,
                to: `/instance-info/${instance.host}`,
                class: _normalizeClass(_ctx.$style.instance)
              }, {
                default: _withCtx(() => [_createVNode(MkInstanceCardMini, { instance }, null, 8, ["instance"])]),
                _: 2
              }, 1034, ["to"])), [[
                _directive_tooltip,
                `${instance.name}\n${instance.host}\n${instance.softwareName} ${instance.softwareVersion}`,
                void 0,
                {
                  mfm: true,
                  noDelay: true
                }
              ]]);
            }),
            128
            /* KEYED_FRAGMENT */
          ))],
          2
          /* CLASS */
        ))]),
        _: 2
      }, 1032, ["name"])]);
    };
  }
};
