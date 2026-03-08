import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, resolveDirective as _resolveDirective, withDirectives as _withDirectives, renderSlot as _renderSlot, normalizeClass as _normalizeClass } from "vue"


export default /*@__PURE__*/_defineComponent({
  __name: 'MkFeatureBanner',
  props: {
    icon: { type: String, required: true },
    color: { type: String, required: true }
  },
  setup(__props: any) {


return (_ctx: any,_cache: any) => {
  const _directive_panel = _resolveDirective("panel")

  return _withDirectives((_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(_ctx.$style.root)
    }, [ _createElementVNode("img", {
        class: _normalizeClass(_ctx.$style.img),
        src: __props.icon
      }, null, 8 /* PROPS */, ["src"]), _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.text)
      }, [ _renderSlot(_ctx.$slots, "default") ]) ])), [ [_directive_panel] ])
}
}

})
