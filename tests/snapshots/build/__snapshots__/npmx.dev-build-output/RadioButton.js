import { useModel as _useModel } from 'vue'
import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, renderSlot as _renderSlot, unref as _unref } from "vue"


export default /*@__PURE__*/_defineComponent({
  __name: 'RadioButton',
  props: {
    disabled: { type: Boolean, required: false },
    type: { type: null, required: false },
    checked: { type: null, required: false },
    value: { type: String, required: true },
    "modelValue": {}
  },
  emits: ["update:modelValue"],
  setup(__props: any) {

const props = __props
const model = _useModel(__props, "modelValue")
const uid = useId()
const internalId = `${model.value}-${uid}`
const checked = computed(() => model.value === props.value)
/** Todo: This shouldn't be necessary, but using v-model on `input type=radio` doesn't work as expected in Vue */
const onChange = () => {
  model.value = props.value
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", null, [ _createElementVNode("input", {
        type: "radio",
        id: _unref(internalId),
        value: props.value,
        checked: checked.value,
        disabled: props.disabled ? true : undefined,
        onChange: onChange,
        class: "peer sr-only"
      }, null, 40 /* PROPS, NEED_HYDRATION */, ["id", "value", "checked", "disabled"]), _createElementVNode("label", {
        class: "bg-bg-muted text-fg-muted border-border hover:(text-fg border-border-hover) inline-flex items-center px-2 py-0.5 text-xs font-mono border rounded transition-colors duration-200 peer-focus-visible:(outline-2 outline-accent/70 outline-offset-2) border-none peer-checked:(bg-fg text-bg border-fg hover:(text-text-bg/50)) peer-disabled:(opacity-50 pointer-events-none)",
        htmlFor: _unref(internalId)
      }, [ _renderSlot(_ctx.$slots, "default") ], 8 /* PROPS */, ["htmlFor"]) ]))
}
}

})
