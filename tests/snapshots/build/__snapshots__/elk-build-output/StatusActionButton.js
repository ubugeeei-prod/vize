import { useSlots as _useSlots } from 'vue'
import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, resolveDynamicComponent as _resolveDynamicComponent, renderSlot as _renderSlot, toDisplayString as _toDisplayString, mergeProps as _mergeProps, normalizeClass as _normalizeClass, withCtx as _withCtx } from "vue"


export default /*@__PURE__*/Object.assign({
  inheritAttrs: false,
}, {
  __name: 'StatusActionButton',
  props: {
    text: { type: [String, Number], required: false },
    content: { type: String, required: true },
    color: { type: String, required: true },
    icon: { type: String, required: true },
    activeIcon: { type: String, required: false },
    inactiveIcon: { type: String, required: false },
    hover: { type: String, required: true },
    elkGroupHover: { type: String, required: true },
    active: { type: Boolean, required: false },
    disabled: { type: Boolean, required: false },
    as: { type: String, required: false, default: 'button' },
    command: { type: Boolean, required: false }
  },
  setup(__props: any) {

const el = ref<HTMLDivElement>()
useCommand({
  scope: 'Actions',
  order: -2,
  visible: () => __props.command && !__props.disabled,
  name: () => __props.content,
  icon: () => __props.icon,
  onActivate() {
    if (!checkLogin())
      return
    const clickEvent = new MouseEvent('click', {
      view: window,
      bubbles: true,
      cancelable: true,
    })
    el.value?.dispatchEvent(clickEvent)
  },
})

return (_ctx: any,_cache: any) => {
  const _component_CommonTooltip = _resolveComponent("CommonTooltip")
  const _component_CommonAnimateNumber = _resolveComponent("CommonAnimateNumber")

  return (_openBlock(), _createBlock(_resolveDynamicComponent(__props.as), _mergeProps(_ctx.$attrs, {
      ref_key: "el", ref: el,
      "w-fit": "",
      flex: "",
      "gap-1": "",
      "items-center": "",
      "transition-all": "",
      "select-none": "",
      rounded: "",
      group: "",
      hover:  !__props.disabled ? __props.hover : undefined,
      "focus:outline-none": "",
      "focus-visible": __props.hover,
      class: __props.active ? __props.color : (__props.disabled ? 'op25 cursor-not-allowed' : 'text-secondary'),
      "aria-label": __props.content,
      disabled: __props.disabled,
      "aria-disabled": __props.disabled
    }), {
      default: _withCtx(() => [
        _createVNode(_component_CommonTooltip, {
          placement: "bottom",
          content: __props.content
        }, {
          default: _withCtx(() => [
            _createElementVNode("div", _mergeProps(__props.disabled ? {} : {
            'elk-group-hover': __props.elkGroupHover,
            'group-focus-visible': __props.elkGroupHover,
            'group-focus-visible:ring': '2 current',
          }, {
              "rounded-full": "",
              p2: ""
            }), [
              _createElementVNode("div", {
                class: _normalizeClass(__props.active && __props.activeIcon ? __props.activeIcon : (__props.disabled && __props.inactiveIcon ? __props.inactiveIcon : __props.icon))
              }, null, 2 /* CLASS */)
            ], 16 /* FULL_PROPS */)
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["content"]),
        (__props.text !== undefined || _ctx.$slots.text)
          ? (_openBlock(), _createBlock(_component_CommonAnimateNumber, {
            key: 0,
            increased: __props.active,
            "text-sm": ""
          }, {
            next: _withCtx(() => [
              _createElementVNode("span", {
                class: _normalizeClass([__props.color])
              }, [
                _renderSlot(_ctx.$slots, "text", {}, () => [
                  _toDisplayString(__props.text)
                ])
              ], 2 /* CLASS */)
            ]),
            default: _withCtx(() => [
              _createElementVNode("span", { "text-secondary-light": "" }, [
                _renderSlot(_ctx.$slots, "text", {}, () => [
                  _toDisplayString(__props.text)
                ])
              ])
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["increased"]))
          : _createCommentVNode("v-if", true)
      ]),
      _: 1 /* STABLE */
    }, 16 /* FULL_PROPS */, ["hover", "focus-visible", "aria-label", "disabled", "aria-disabled"]))
}
}

})
