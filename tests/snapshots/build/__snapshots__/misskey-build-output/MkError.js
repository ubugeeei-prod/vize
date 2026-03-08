import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"

import MkButton from '@/components/MkButton.vue'
import { i18n } from '@/i18n.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkError',
  emits: ["retry"],
  setup(__props, { emit: __emit }) {

const emit = __emit

return (_ctx: any,_cache: any) => {
  const _component_MkResult = _resolveComponent("MkResult")

  return (_openBlock(), _createBlock(_component_MkResult, { type: "error" }, {
      default: _withCtx(() => [
        _createVNode(MkButton, {
          class: _normalizeClass(_ctx.$style.button),
          rounded: "",
          onClick: _cache[0] || (_cache[0] = () => emit('retry'))
        }, {
          default: _withCtx(() => [
            _createTextVNode(_toDisplayString(_unref(i18n).ts.retry), 1 /* TEXT */)
          ]),
          _: 1 /* STABLE */
        })
      ]),
      _: 1 /* STABLE */
    }))
}
}

})
