import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue";
import { useTemplateRef } from "vue";
import MkModalWindow from "@/components/MkModalWindow.vue";
import XAntennaEditor from "@/components/MkAntennaEditor.vue";
import { i18n } from "@/i18n.js";
export default {
  __name: "MkAntennaEditorDialog",
  props: { antenna: {
    type: null,
    required: false
  } },
  emits: [
    "created",
    "updated",
    "deleted",
    "closed"
  ],
  setup(__props, { emit: __emit }) {
    const emit = __emit;
    const dialog = useTemplateRef("dialog");
    function onAntennaCreated(newAntenna) {
      emit("created", newAntenna);
      dialog.value?.close();
    }
    function onAntennaUpdated(editedAntenna) {
      emit("updated", editedAntenna);
      dialog.value?.close();
    }
    function onAntennaDeleted() {
      emit("deleted");
      dialog.value?.close();
    }
    function close() {
      dialog.value?.close();
    }
    return (_ctx, _cache) => {
      return _openBlock(), _createBlock(MkModalWindow, {
        ref_key: "dialog",
        ref: dialog,
        withOkButton: false,
        width: 500,
        height: 550,
        onClose: _cache[0] || (_cache[0] = ($event) => close()),
        onClosed: _cache[1] || (_cache[1] = ($event) => emit("closed"))
      }, {
        header: _withCtx(() => [_createTextVNode(
          _toDisplayString(__props.antenna == null ? _unref(i18n).ts.createAntenna : _unref(i18n).ts.editAntenna),
          1
          /* TEXT */
        )]),
        default: _withCtx(() => [_createVNode(XAntennaEditor, {
          antenna: __props.antenna,
          onCreated: onAntennaCreated,
          onUpdated: onAntennaUpdated,
          onDeleted: onAntennaDeleted
        }, null, 8, ["antenna"])]),
        _: 1
      }, 8, [
        "withOkButton",
        "width",
        "height"
      ]);
    };
  }
};
