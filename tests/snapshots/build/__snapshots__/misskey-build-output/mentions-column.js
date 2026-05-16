import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", {
  class: "ti ti-at",
  style: "margin-right: 8px;"
});
import { markRaw } from "vue";
import XColumn from "./column.vue";
import { i18n } from "@/i18n.js";
import MkNotesTimeline from "@/components/MkNotesTimeline.vue";
import { Paginator } from "@/utility/paginator.js";
export default {
  __name: "mentions-column",
  props: {
    column: {
      type: null,
      required: true
    },
    isStacked: {
      type: Boolean,
      required: true
    }
  },
  setup(__props) {
    const paginator = markRaw(new Paginator("notes/mentions", { limit: 10 }));
    function reloadTimeline() {
      return new Promise((res) => {
        paginator.reload().then(() => {
          res();
        });
      });
    }
    return (_ctx, _cache) => {
      return _openBlock(), _createBlock(XColumn, {
        column: __props.column,
        isStacked: __props.isStacked,
        refresher: () => reloadTimeline()
      }, {
        header: _withCtx(() => [_hoisted_1, _createTextVNode(
          _toDisplayString(__props.column.name || _unref(i18n).ts._deck._columns.mentions),
          1
          /* TEXT */
        )]),
        default: _withCtx(() => [_createVNode(MkNotesTimeline, { paginator: _unref(paginator) }, null, 8, ["paginator"])]),
        _: 1
      }, 8, [
        "column",
        "isStacked",
        "refresher"
      ]);
    };
  }
};
