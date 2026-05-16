import { useModel as _useModel } from 'vue'
import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, withDirectives as _withDirectives, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, vModelText as _vModelText } from "vue"


export default /*@__PURE__*/_defineComponent({
  __name: 'DurationPicker',
  props: {
    "modelValue": {},
    "modelModifiers": {},
    "isValid": {},
    "isValidModifiers": {},
  },
  emits: ["update:modelValue", "update:isValid"],
  setup(__props) {

const model = _useModel(__props, "modelValue")
const isValid = _useModel(__props, "isValid")
const days = ref<number | ''>(0)
const hours = ref<number | ''>(1)
const minutes = ref<number | ''>(0)
watchEffect(() => {
  if (days.value === '' || hours.value === '' || minutes.value === '') {
    isValid.value = false
    return
  }
  const duration
    = days.value * 24 * 60 * 60
      + hours.value * 60 * 60
      + minutes.value * 60
  if (duration <= 0) {
    isValid.value = false
    return
  }
  isValid.value = true
  model.value = duration
})

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      flex: "",
      "flex-grow-0": "",
      "gap-2": ""
    }, [ _createElementVNode("label", {
        flex: "",
        "items-center": "",
        "gap-2": ""
      }, [ _withDirectives(_createElementVNode("input", {
          "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((days).value = $event)),
          type: "number",
          min: "0",
          max: "1999",
          "input-base": "",
          class: _normalizeClass(!isValid.value ? 'input-error' : null)
        }, null, 2 /* CLASS */), [ [_vModelText, days.value] ]), _createTextVNode("\n      " + _toDisplayString(_ctx.$t('confirm.mute_account.days', days.value === '' ? 0 : days.value)), 1 /* TEXT */) ]), _createElementVNode("label", {
        flex: "",
        "items-center": "",
        "gap-2": ""
      }, [ _withDirectives(_createElementVNode("input", {
          "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((hours).value = $event)),
          type: "number",
          min: "0",
          max: "24",
          "input-base": "",
          class: _normalizeClass(!isValid.value ? 'input-error' : null)
        }, null, 2 /* CLASS */), [ [_vModelText, hours.value] ]), _createTextVNode("\n      " + _toDisplayString(_ctx.$t('confirm.mute_account.hours', hours.value === '' ? 0 : hours.value)), 1 /* TEXT */) ]), _createElementVNode("label", {
        flex: "",
        "items-center": "",
        "gap-2": ""
      }, [ _withDirectives(_createElementVNode("input", {
          "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((minutes).value = $event)),
          type: "number",
          min: "0",
          max: "59",
          step: "5",
          "input-base": "",
          class: _normalizeClass(!isValid.value ? 'input-error' : null)
        }, null, 2 /* CLASS */), [ [_vModelText, minutes.value] ]), _createTextVNode("\n      " + _toDisplayString(_ctx.$t('confirm.mute_account.minute', minutes.value === '' ? 0 : minutes.value)), 1 /* TEXT */) ]) ]))
}
}

})
