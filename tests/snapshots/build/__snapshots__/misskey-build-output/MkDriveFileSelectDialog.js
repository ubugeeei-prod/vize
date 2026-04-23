import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue";
import { ref, useTemplateRef } from "vue";
import MkDrive from "@/components/MkDrive.vue";
import MkModalWindow from "@/components/MkModalWindow.vue";
import { i18n } from "@/i18n.js";
export default {
  __name: "MkDriveFileSelectDialog",
  props: {
    initialFolder: {
      type: null,
      required: false
    },
    multiple: {
      type: Boolean,
      required: true
    }
  },
  emits: ["done", "closed"],
  setup(__props, { emit: __emit }) {
    const emit = __emit;
    const dialog = useTemplateRef("dialog");
    const selected = ref([]);
    function ok() {
      emit("done", selected.value);
      dialog.value?.close();
    }
    function cancel() {
      emit("done");
      dialog.value?.close();
    }
    function onChangeSelection(v) {
      selected.value = v;
    }
    return (_ctx, _cache) => {
      return _openBlock(), _createBlock(MkModalWindow, {
        ref_key: "dialog",
        ref: dialog,
        width: 800,
        height: 500,
        withOkButton: true,
        okButtonDisabled: selected.value.length === 0,
        onClick: _cache[0] || (_cache[0] = ($event) => cancel()),
        onClose: _cache[1] || (_cache[1] = ($event) => cancel()),
        onOk: _cache[2] || (_cache[2] = ($event) => ok()),
        onClosed: _cache[3] || (_cache[3] = ($event) => emit("closed"))
      }, {
        header: _withCtx(() => [
          _createTextVNode(
            _toDisplayString(__props.multiple ? _unref(i18n).ts.selectFiles : _unref(i18n).ts.selectFile),
            1
            /* TEXT */
          ),
          _createTextVNode(" "),
          selected.value.length > 0 ? (_openBlock(), _createElementBlock(
            "span",
            {
              key: 0,
              style: "margin-left: 8px; opacity: 0.5;"
            },
            "(" + _toDisplayString(selected.value.length) + ")",
            1
            /* TEXT */
          )) : _createCommentVNode("v-if", true)
        ]),
        default: _withCtx(() => [_createVNode(MkDrive, {
          multiple: __props.multiple,
          select: "file",
          initialFolder: __props.initialFolder,
          onChangeSelectedFiles: onChangeSelection
        }, null, 8, ["multiple", "initialFolder"])]),
        _: 1
      }, 8, [
        "width",
        "height",
        "withOkButton",
        "okButtonDisabled"
      ]);
    };
  }
};
