import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveDynamicComponent as _resolveDynamicComponent, renderSlot as _renderSlot, toDisplayString as _toDisplayString, mergeProps as _mergeProps, normalizeClass as _normalizeClass, withCtx as _withCtx } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("div", { "flex-auto": "true" })

export default /*@__PURE__*/_defineComponent({
  __name: 'DropdownItem',
  props: {
    is: { type: String, required: false, default: 'div' },
    text: { type: String, required: false },
    description: { type: String, required: false },
    icon: { type: String, required: false },
    checked: { type: Boolean, required: false },
    command: { type: Boolean, required: false }
  },
  emits: ["click"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const type = computed(() => __props.is === 'button' ? 'button' : null)
const { hide } = useDropdownContext() || {}
const el = ref<HTMLDivElement>()
function handleClick(evt: MouseEvent) {
  hide?.()
  emit('click', evt)
}
useCommand({
  scope: 'Actions',
  order: -1,
  visible: () => __props.command && __props.text,
  name: () => __props.text!,
  icon: () => __props.icon ?? 'i-ri:question-line',
  description: () => __props.description,
  onActivate() {
    const clickEvent = new MouseEvent('click', {
      view: window,
      bubbles: true,
      cancelable: true,
    })
    el.value?.dispatchEvent(clickEvent)
  },
})

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createBlock(_resolveDynamicComponent(__props.is), _mergeProps(_ctx.$attrs, {
      ref_key: "el", ref: el,
      type: type.value,
      "w-full": "",
      flex: "",
      "gap-3": "",
      "items-center": "",
      "cursor-pointer": "",
      px4: "",
      py3: "",
      "select-none": "",
      "hover-bg-active": "",
      "aria-label": __props.text,
      onClick: handleClick
    }), {
      default: _withCtx(() => [
        (__props.icon)
          ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: _normalizeClass(__props.icon)
          }))
          : _createCommentVNode("v-if", true),
        _createElementVNode("div", { flex: "~ col" }, [
          _createElementVNode("div", { "text-15px": "" }, [
            _renderSlot(_ctx.$slots, "default", {}, () => [
              _toDisplayString(__props.text)
            ])
          ]),
          _createElementVNode("div", {
            "text-3": "",
            "text-secondary": ""
          }, [
            _renderSlot(_ctx.$slots, "description", {}, () => [
              (__props.description)
                ? (_openBlock(), _createElementBlock("p", { key: 0 }, _toDisplayString(__props.description), 1 /* TEXT */))
                : _createCommentVNode("v-if", true)
            ])
          ])
        ]),
        _hoisted_1,
        (__props.checked)
          ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            "i-ri:check-line": ""
          }))
          : _createCommentVNode("v-if", true),
        _renderSlot(_ctx.$slots, "actions")
      ]),
      _: 1 /* STABLE */
    }, 16 /* FULL_PROPS */, ["type", "aria-label"]))
}
}

})
