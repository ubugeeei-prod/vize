import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass } from "vue";
export default {
  __name: "MkNumberCell",
  props: {
    content: {
      type: String,
      required: true
    },
    row: {
      type: null,
      required: false
    }
  },
  setup(__props) {
    return (_ctx, _cache) => {
      return _openBlock(), _createElementBlock("div", {
        class: _normalizeClass(["mk_grid_th", [_ctx.$style.cell]]),
        tabindex: -1,
        "data-grid-cell": "",
        "data-grid-cell-row": __props.row?.index ?? -1,
        "data-grid-cell-col": -1
      }, [_createElementVNode(
        "div",
        { class: _normalizeClass([_ctx.$style.root]) },
        _toDisplayString(__props.content),
        3
        /* TEXT, CLASS */
      )], 10, [
        "tabindex",
        "data-grid-cell-row",
        "data-grid-cell-col"
      ]);
    };
  }
};
