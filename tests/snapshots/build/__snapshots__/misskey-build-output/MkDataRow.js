import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createCommentVNode as _createCommentVNode, renderList as _renderList, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle } from "vue"

import { GridEventEmitter } from '@/components/grid/grid.js'
import MkDataCell from '@/components/grid/MkDataCell.vue'
import MkNumberCell from '@/components/grid/MkNumberCell.vue'
import type { Size } from '@/components/grid/grid.js'
import type { CellValue, GridCell } from '@/components/grid/cell.js'
import type { GridRow, GridRowSetting } from '@/components/grid/row.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkDataRow',
  props: {
    row: { type: null, required: true },
    cells: { type: Array, required: true },
    setting: { type: null, required: true },
    bus: { type: null, required: true }
  },
  emits: ["operation:beginEdit", "operation:endEdit", "change:value", "change:contentSize"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(["mk_grid_tr", [
  		_ctx.$style.row,
  		__props.row.ranged ? _ctx.$style.ranged : {},
  		...(__props.row.additionalStyles ?? []).map(it => it.className ?? {}),
  	]]),
      style: _normalizeStyle([
  		...(__props.row.additionalStyles ?? []).map(it => it.style ?? {}),
  	]),
      "data-grid-row": __props.row.index
    }, [ (__props.setting.showNumber) ? (_openBlock(), _createBlock(MkNumberCell, {
          key: 0,
          content: (__props.row.index + 1).toString(),
          row: __props.row
        }, null, 8 /* PROPS */, ["content", "row"])) : _createCommentVNode("v-if", true), (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(__props.cells, (cell) => {
        return (_openBlock(), _createBlock(MkDataCell, {
          key: cell.address.col,
          vIf: cell.column.setting.type !== 'hidden',
          cell: cell,
          rowSetting: __props.setting,
          bus: __props.bus,
          "onOperation:beginEdit": _cache[0] || (_cache[0] = (sender) => emit("operation:beginEdit", sender)),
          "onOperation:endEdit": _cache[1] || (_cache[1] = (sender) => emit("operation:endEdit", sender)),
          "onChange:value": _cache[2] || (_cache[2] = (sender, newValue) => emit("change:value", sender, newValue)),
          "onChange:contentSize": _cache[3] || (_cache[3] = (sender, newSize) => emit("change:contentSize", sender, newSize))
        }, null, 8 /* PROPS */, ["vIf", "cell", "rowSetting", "bus"]))
      }), 128 /* KEYED_FRAGMENT */)) ], 14 /* CLASS, STYLE, PROPS */, ["data-grid-row"]))
}
}

})
