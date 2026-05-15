import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-player-play" });
const _hoisted_2 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-check" });
import { ref, computed, watch } from "vue";
import MkSelect from "@/components/MkSelect.vue";
import MkButton from "@/components/MkButton.vue";
import MkRange from "@/components/MkRange.vue";
import { i18n } from "@/i18n.js";
import * as os from "@/os.js";
import { useMkSelect } from "@/composables/use-mkselect.js";
import { misskeyApi } from "@/utility/misskey-api.js";
import { playMisskeySfxFile, soundsTypes, getSoundDuration } from "@/utility/sound.js";
import { selectFile } from "@/utility/drive.js";
export default {
  __name: "sounds.sound",
  props: { def: {
    type: null,
    required: true
  } },
  emits: ["update"],
  async setup(__props, { emit: __emit }) {
    let __temp, __restore;
    const emit = __emit;
    const props = __props;
    const { model: type, def: typeDef } = useMkSelect({
      items: soundsTypes.map((x) => ({
        label: getSoundTypeName(x),
        value: x
      })),
      initialValue: props.def.type
    });
    const fileId = ref("fileId" in props.def ? props.def.fileId : undefined);
    const fileUrl = ref("fileUrl" in props.def ? props.def.fileUrl : undefined);
    const fileName = ref("");
    const driveFileError = ref(false);
    const hasChanged = ref(false);
    const volume = ref(props.def.volume);
    if (type.value === "_driveFile_" && fileId.value) {
      await misskeyApi("drive/files/show", { fileId: fileId.value }).then((res) => {
        fileName.value = res.name;
      }).catch((res) => {
        driveFileError.value = true;
      });
    }
    function getSoundTypeName(f) {
      switch (f) {
        case null: return i18n.ts.none;
        case "_driveFile_": return i18n.ts._soundSettings.driveFile;
        default: return f;
      }
    }
    const friendlyFileName = computed(() => {
      if (fileName.value) {
        return fileName.value;
      }
      if (fileUrl.value) {
        return fileUrl.value;
      }
      return i18n.ts._soundSettings.driveFileWarn;
    });
    function selectSound(ev) {
      selectFile({
        anchorElement: ev.currentTarget ?? ev.target,
        multiple: false,
        label: i18n.ts._soundSettings.driveFile
      }).then(async (file) => {
        if (!file.type.startsWith("audio")) {
          os.alert({
            type: "warning",
            title: i18n.ts._soundSettings.driveFileTypeWarn,
            text: i18n.ts._soundSettings.driveFileTypeWarnDescription
          });
          return;
        }
        const duration = await getSoundDuration(file.url);
        if (duration >= 2e3) {
          const { canceled } = await os.confirm({
            type: "warning",
            title: i18n.ts._soundSettings.driveFileDurationWarn,
            text: i18n.ts._soundSettings.driveFileDurationWarnDescription,
            okText: i18n.ts.continue,
            cancelText: i18n.ts.cancel
          });
          if (canceled) return;
        }
        fileUrl.value = file.url;
        fileName.value = file.name;
        fileId.value = file.id;
        driveFileError.value = false;
        hasChanged.value = true;
      });
    }
    watch([type, volume], ([typeTo, volumeTo], [typeFrom, volumeFrom]) => {
      if (typeFrom !== typeTo && typeTo !== "_driveFile_") {
        fileUrl.value = undefined;
        fileName.value = "";
        fileId.value = undefined;
        driveFileError.value = false;
      }
      hasChanged.value = true;
    });
    function listen() {
      if (type.value === "_driveFile_" && (!fileUrl.value || !fileId.value)) {
        os.alert({
          type: "warning",
          text: i18n.ts._soundSettings.driveFileWarn
        });
        return;
      }
      playMisskeySfxFile(type.value === "_driveFile_" ? {
        type: "_driveFile_",
        fileId: fileId.value,
        fileUrl: fileUrl.value,
        volume: volume.value
      } : {
        type: type.value,
        volume: volume.value
      });
    }
    function save() {
      if (hasChanged.value === false || driveFileError.value === true) {
        return;
      }
      if (type.value === "_driveFile_" && !fileUrl.value) {
        os.alert({
          type: "warning",
          text: i18n.ts._soundSettings.driveFileWarn
        });
        return;
      }
      if (type.value !== "_driveFile_") {
        fileUrl.value = undefined;
        fileName.value = "";
        fileId.value = undefined;
      }
      emit("update", {
        type: type.value,
        fileId: fileId.value,
        fileUrl: fileUrl.value,
        volume: volume.value
      });
      os.success();
    }
    return (_ctx, _cache) => {
      const _component_MkCondensedLine = _resolveComponent("MkCondensedLine");
      return _openBlock(), _createElementBlock("div", { class: "_gaps_m" }, [
        _createVNode(MkSelect, {
          items: _unref(typeDef),
          modelValue: _unref(type),
          "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event) => type.value = $event)
        }, {
          label: _withCtx(() => [_createTextVNode(
            _toDisplayString(_unref(i18n).ts.sound),
            1
            /* TEXT */
          )]),
          _: 1
        }, 8, ["items", "modelValue"]),
        _unref(type) === "_driveFile_" && driveFileError.value === true ? (_openBlock(), _createElementBlock(
          "div",
          {
            key: 0,
            class: _normalizeClass(_ctx.$style.fileSelectorRoot)
          },
          [_createVNode(
            MkButton,
            {
              class: _normalizeClass(_ctx.$style.fileSelectorButton),
              inline: "",
              rounded: "",
              primary: "",
              onClick: selectSound
            },
            {
              default: _withCtx(() => [_createTextVNode(
                _toDisplayString(_unref(i18n).ts.selectFile),
                1
                /* TEXT */
              )]),
              _: 1
            },
            2
            /* CLASS */
          ), _createElementVNode(
            "div",
            { class: _normalizeClass(_ctx.$style.fileErrorRoot) },
            [_createVNode(_component_MkCondensedLine, null, {
              default: _withCtx(() => [_createTextVNode(
                _toDisplayString(_unref(i18n).ts._soundSettings.driveFileError),
                1
                /* TEXT */
              )]),
              _: 1
            })],
            2
            /* CLASS */
          )],
          2
          /* CLASS */
        )) : _unref(type) === "_driveFile_" ? (_openBlock(), _createElementBlock(
          "div",
          {
            key: 1,
            class: _normalizeClass(_ctx.$style.fileSelectorRoot)
          },
          [_createVNode(
            MkButton,
            {
              class: _normalizeClass(_ctx.$style.fileSelectorButton),
              inline: "",
              rounded: "",
              primary: "",
              onClick: selectSound
            },
            {
              default: _withCtx(() => [_createTextVNode(
                _toDisplayString(_unref(i18n).ts.selectFile),
                1
                /* TEXT */
              )]),
              _: 1
            },
            2
            /* CLASS */
          ), _createElementVNode(
            "div",
            { class: _normalizeClass(["_nowrap", !fileUrl.value && _ctx.$style.fileNotSelected]) },
            _toDisplayString(friendlyFileName.value),
            3
            /* TEXT, CLASS */
          )],
          2
          /* CLASS */
        )) : _createCommentVNode("v-if", true),
        _createVNode(MkRange, {
          min: 0,
          max: 1,
          step: .05,
          textConverter: (v) => `${Math.floor(v * 100)}%`,
          modelValue: volume.value,
          "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event) => volume.value = $event)
        }, {
          label: _withCtx(() => [_createTextVNode(
            _toDisplayString(_unref(i18n).ts.volume),
            1
            /* TEXT */
          )]),
          _: 1
        }, 8, [
          "min",
          "max",
          "step",
          "textConverter",
          "modelValue"
        ]),
        _createElementVNode("div", { class: "_buttons" }, [_createVNode(MkButton, {
          inline: "",
          onClick: listen
        }, {
          default: _withCtx(() => [
            _hoisted_1,
            _createTextVNode(" "),
            _createTextVNode(
              _toDisplayString(_unref(i18n).ts.listen),
              1
              /* TEXT */
            )
          ]),
          _: 1
        }), _createVNode(MkButton, {
          inline: "",
          primary: "",
          disabled: !hasChanged.value || driveFileError.value,
          onClick: save
        }, {
          default: _withCtx(() => [
            _hoisted_2,
            _createTextVNode(" "),
            _createTextVNode(
              _toDisplayString(_unref(i18n).ts.save),
              1
              /* TEXT */
            )
          ]),
          _: 1
        }, 8, ["disabled"])])
      ]);
    };
  }
};
