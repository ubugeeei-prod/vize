import { useModel as _useModel } from "vue";
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-trash" });
const _hoisted_2 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-arrow-up" });
const _hoisted_3 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-arrow-down" });
import MkFolder from "@/components/MkFolder.vue";
import MkButton from "@/components/MkButton.vue";
import MkImageEffectorFxForm from "@/components/MkImageEffectorFxForm.vue";
import { FXS } from "@/utility/image-effector/fxs.js";
export default {
  __name: "MkImageEffectorDialog.Layer",
  props: {
    "layer": { required: true },
    "layerModifiers": {}
  },
  emits: [
    "del",
    "swapUp",
    "swapDown",
    "update:layer"
  ],
  setup(__props, { emit: __emit }) {
    const emit = __emit;
    const layer = _useModel(__props, "layer");
    const fx = FXS[layer.value.fxId];
    if (fx == null) {
      throw new Error(`Unrecognized effect: ${layer.value.fxId}`);
    }
    return (_ctx, _cache) => {
      return _openBlock(), _createBlock(MkFolder, {
        defaultOpen: true,
        canPage: false
      }, {
        label: _withCtx(() => [_createTextVNode(
          _toDisplayString(_unref(fx).uiDefinition.name),
          1
          /* TEXT */
        )]),
        footer: _withCtx(() => [_createElementVNode("div", { class: "_buttons" }, [
          _createVNode(MkButton, {
            iconOnly: "",
            onClick: _cache[0] || (_cache[0] = ($event) => emit("del"))
          }, {
            default: _withCtx(() => [_hoisted_1]),
            _: 1
          }),
          _createVNode(MkButton, {
            iconOnly: "",
            onClick: _cache[1] || (_cache[1] = ($event) => emit("swapUp"))
          }, {
            default: _withCtx(() => [_hoisted_2]),
            _: 1
          }),
          _createVNode(MkButton, {
            iconOnly: "",
            onClick: _cache[2] || (_cache[2] = ($event) => emit("swapDown"))
          }, {
            default: _withCtx(() => [_hoisted_3]),
            _: 1
          })
        ])]),
        default: _withCtx(() => [_createVNode(MkImageEffectorFxForm, {
          paramDefs: _unref(fx).uiDefinition.params,
          modelValue: layer.value.params,
          "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event) => layer.value.params = $event)
        }, null, 8, ["paramDefs", "modelValue"])]),
        _: 1
      }, 8, ["defaultOpen", "canPage"]);
    };
  }
};
