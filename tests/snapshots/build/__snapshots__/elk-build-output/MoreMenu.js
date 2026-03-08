import { useModel as _useModel } from 'vue'
import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementVNode as _createElementVNode, resolveComponent as _resolveComponent, normalizeClass as _normalizeClass, withCtx as _withCtx } from "vue"


export default /*@__PURE__*/_defineComponent({
  __name: 'MoreMenu',
  props: {
    "modelValue": {},
    "modelModifiers": {},
  },
  emits: ["update:modelValue"],
  setup(__props) {

const model = _useModel(__props, "modelValue")

return (_ctx: any,_cache: any) => {
  const _component_NavBottomMoreMenu = _resolveComponent("NavBottomMoreMenu")

  return (_openBlock(), _createBlock(_component_NavBottomMoreMenu, {
      flex: "",
      "flex-row": "",
      "items-center": "",
      "place-content-center": "",
      "h-full": "",
      "flex-1": "",
      "cursor-pointer": "",
      modelValue: model.value,
      "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event) => model.value = $event)
    }, {
      default: _withCtx(({ toggleVisible, show }) => [
        _createElementVNode("button", {
          flex: "",
          "items-center": "",
          "place-content-center": "",
          "h-full": "",
          "flex-1": "",
          class: _normalizeClass(["select-none", show ? '!text-primary' : '']),
          "aria-label": _ctx.$t('nav.more_menu'),
          onClick: _cache[1] || (_cache[1] = (...args) => (_ctx.toggleVisible && _ctx.toggleVisible(...args)))
        }, [
          _createElementVNode("span", {
            class: _normalizeClass(show ? 'i-ri:close-fill' : 'i-ri:more-fill')
          })
        ], 8 /* PROPS */, ["aria-label"])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["modelValue"]))
}
}

})
