import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue";
import { useTemplateRef } from "vue";
import MkModal from "@/components/MkModal.vue";
import MkEmojiPicker from "@/components/MkEmojiPicker.vue";
import { prefer } from "@/preferences.js";
export default {
  __name: "MkEmojiPickerDialog",
  props: {
    manualShowing: {
      type: [Boolean, null],
      required: false,
      default: null
    },
    anchorElement: {
      type: null,
      required: false
    },
    showPinned: {
      type: Boolean,
      required: false,
      default: true
    },
    pinnedEmojis: {
      type: Array,
      required: false,
      default: undefined
    },
    asReactionPicker: {
      type: Boolean,
      required: false,
      default: false
    },
    targetNote: {
      type: null,
      required: false
    },
    choseAndClose: {
      type: Boolean,
      required: false,
      default: true
    }
  },
  emits: [
    "done",
    "close",
    "closed"
  ],
  setup(__props, { emit: __emit }) {
    const emit = __emit;
    const props = __props;
    const modal = useTemplateRef("modal");
    const picker = useTemplateRef("picker");
    function chosen(emoji) {
      emit("done", emoji);
      if (props.choseAndClose) {
        modal.value?.close();
      }
    }
    function opening() {
      picker.value?.reset();
      picker.value?.focus();
      // 何故かちょっと待たないとフォーカスされない
      window.setTimeout(() => {
        picker.value?.focus();
      }, 10);
    }
    return (_ctx, _cache) => {
      return _openBlock(), _createBlock(MkModal, {
        ref_key: "modal",
        ref: modal,
        zPriority: "middle",
        preferType: _unref(prefer).s.emojiPickerStyle,
        hasInteractionWithOtherFocusTrappedEls: true,
        transparentBg: true,
        manualShowing: __props.manualShowing,
        anchorElement: __props.anchorElement,
        onClick: _cache[0] || (_cache[0] = ($event) => _unref(modal)?.close()),
        onEsc: _cache[1] || (_cache[1] = ($event) => _unref(modal)?.close()),
        onOpening: opening,
        onClose: _cache[2] || (_cache[2] = ($event) => emit("close")),
        onClosed: _cache[3] || (_cache[3] = ($event) => emit("closed"))
      }, {
        default: _withCtx(({ type, maxHeight }) => [_createVNode(MkEmojiPicker, {
          ref_key: "picker",
          ref: picker,
          class: _normalizeClass(["_popup _shadow", { [_ctx.$style.drawer]: type === "drawer" }]),
          showPinned: __props.showPinned,
          pinnedEmojis: __props.pinnedEmojis,
          asReactionPicker: __props.asReactionPicker,
          targetNote: __props.targetNote,
          asDrawer: type === "drawer",
          "max-height": maxHeight,
          onChosen: chosen,
          onEsc: ($event) => _unref(modal)?.close()
        }, null, 10, [
          "showPinned",
          "pinnedEmojis",
          "asReactionPicker",
          "targetNote",
          "asDrawer",
          "max-height",
          "onEsc"
        ])]),
        _: 1
      }, 8, [
        "zPriority",
        "preferType",
        "hasInteractionWithOtherFocusTrappedEls",
        "transparentBg",
        "manualShowing",
        "anchorElement"
      ]);
    };
  }
};
