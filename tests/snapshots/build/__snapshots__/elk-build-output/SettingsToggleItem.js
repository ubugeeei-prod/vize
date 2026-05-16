import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, renderSlot as _renderSlot, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass } from "vue"


export default /*@__PURE__*/_defineComponent({
  __name: 'SettingsToggleItem',
  props: {
    icon: { type: String, required: false },
    text: { type: String, required: false },
    checked: { type: Boolean, required: true },
    disabled: { type: Boolean, required: false, default: false }
  },
  setup(__props: any) {


return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("button", {
      "exact-active-class": "text-primary",
      block: "",
      "w-full": "",
      group: "",
      "focus:outline-none": "",
      "text-start": "",
      role: "checkbox",
      "aria-checked": __props.checked,
      disabled: __props.disabled,
      class: _normalizeClass(__props.disabled ? 'opacity-50 cursor-not-allowed' : '')
    }, [ _createElementVNode("span", {
        "w-full": "",
        flex: "",
        px5: "",
        py3: "",
        "md:gap2": "",
        gap4: "",
        "items-center": "",
        "transition-250": "",
        class: _normalizeClass(__props.disabled ? '' : 'group-hover:bg-active'),
        "group-focus-visible:ring": "2 current"
      }, [ _createElementVNode("span", {
          "flex-1": "",
          flex: "",
          "items-center": "",
          "md:gap2": "",
          gap4: ""
        }, [ (__props.icon) ? (_openBlock(), _createElementBlock("span", {
              key: 0,
              flex: "",
              "items-center": "",
              "justify-center": "",
              "flex-shrink-0": "",
              class: _normalizeClass(_ctx.$slots.description ? 'w-12 h-12' : '')
            }, [ _renderSlot(_ctx.$slots, "icon", {}, () => [ (__props.icon) ? (_openBlock(), _createElementBlock("span", {
                    key: 0,
                    class: _normalizeClass(__props.icon),
                    "md:text-size-inherit": "",
                    "text-xl": ""
                  })) : _createCommentVNode("v-if", true) ]) ])) : _createCommentVNode("v-if", true), _createElementVNode("span", { "space-y-1": "" }, [ _createElementVNode("span", {
              class: _normalizeClass(__props.checked ? 'text-base' : 'text-secondary')
            }, [ _renderSlot(_ctx.$slots, "default", {}, () => [ _createElementVNode("span", null, _toDisplayString(__props.text), 1 /* TEXT */) ]) ], 2 /* CLASS */), (_ctx.$slots.description) ? (_openBlock(), _createElementBlock("span", {
                key: 0,
                block: "",
                "text-sm": "",
                "text-secondary": ""
              }, [ _renderSlot(_ctx.$slots, "description") ])) : _createCommentVNode("v-if", true) ]) ]), _createElementVNode("span", {
          "text-lg": "",
          class: _normalizeClass(__props.checked ? 'i-ri-checkbox-line text-primary' : 'i-ri-checkbox-blank-line text-secondary')
        }, null, 2 /* CLASS */) ], 2 /* CLASS */) ], 10 /* CLASS, PROPS */, ["aria-checked", "disabled"]))
}
}

})
