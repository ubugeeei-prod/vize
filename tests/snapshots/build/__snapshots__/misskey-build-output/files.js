import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, renderList as _renderList, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue";
import { computed, markRaw } from "vue";
import MkNoteMediaGrid from "@/components/MkNoteMediaGrid.vue";
import MkPagination from "@/components/MkPagination.vue";
import { Paginator } from "@/utility/paginator.js";
export default {
  __name: "files",
  props: { user: {
    type: null,
    required: true
  } },
  setup(__props) {
    const props = __props;
    const paginator = markRaw(new Paginator("users/notes", {
      limit: 15,
      computedParams: computed(() => ({
        userId: props.user.id,
        withFiles: true
      }))
    }));
    return (_ctx, _cache) => {
      return _openBlock(), _createElementBlock("div", {
        class: "_spacer",
        style: "--MI_SPACER-w: 1100px;"
      }, [_createElementVNode(
        "div",
        { class: _normalizeClass(_ctx.$style.root) },
        [_createVNode(MkPagination, {
          paginator: _unref(paginator),
          withControl: ""
        }, {
          default: _withCtx(({ items }) => [_createElementVNode(
            "div",
            { class: _normalizeClass(_ctx.$style.stream) },
            [(_openBlock(true), _createElementBlock(
              _Fragment,
              null,
              _renderList(items, (note) => {
                return _openBlock(), _createBlock(MkNoteMediaGrid, {
                  note,
                  square: ""
                }, null, 8, ["note"]);
              }),
              256
              /* UNKEYED_FRAGMENT */
            ))],
            2
            /* CLASS */
          )]),
          _: 1
        }, 8, ["paginator"])],
        2
        /* CLASS */
      )]);
    };
  }
};
