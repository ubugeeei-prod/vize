import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createCommentVNode as _createCommentVNode, renderList as _renderList, normalizeClass as _normalizeClass } from "vue"

import { GridEventEmitter } from '@/components/grid/grid.js'
import MkHeaderCell from '@/components/grid/MkHeaderCell.vue'
import MkNumberCell from '@/components/grid/MkNumberCell.vue'
import type { Size } from '@/components/grid/grid.js'
import type { GridColumn } from '@/components/grid/column.js'
import type { GridRowSetting } from '@/components/grid/row.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkHeaderRow',
  props: {
    columns: { type: Array, required: true },
    gridSetting: { type: null, required: true },
    bus: { type: null, required: true }
  },
  emits: ["operation:beginWidthChange", "operation:endWidthChange", "operation:widthLargest", "operation:selectionColumn", "change:width", "change:contentSize"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(["mk_grid_tr", _ctx.$style.root]),
      "data-grid-row": -1
    }, [ (__props.gridSetting.showNumber) ? (_openBlock(), _createBlock(MkNumberCell, {
          key: 0,
          content: "#",
          top: true
        }, null, 8 /* PROPS */, ["top"])) : _createCommentVNode("v-if", true), (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(__props.columns, (column) => {
        return (_openBlock(), _createBlock(MkHeaderCell, {
          key: column.index,
          column: column,
          bus: __props.bus,
          "onOperation:beginWidthChange": _cache[0] || (_cache[0] = (sender) => emit("operation:beginWidthChange", sender)),
          "onOperation:endWidthChange": _cache[1] || (_cache[1] = (sender) => emit("operation:endWidthChange", sender)),
          "onOperation:widthLargest": _cache[2] || (_cache[2] = (sender) => emit("operation:widthLargest", sender)),
          "onChange:width": _cache[3] || (_cache[3] = (sender, width) => emit("change:width", sender, width)),
          "onChange:contentSize": _cache[4] || (_cache[4] = (sender, newSize) => emit("change:contentSize", sender, newSize))
        }, null, 8 /* PROPS */, ["column", "bus"]))
      }), 128 /* KEYED_FRAGMENT */)) ], 8 /* PROPS */, ["data-grid-row"]))
}
}

})
