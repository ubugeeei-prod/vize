import { useModel as _useModel } from 'vue'
import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withKeys as _withKeys } from "vue"


export default /*@__PURE__*/_defineComponent({
  __name: 'CommonTabs',
  props: {
    options: { type: Array, required: true },
    command: { type: Boolean, required: false },
    "modelValue": { required: true }
  },
  emits: ["update:modelValue"],
  setup(__props: any) {

const modelValue = _useModel(__props, "modelValue")
const tabs = computed(() => {
  return __props.options.map((option) => {
    if (typeof option === 'string')
      return { name: option, display: option }
    else
      return option
  })
})
function toValidName(option: string) {
  return option.toLowerCase().replace(/[^a-z0-9]/gi, '-')
}
useCommands(() => __props.command
  ? tabs.value.map(tab => ({
      scope: 'Tabs',
      name: tab.display,
      icon: tab.icon ?? 'i-ri:file-list-2-line',
      onActivate: () => modelValue.value = tab.name,
    }))
  : [])

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      flex: "",
      "w-full": "",
      "items-center": "",
      "lg:text-lg": ""
    }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(tabs.value, (option) => {
        return (_openBlock(), _createElementBlock(_Fragment, { key: option }, [
          _createElementVNode("input", {
            id: `tab-${toValidName(option.name)}`,
            checked: modelValue.value === option.name,
            value: option,
            type: "radio",
            name: "tabs",
            display: "none",
            onChange: _cache[0] || (_cache[0] = ($event: any) => (modelValue.value = option.name))
          }, null, 40 /* PROPS, NEED_HYDRATION */, ["id", "checked", "value"]),
          _createElementVNode("label", {
            flex: "",
            "flex-auto": "",
            "cursor-pointer": "",
            px3: "",
            m1: "",
            rounded: "",
            "transition-all": "",
            for: `tab-${toValidName(option.name)}`,
            tabindex: "0",
            "hover:bg-active": "",
            "transition-100": "",
            onKeypress: _cache[1] || (_cache[1] = _withKeys(($event: any) => (modelValue.value = option.name), ["enter"]))
          }, [
            _createElementVNode("span", {
              mxa: "",
              px4: "",
              py3: "",
              "text-center": "",
              "border-b-3": "",
              class: _normalizeClass(modelValue.value === option.name ? 'font-bold border-primary' : 'op50 hover:op50 border-transparent')
            }, _toDisplayString(option.display), 3 /* TEXT, CLASS */)
          ], 40 /* PROPS, NEED_HYDRATION */, ["for"])
        ], 64 /* STABLE_FRAGMENT */))
      }), 128 /* KEYED_FRAGMENT */)) ]))
}
}

})
