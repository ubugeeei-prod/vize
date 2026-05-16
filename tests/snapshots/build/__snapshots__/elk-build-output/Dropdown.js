import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, resolveComponent as _resolveComponent, renderSlot as _renderSlot, mergeProps as _mergeProps, withCtx as _withCtx, unref as _unref } from "vue"

import { InjectionKeyDropdownContext } from '~/constants/symbols'

export default /*@__PURE__*/_defineComponent({
  __name: 'Dropdown',
  props: {
    placement: { type: String, required: false },
    autoBoundaryMaxSize: { type: Boolean, required: false }
  },
  setup(__props: any, { expose: __expose }) {

const dropdown = ref<any>()
const colorMode = useColorMode()
function hide() {
  return dropdown.value.hide()
}
provide(InjectionKeyDropdownContext, {
  hide,
})
__expose({
  hide,
})

return (_ctx: any,_cache: any) => {
  const _component_VDropdown = _resolveComponent("VDropdown")

  return (_openBlock(), _createBlock(_component_VDropdown, _mergeProps(_ctx.$attrs, {
      ref_key: "dropdown", ref: dropdown,
      class: _unref(colorMode).value,
      placement: __props.placement || 'auto',
      "auto-boundary-max-size": __props.autoBoundaryMaxSize
    }), {
      popper: _withCtx((scope) => [
        _renderSlot(_ctx.$slots, "popper", _mergeProps(scope, {
          name: "popper"
        }))
      ]),
      default: _withCtx(() => [
        _renderSlot(_ctx.$slots, "default")
      ]),
      _: 1 /* STABLE */
    }, 16 /* FULL_PROPS */, ["placement", "auto-boundary-max-size"]))
}
}

})
