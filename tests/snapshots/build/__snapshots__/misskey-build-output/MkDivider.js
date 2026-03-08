import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, normalizeStyle as _normalizeStyle } from "vue"


export default /*@__PURE__*/_defineComponent({
  __name: 'MkDivider',
  props: {
    marginTopBottom: { type: String, required: false },
    marginLeftRight: { type: String, required: false },
    borderStyle: { type: String, required: false },
    borderWidth: { type: String, required: false },
    borderColor: { type: String, required: false }
  },
  setup(__props: any) {


return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      class: "default",
      style: _normalizeStyle([
  		__props.marginTopBottom ? { marginTop: __props.marginTopBottom, marginBottom: __props.marginTopBottom } : {},
  		__props.marginLeftRight ? { marginLeft: __props.marginLeftRight, marginRight: __props.marginLeftRight } : {},
  		__props.borderStyle ? { borderStyle: __props.borderStyle } : {},
  		__props.borderWidth ? { borderWidth: __props.borderWidth } : {},
  		__props.borderColor ? { borderColor: __props.borderColor } : {},
  	])
    }, null, 4 /* STYLE */))
}
}

})
