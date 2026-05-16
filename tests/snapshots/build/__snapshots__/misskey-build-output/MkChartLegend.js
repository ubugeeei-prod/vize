import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle } from "vue";
import { shallowRef } from "vue";
export default {
  __name: "MkChartLegend",
  setup(__props, { expose: __expose }) {
    const chart = shallowRef();
    const type = shallowRef();
    const items = shallowRef([]);
    function update(_chart, _items) {
      chart.value = _chart, items.value = _items;
      if ("type" in _chart.config) type.value = _chart.config.type;
    }
    function onClick(item) {
      if (chart.value == null) return;
      if (type.value === "pie" || type.value === "doughnut") {
        // Pie and doughnut charts only have a single dataset and visibility is per item
        if (item.index != null) chart.value.toggleDataVisibility(item.index);
      } else {
        if (item.datasetIndex != null) chart.value.setDatasetVisibility(item.datasetIndex, !chart.value.isDatasetVisible(item.datasetIndex));
      }
      chart.value.update();
    }
    __expose({ update });
    return (_ctx, _cache) => {
      return _openBlock(), _createElementBlock(
        "div",
        { class: _normalizeClass(_ctx.$style.root) },
        [(_openBlock(true), _createElementBlock(
          _Fragment,
          null,
          _renderList(items.value, (item) => {
            return _openBlock(), _createElementBlock("button", {
              class: _normalizeClass(["_button item", { disabled: item.hidden }]),
              onClick: ($event) => onClick(item)
            }, [_createElementVNode(
              "span",
              {
                class: "box",
                style: _normalizeStyle({ background: type.value === "line" ? item.strokeStyle?.toString() : item.fillStyle?.toString() })
              },
              null,
              4
              /* STYLE */
            ), _createTextVNode(
              " " + _toDisplayString(item.text),
              1
              /* TEXT */
            )], 10, ["onClick"]);
          }),
          256
          /* UNKEYED_FRAGMENT */
        ))],
        2
        /* CLASS */
      );
    };
  }
};
