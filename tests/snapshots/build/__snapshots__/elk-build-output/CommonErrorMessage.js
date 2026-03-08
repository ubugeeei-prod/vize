import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, renderSlot as _renderSlot, mergeProps as _mergeProps } from "vue"


export default /*@__PURE__*/_defineComponent({
  __name: 'CommonErrorMessage',
  props: {
    describedBy: { type: String, required: true }
  },
  setup(__props: any) {


return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", _mergeProps(_ctx.$attrs, {
      role: "alert",
      "aria-live": "polite",
      "aria-describedby": __props.describedBy,
      flex: "~ col",
      "gap-1": "",
      "text-sm": "",
      "pt-1": "",
      "ps-2": "",
      "pe-1": "",
      "pb-2": "",
      "text-red-600": "",
      "dark:text-red-400": "",
      border: "~ base rounded red-600 dark:red-400"
    }), [ _renderSlot(_ctx.$slots, "default") ], 16 /* FULL_PROPS */, ["aria-describedby"]))
}
}

})
