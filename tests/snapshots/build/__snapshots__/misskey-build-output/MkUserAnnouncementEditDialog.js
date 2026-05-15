import { openBlock as _openBlock, createBlock as _createBlock, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, createSlots as _createSlots, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-trash" });
const _hoisted_2 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-check" });
import { ref, useTemplateRef } from "vue";
import MkModalWindow from "@/components/MkModalWindow.vue";
import * as os from "@/os.js";
import { misskeyApi } from "@/utility/misskey-api.js";
import { i18n } from "@/i18n.js";
export default {
  __name: "MkUserAnnouncementEditDialog",
  props: {
    user: {
      type: null,
      required: true
    },
    announcement: {
      type: null,
      required: false
    }
  },
  emits: ["done", "closed"],
  setup(__props, { emit: __emit }) {
    const emit = __emit;
    const props = __props;
    const dialog = useTemplateRef("dialog");
    const title = ref(props.announcement ? props.announcement.title : "");
    const text = ref(props.announcement ? props.announcement.text : "");
    const icon = ref(props.announcement ? props.announcement.icon : "info");
    const display = ref(props.announcement ? props.announcement.display : "dialog");
    const needConfirmationToRead = ref(props.announcement ? props.announcement.needConfirmationToRead : false);
    async function done() {
      const params = {
        title: title.value,
        text: text.value,
        icon: icon.value,
        imageUrl: null,
        display: display.value,
        needConfirmationToRead: needConfirmationToRead.value,
        userId: props.user.id
      };
      if (props.announcement) {
        await os.apiWithDialog("admin/announcements/update", {
          ...params,
          id: props.announcement.id
        });
        emit("done", { updated: {
          ...params,
          id: props.announcement.id
        } });
        dialog.value?.close();
      } else {
        const created = await os.apiWithDialog("admin/announcements/create", params);
        emit("done", { created });
        dialog.value?.close();
      }
    }
    async function del() {
      const { canceled } = await os.confirm({
        type: "warning",
        text: i18n.tsx.removeAreYouSure({ x: title.value })
      });
      if (canceled) return;
      if (props.announcement) {
        await misskeyApi("admin/announcements/delete", { id: props.announcement.id });
      }
      emit("done", { deleted: true });
      dialog.value?.close();
    }
    return (_ctx, _cache) => {
      return _openBlock(), _createBlock(MkModalWindow, {
        ref_key: "dialog",
        ref: dialog,
        width: 400,
        onClose: _cache[0] || (_cache[0] = ($event) => _unref(dialog)?.close()),
        onClosed: _cache[1] || (_cache[1] = ($event) => emit("closed"))
      }, _createSlots({ _: 2 }, [__props.announcement ? {
        name: "header",
        fn: _withCtx(() => [_createTextVNode(
          ":" + _toDisplayString(__props.announcement.title) + ":",
          1
          /* TEXT */
        )]),
        key: "0"
      } : {
        name: "header",
        fn: _withCtx(() => [_createTextVNode("New announcement")]),
        key: "1"
      }]), 1032, ["width"]);
    };
  }
};
