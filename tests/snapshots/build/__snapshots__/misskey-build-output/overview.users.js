import { Fragment as _Fragment, Transition as _Transition, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, resolveComponent as _resolveComponent, renderList as _renderList, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue";
import { ref } from "vue";
import { useInterval } from "@@/js/use-interval.js";
import { misskeyApi } from "@/utility/misskey-api.js";
import MkUserCardMini from "@/components/MkUserCardMini.vue";
import { prefer } from "@/preferences.js";
export default {
  __name: "overview.users",
  setup(__props) {
    const newUsers = ref(null);
    const fetching = ref(true);
    const fetch = async () => {
      const _newUsers = await misskeyApi("admin/show-users", {
        limit: 5,
        sort: "+createdAt",
        origin: "local"
      });
      newUsers.value = _newUsers;
      fetching.value = false;
    };
    useInterval(fetch, 1e3 * 60, {
      immediate: true,
      afterMounted: true
    });
    return (_ctx, _cache) => {
      const _component_MkLoading = _resolveComponent("MkLoading");
      const _component_MkA = _resolveComponent("MkA");
      return _openBlock(), _createElementBlock(
        "div",
        { class: _normalizeClass(_ctx.$style.root) },
        [_createVNode(_Transition, {
          name: _unref(prefer).s.animation ? "_transition_zoom" : "",
          mode: "out-in"
        }, {
          default: _withCtx(() => [fetching.value ? (_openBlock(), _createBlock(_component_MkLoading, { key: 0 })) : (_openBlock(), _createElementBlock("div", {
            key: 1,
            class: "users"
          }, [(_openBlock(true), _createElementBlock(
            _Fragment,
            null,
            _renderList(newUsers.value, (user, i) => {
              return _openBlock(), _createBlock(_component_MkA, {
                key: user.id,
                to: `/admin/user/${user.id}`,
                class: "user"
              }, {
                default: _withCtx(() => [_createVNode(MkUserCardMini, { user }, null, 8, ["user"])]),
                _: 2
              }, 1032, ["to"]);
            }),
            128
            /* KEYED_FRAGMENT */
          ))]))]),
          _: 2
        }, 1032, ["name"])],
        2
        /* CLASS */
      );
    };
  }
};
