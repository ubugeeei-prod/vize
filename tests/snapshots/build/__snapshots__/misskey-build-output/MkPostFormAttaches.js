import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, withDirectives as _withDirectives, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, vShow as _vShow, withModifiers as _withModifiers, withKeys as _withKeys } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", {
  class: "ti ti-eye-exclamation",
  style: "margin: auto;"
});
import { inject } from "vue";
import { copyToClipboard } from "@/utility/copy-to-clipboard";
import MkDriveFileThumbnail from "@/components/MkDriveFileThumbnail.vue";
import MkDraggable from "@/components/MkDraggable.vue";
import * as os from "@/os.js";
import { misskeyApi } from "@/utility/misskey-api.js";
import { i18n } from "@/i18n.js";
import { prefer } from "@/preferences.js";
import { DI } from "@/di.js";
import { globalEvents } from "@/events.js";
export default {
  __name: "MkPostFormAttaches",
  props: {
    modelValue: {
      type: Array,
      required: true
    },
    detachMediaFn: {
      type: Function,
      required: false
    }
  },
  emits: [
    "update:modelValue",
    "detach",
    "changeSensitive",
    "changeName"
  ],
  setup(__props, { emit: __emit }) {
    const emit = __emit;
    const props = __props;
    const mock = inject(DI.mock, false);
    let menuShowing = false;
    function detachMedia(id) {
      if (mock) return;
      if (props.detachMediaFn) {
        props.detachMediaFn(id);
      } else {
        emit("detach", id);
      }
    }
    async function detachAndDeleteMedia(file) {
      if (mock) return;
      detachMedia(file.id);
      const { canceled } = await os.confirm({
        type: "warning",
        text: i18n.tsx.driveFileDeleteConfirm({ name: file.name })
      });
      if (canceled) return;
      await os.apiWithDialog("drive/files/delete", { fileId: file.id });
      globalEvents.emit("driveFilesDeleted", [file]);
    }
    function toggleSensitive(file) {
      if (mock) {
        emit("changeSensitive", file, !file.isSensitive);
        return;
      }
      misskeyApi("drive/files/update", {
        fileId: file.id,
        isSensitive: !file.isSensitive
      }).then(() => {
        emit("changeSensitive", file, !file.isSensitive);
      });
    }
    async function rename(file) {
      if (mock) return;
      const { canceled, result } = await os.inputText({
        title: i18n.ts.enterFileName,
        default: file.name,
        minLength: 1
      });
      if (canceled) return;
      misskeyApi("drive/files/update", {
        fileId: file.id,
        name: result
      }).then(() => {
        emit("changeName", file, result);
        file.name = result;
      });
    }
    async function describe(file) {
      if (mock) return;
      const { dispose } = await os.popupAsyncWithDialog(import("@/components/MkFileCaptionEditWindow.vue").then((x) => x.default), {
        default: file.comment !== null ? file.comment : "",
        file
      }, {
        done: (caption) => {
          let comment = caption.length === 0 ? null : caption;
          misskeyApi("drive/files/update", {
            fileId: file.id,
            comment
          }).then(() => {
            file.comment = comment;
          });
        },
        closed: () => dispose()
      });
    }
    function showFileMenu(file, ev) {
      if (menuShowing) return;
      const isImage = file.type.startsWith("image/");
      const menuItems = [];
      menuItems.push({
        text: i18n.ts.renameFile,
        icon: "ti ti-forms",
        action: () => {
          rename(file);
        }
      }, {
        text: file.isSensitive ? i18n.ts.unmarkAsSensitive : i18n.ts.markAsSensitive,
        icon: file.isSensitive ? "ti ti-eye-exclamation" : "ti ti-eye",
        action: () => {
          toggleSensitive(file);
        }
      }, {
        text: i18n.ts.describeFile,
        icon: "ti ti-text-caption",
        action: () => {
          describe(file);
        }
      });
      if (isImage) {
        menuItems.push({
          text: i18n.ts.preview,
          icon: "ti ti-photo-search",
          action: async () => {
            const { dispose } = await os.popupAsyncWithDialog(import("@/components/MkImgPreviewDialog.vue").then((x) => x.default), { file }, { closed: () => dispose() });
          }
        });
      }
      menuItems.push({ type: "divider" }, {
        text: i18n.ts.attachCancel,
        icon: "ti ti-circle-x",
        action: () => {
          detachMedia(file.id);
        }
      }, {
        text: i18n.ts.deleteFile,
        icon: "ti ti-trash",
        danger: true,
        action: () => {
          detachAndDeleteMedia(file);
        }
      });
      if (prefer.s.devMode) {
        menuItems.push({ type: "divider" }, {
          icon: "ti ti-hash",
          text: i18n.ts.copyFileId,
          action: () => {
            copyToClipboard(file.id);
          }
        });
      }
      os.popupMenu(menuItems, ev.currentTarget ?? ev.target).then(() => menuShowing = false);
      menuShowing = true;
    }
    return (_ctx, _cache) => {
      return _withDirectives((_openBlock(), _createElementBlock(
        "div",
        { class: _normalizeClass(_ctx.$style.root) },
        [_createVNode(MkDraggable, {
          modelValue: props.modelValue,
          class: _normalizeClass(_ctx.$style.files),
          direction: "horizontal",
          withGaps: "",
          "onUpdate:modelValue": _cache[0] || (_cache[0] = (v) => emit("update:modelValue", v))
        }, {
          default: _withCtx(({ item }) => [_createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.file),
            role: "button",
            tabindex: "0",
            onClick: ($event) => showFileMenu(item, $event),
            onKeydown: _withKeys(($event) => showFileMenu(item, $event), ["space", "enter"]),
            onContextmenu: _withModifiers(($event) => showFileMenu(item, $event), ["prevent", "stop"])
          }, [_createVNode(MkDriveFileThumbnail, {
            style: "pointer-events: none;",
            "data-id": item.id,
            class: _normalizeClass(_ctx.$style.thumbnail),
            file: item,
            fit: "cover"
          }, null, 10, ["data-id", "file"]), item.isSensitive ? (_openBlock(), _createElementBlock(
            "div",
            {
              key: 0,
              class: _normalizeClass(_ctx.$style.sensitive),
              style: "pointer-events: none;"
            },
            [_hoisted_1],
            2
            /* CLASS */
          )) : _createCommentVNode("v-if", true)], 42, [
            "onClick",
            "onKeydown",
            "onContextmenu"
          ])]),
          _: 1
        }, 10, ["modelValue"]), _createElementVNode(
          "p",
          { class: _normalizeClass([_ctx.$style.remain, { [_ctx.$style.exceeded]: props.modelValue.length > 16 }]) },
          _toDisplayString(props.modelValue.length) + "/16\n  ",
          3
          /* TEXT, CLASS */
        )],
        2
        /* CLASS */
      )), [[_vShow, props.modelValue.length != 0]]);
    };
  }
};
