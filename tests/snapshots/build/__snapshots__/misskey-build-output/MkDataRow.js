import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createCommentVNode as _createCommentVNode, renderList as _renderList, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle } from "vue";
import MkDataCell from "@/components/grid/MkDataCell.vue";
import MkNumberCell from "@/components/grid/MkNumberCell.vue";
export default {
  __name: "MkDataRow",
  props: {
    row: {
      type: null,
      required: true
    },
    cells: {
      type: Array,
      required: true
    },
    setting: {
      type: null,
      required: true
    },
    bus: {
      type: null,
      required: true
    }
  },
  emits: [
    "operation:beginEdit",
    "operation:endEdit",
    "change:value",
    "change:contentSize"
  ],
  setup(__props, { emit: __emit }) {
    const emit = __emit;
    return (_ctx, _cache) => {
      return _openBlock(), _createElementBlock("div", {
        class: _normalizeClass(["mk_grid_tr", [
          _ctx.$style.row,
          __props.row.ranged ? _ctx.$style.ranged : {},
          ...(__props.row.additionalStyles ?? []).map((it) => it.className ?? {})
        ]]),
        style: _normalizeStyle([...(__props.row.additionalStyles ?? []).map((it) => it.style ?? {})]),
        "data-grid-row": __props.row.index
      }, [__props.setting.showNumber ? (_openBlock(), _createBlock(MkNumberCell, {
        key: 0,
        content: (__props.row.index + 1).toString(),
        row: __props.row
      }, null, 8, ["content", "row"])) : _createCommentVNode("v-if", true), (_openBlock(true), _createElementBlock(
        _Fragment,
        null,
        _renderList(__props.cells, (cell) => {
          return _openBlock(), _createBlock(MkDataCell, {
            key: cell.address.col,
            vIf: cell.column.setting.type !== "hidden",
            cell,
            rowSetting: __props.setting,
            bus: __props.bus,
            "onOperation:beginEdit": (sender) => emit("operation:beginEdit", sender),
            "onOperation:endEdit": (sender) => emit("operation:endEdit", sender),
            "onChange:value": (sender, newValue) => emit("change:value", sender, newValue),
            "onChange:contentSize": (sender, newSize) => emit("change:contentSize", sender, newSize)
          }, null, 8, [
            "vIf",
            "cell",
            "rowSetting",
            "bus",
            "onOperation:beginEdit",
            "onOperation:endEdit",
            "onChange:value",
            "onChange:contentSize"
          ]);
        }),
        128
        /* KEYED_FRAGMENT */
      ))], 14, ["data-grid-row"]);
    };
  }
};
