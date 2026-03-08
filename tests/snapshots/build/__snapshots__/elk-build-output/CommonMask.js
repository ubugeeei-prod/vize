import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, normalizeStyle as _normalizeStyle } from "vue"


export default /*@__PURE__*/_defineComponent({
  __name: 'CommonMask',
  props: {
    zIndex: { type: Number, required: false, default: 100 },
    background: { type: String, required: false, default: 'transparent' }
  },
  setup(__props: any) {


return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      fixed: "",
      "top-0": "",
      "bottom-0": "",
      "left-0": "",
      "right-0": "",
      style: _normalizeStyle({ background: __props.background, zIndex: __props.zIndex })
    }, null, 4 /* STYLE */))
}
}

})
