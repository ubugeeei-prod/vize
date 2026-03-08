import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"

import * as Misskey from 'misskey-js'
import MkDrive from '@/components/MkDrive.vue'
import MkWindow from '@/components/MkWindow.vue'
import { i18n } from '@/i18n.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkDriveWindow',
  props: {
    initialFolder: { type: null, required: false }
  },
  emits: ["closed"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createBlock(MkWindow, {
      ref: "window",
      initialWidth: 800,
      initialHeight: 500,
      canResize: true,
      onClosed: _cache[0] || (_cache[0] = ($event: any) => (emit('closed')))
    }, {
      header: _withCtx(() => [
        _createTextVNode(_toDisplayString(_unref(i18n).ts.drive), 1 /* TEXT */)
      ]),
      default: _withCtx(() => [
        _createVNode(MkDrive, { initialFolder: __props.initialFolder }, null, 8 /* PROPS */, ["initialFolder"])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["initialWidth", "initialHeight", "canResize"]))
}
}

})
