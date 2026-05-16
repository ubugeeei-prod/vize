import { openBlock as _openBlock, createBlock as _createBlock, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx } from "vue";
import { ref } from "vue";
import MkModalWindow from "./MkModalWindow.vue";
export default {
  __name: "MkImgPreviewDialog",
  props: { file: {
    type: null,
    required: true
  } },
  emits: ["closed"],
  setup(__props, { emit: __emit }) {
    const emit = __emit;
    const modal = ref(null);
    function close() {
      modal.value?.close();
    }
    return (_ctx, _cache) => {
      return _openBlock(), _createBlock(MkModalWindow, {
        ref_key: "modal",
        ref: modal,
        width: 1800,
        height: 900,
        onClose: close,
        onEsc: close,
        onClick: close,
        onClosed: _cache[0] || (_cache[0] = ($event) => emit("closed"))
      }, {
        header: _withCtx(() => [_createTextVNode(
          _toDisplayString(__props.file.name),
          1
          /* TEXT */
        )]),
        default: _withCtx(() => [_createElementVNode(
          "div",
          { class: _normalizeClass(_ctx.$style.container) },
          [_createElementVNode("img", {
            src: __props.file.url,
            alt: __props.file.comment || __props.file.name,
            class: _normalizeClass(_ctx.$style.img)
          }, null, 10, ["src", "alt"])],
          2
          /* CLASS */
        )]),
        _: 1
      }, 8, ["width", "height"]);
    };
  }
};
