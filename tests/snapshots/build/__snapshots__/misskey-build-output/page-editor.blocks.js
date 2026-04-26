import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, resolveDynamicComponent as _resolveDynamicComponent, withCtx as _withCtx } from "vue";
import XSection from "./els/page-editor.el.section.vue";
import XText from "./els/page-editor.el.text.vue";
import XImage from "./els/page-editor.el.image.vue";
import XNote from "./els/page-editor.el.note.vue";
import MkDraggable from "@/components/MkDraggable.vue";
export default {
  __name: "page-editor.blocks",
  props: { modelValue: {
    type: null,
    required: true
  } },
  emits: ["update:modelValue", "content"],
  setup(__props, { emit: __emit }) {
    const emit = __emit;
    const props = __props;
    function getComponent(type) {
      switch (type) {
        case "section": return XSection;
        case "text": return XText;
        case "image": return XImage;
        case "note": return XNote;
        default: return XText;
      }
    }
    function updateItem(v) {
      const i = props.modelValue.findIndex((x) => x.id === v.id);
      const newValue = [
        ...props.modelValue.slice(0, i),
        v,
        ...props.modelValue.slice(i + 1)
      ];
      emit("update:modelValue", newValue);
    }
    function removeItem(v) {
      const i = props.modelValue.findIndex((x) => x.id === v.id);
      const newValue = [...props.modelValue.slice(0, i), ...props.modelValue.slice(i + 1)];
      emit("update:modelValue", newValue);
    }
    return (_ctx, _cache) => {
      return _openBlock(), _createBlock(MkDraggable, {
        modelValue: __props.modelValue,
        direction: "vertical",
        withGaps: "",
        canNest: "",
        group: "pageBlocks",
        "onUpdate:modelValue": _cache[0] || (_cache[0] = (v) => emit("update:modelValue", v))
      }, {
        default: _withCtx(({ item }) => [_createElementVNode("div", null, [_createVNode(_resolveDynamicComponent(getComponent(item.type)), {
          modelValue: item,
          "onUpdate:modelValue": updateItem,
          onRemove: () => removeItem(item)
        }, null, 8, ["modelValue", "onRemove"])])]),
        _: 1
      }, 8, ["modelValue"]);
    };
  }
};
