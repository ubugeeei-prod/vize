import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createCommentVNode as _createCommentVNode, renderList as _renderList, normalizeClass as _normalizeClass } from "vue";
import MkHeaderCell from "@/components/grid/MkHeaderCell.vue";
import MkNumberCell from "@/components/grid/MkNumberCell.vue";
export default {
  __name: "MkHeaderRow",
  props: {
    columns: {
      type: Array,
      required: true
    },
    gridSetting: {
      type: null,
      required: true
    },
    bus: {
      type: null,
      required: true
    }
  },
  emits: [
    "operation:beginWidthChange",
    "operation:endWidthChange",
    "operation:widthLargest",
    "operation:selectionColumn",
    "change:width",
    "change:contentSize"
  ],
  setup(__props, { emit: __emit }) {
    const emit = __emit;
    return (_ctx, _cache) => {
      return _openBlock(), _createElementBlock("div", {
        class: _normalizeClass(["mk_grid_tr", _ctx.$style.root]),
        "data-grid-row": -1
      }, [__props.gridSetting.showNumber ? (_openBlock(), _createBlock(MkNumberCell, {
        key: 0,
        content: "#",
        top: true
      }, null, 8, ["top"])) : _createCommentVNode("v-if", true), (_openBlock(true), _createElementBlock(
        _Fragment,
        null,
        _renderList(__props.columns, (column) => {
          return _openBlock(), _createBlock(MkHeaderCell, {
            key: column.index,
            column,
            bus: __props.bus,
            "onOperation:beginWidthChange": (sender) => emit("operation:beginWidthChange", sender),
            "onOperation:endWidthChange": (sender) => emit("operation:endWidthChange", sender),
            "onOperation:widthLargest": (sender) => emit("operation:widthLargest", sender),
            "onChange:width": (sender, width) => emit("change:width", sender, width),
            "onChange:contentSize": (sender, newSize) => emit("change:contentSize", sender, newSize)
          }, null, 8, [
            "column",
            "bus",
            "onOperation:beginWidthChange",
            "onOperation:endWidthChange",
            "onOperation:widthLargest",
            "onChange:width",
            "onChange:contentSize"
          ]);
        }),
        128
        /* KEYED_FRAGMENT */
      ))], 10, ["data-grid-row"]);
    };
  }
};
