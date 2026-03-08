import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, renderSlot as _renderSlot, toDisplayString as _toDisplayString } from "vue"

const _hoisted_1 = /*#__PURE__*/ _createElementVNode("div", { "i-ri:forbid-line": "true", "text-10": "true", mt10: "true", mb2: "true" })

export function render(_ctx, _cache) {
  return (_openBlock(), _createElementBlock("div", {
    flex: "~ col",
    "items-center": ""
  }, [
    _hoisted_1,
    _createElementVNode("div", { "text-lg": "" }, [
      _renderSlot(_ctx.$slots, "default", {}, () => [
        _toDisplayString(_ctx.$t('common.not_found'))
      ])
    ])
  ]))
}
