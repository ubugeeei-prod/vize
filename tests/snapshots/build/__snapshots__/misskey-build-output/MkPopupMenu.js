import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, normalizeClass as _normalizeClass, withCtx as _withCtx } from "vue";
import { ref, useTemplateRef } from "vue";
import MkModal from "./MkModal.vue";
import MkMenu from "./MkMenu.vue";
export default {
  __name: "MkPopupMenu",
  props: {
    items: {
      type: Array,
      required: true
    },
    align: {
      type: String,
      required: false
    },
    width: {
      type: Number,
      required: false
    },
    anchorElement: {
      type: null,
      required: false
    },
    returnFocusTo: {
      type: null,
      required: false
    }
  },
  emits: ["closed", "closing"],
  setup(__props, { emit: __emit }) {
    const emit = __emit;
    const modal = useTemplateRef("modal");
    const manualShowing = ref(true);
    const hiding = ref(false);
    function click() {
      close();
    }
    function onModalClose() {
      emit("closing");
    }
    function onMenuClose() {
      close();
      if (hiding.value) {
        // hidingであればclosedを発火
        emit("closed");
      }
    }
    function onModalClosed() {
      if (!hiding.value) {
        // hidingでなければclosedを発火
        emit("closed");
      }
    }
    function hide() {
      manualShowing.value = false;
      hiding.value = true;
      // closeは呼ぶ必要がある
      modal.value?.close();
    }
    function close() {
      manualShowing.value = false;
      // closeは呼ぶ必要がある
      modal.value?.close();
    }
    return (_ctx, _cache) => {
      return _openBlock(), _createBlock(MkModal, {
        ref_key: "modal",
        ref: modal,
        manualShowing: manualShowing.value,
        zPriority: "high",
        anchorElement: __props.anchorElement,
        transparentBg: true,
        returnFocusTo: __props.returnFocusTo,
        onClick: click,
        onClose: onModalClose,
        onClosed: onModalClosed
      }, {
        default: _withCtx(({ type, maxHeight }) => [_createVNode(MkMenu, {
          items: __props.items,
          align: __props.align,
          width: __props.width,
          "max-height": maxHeight,
          asDrawer: type === "drawer",
          returnFocusTo: __props.returnFocusTo,
          class: _normalizeClass({ [_ctx.$style.drawer]: type === "drawer" }),
          onClose: onMenuClose,
          onHide: hide
        }, null, 10, [
          "items",
          "align",
          "width",
          "max-height",
          "asDrawer",
          "returnFocusTo"
        ])]),
        _: 1
      }, 8, [
        "manualShowing",
        "zPriority",
        "anchorElement",
        "transparentBg",
        "returnFocusTo"
      ]);
    };
  }
};
