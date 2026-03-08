import { useModel as _useModel } from 'vue'
import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-trash" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-up" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-down" })
import type { ImageEffectorLayer } from '@/utility/image-effector/ImageEffector.js'
import MkFolder from '@/components/MkFolder.vue'
import MkButton from '@/components/MkButton.vue'
import MkImageEffectorFxForm from '@/components/MkImageEffectorFxForm.vue'
import { FXS } from '@/utility/image-effector/fxs.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkImageEffectorDialog.Layer',
  props: {
    "layer": { required: true },
    "layerModifiers": {},
  },
  emits: ["del", "swapUp", "swapDown", "update:layer"],
  setup(__props, { emit: __emit }) {

const emit = __emit
const layer = _useModel(__props, "layer")
const fx = FXS[layer.value.fxId];
if (fx == null) {
	throw new Error(`Unrecognized effect: ${layer.value.fxId}`);
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createBlock(MkFolder, {
      defaultOpen: true,
      canPage: false
    }, {
      label: _withCtx(() => [
        _createTextVNode(_toDisplayString(_unref(fx).uiDefinition.name), 1 /* TEXT */)
      ]),
      footer: _withCtx(() => [
        _createElementVNode("div", { class: "_buttons" }, [
          _createVNode(MkButton, {
            iconOnly: "",
            onClick: _cache[0] || (_cache[0] = ($event: any) => (emit('del')))
          }, {
            default: _withCtx(() => [
              _hoisted_1
            ]),
            _: 1 /* STABLE */
          }),
          _createVNode(MkButton, {
            iconOnly: "",
            onClick: _cache[1] || (_cache[1] = ($event: any) => (emit('swapUp')))
          }, {
            default: _withCtx(() => [
              _hoisted_2
            ]),
            _: 1 /* STABLE */
          }),
          _createVNode(MkButton, {
            iconOnly: "",
            onClick: _cache[2] || (_cache[2] = ($event: any) => (emit('swapDown')))
          }, {
            default: _withCtx(() => [
              _hoisted_3
            ]),
            _: 1 /* STABLE */
          })
        ])
      ]),
      default: _withCtx(() => [
        _createVNode(MkImageEffectorFxForm, {
          paramDefs: _unref(fx).uiDefinition.params,
          modelValue: layer.value.params,
          "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event: any) => ((layer.value.params) = $event))
        }, null, 8 /* PROPS */, ["paramDefs", "modelValue"])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["defaultOpen", "canPage"]))
}
}

})
