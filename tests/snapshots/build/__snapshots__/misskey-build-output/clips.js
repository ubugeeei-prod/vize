import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue";
import { computed, markRaw } from "vue";
import MkPagination from "@/components/MkPagination.vue";
import { Paginator } from "@/utility/paginator.js";
export default {
  __name: "clips",
  props: { user: {
    type: null,
    required: true
  } },
  setup(__props) {
    const props = __props;
    const paginator = markRaw(new Paginator("users/clips", {
      limit: 20,
      computedParams: computed(() => ({ userId: props.user.id }))
    }));
    return (_ctx, _cache) => {
      const _component_MkA = _resolveComponent("MkA");
      return _openBlock(), _createElementBlock("div", {
        class: "_spacer",
        style: "--MI_SPACER-w: 700px;"
      }, [_createElementVNode("div", null, [_createVNode(MkPagination, {
        paginator: _unref(paginator),
        withControl: ""
      }, {
        default: _withCtx(({ items }) => [(_openBlock(true), _createElementBlock(
          _Fragment,
          null,
          _renderList(items, (item) => {
            return _openBlock(), _createBlock(_component_MkA, {
              key: item.id,
              to: `/clips/${item.id}`,
              class: _normalizeClass(["_panel _margin", _ctx.$style.item])
            }, {
              default: _withCtx(() => [_createElementVNode(
                "b",
                null,
                _toDisplayString(item.name),
                1
                /* TEXT */
              ), item.description ? (_openBlock(), _createElementBlock(
                "div",
                {
                  key: 0,
                  class: _normalizeClass(_ctx.$style.description)
                },
                _toDisplayString(item.description),
                3
                /* TEXT, CLASS */
              )) : _createCommentVNode("v-if", true)]),
              _: 2
            }, 1034, ["to"]);
          }),
          128
          /* KEYED_FRAGMENT */
        ))]),
        _: 2
      }, 1032, ["paginator"])])]);
    };
  }
};
