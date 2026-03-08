import { useModel as _useModel } from 'vue'
import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, renderSlot as _renderSlot } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("div", { "i-ri:close-line": "true" })

export default /*@__PURE__*/_defineComponent({
  __name: 'CommonAlert',
  props: {
    "modelValue": {},
    "modelModifiers": {},
  },
  emits: ["close", "update:modelValue"],
  setup(__props, { emit: __emit }) {

const emit = __emit
const visible = _useModel(__props, "modelValue")
function close() {
  emit('close')
  visible.value = false
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      flex: "~ gap-2",
      "justify-between": "",
      "items-center": "",
      border: "b base",
      "text-sm": "",
      "text-secondary": "",
      px4: "",
      py2: "",
      "sm:py4": ""
    }, [ _createElementVNode("div", null, [ _renderSlot(_ctx.$slots, "default") ]), _createElementVNode("button", {
        "text-xl": "",
        "hover:text-primary": "",
        "bg-hover-overflow": "",
        w: "1.2em",
        h: "1.2em",
        onClick: _cache[0] || (_cache[0] = ($event: any) => (close()))
      }, [ _hoisted_1 ]) ]))
}
}

})
