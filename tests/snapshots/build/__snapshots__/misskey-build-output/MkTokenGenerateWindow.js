import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue";
import { useTemplateRef, ref } from "vue";
import * as Misskey from "misskey-js";
import MkInput from "./MkInput.vue";
import MkSwitch from "./MkSwitch.vue";
import MkButton from "./MkButton.vue";
import MkInfo from "./MkInfo.vue";
import MkModalWindow from "@/components/MkModalWindow.vue";
import { i18n } from "@/i18n.js";
import { iAmAdmin } from "@/i.js";
export default {
  __name: "MkTokenGenerateWindow",
  props: {
    title: {
      type: [String, null],
      required: false,
      default: null
    },
    information: {
      type: [String, null],
      required: false,
      default: null
    },
    initialName: {
      type: [String, null],
      required: false,
      default: null
    },
    initialPermissions: {
      type: [Array, null],
      required: false,
      default: null
    }
  },
  emits: ["closed", "done"],
  setup(__props, { emit: __emit }) {
    const emit = __emit;
    const props = __props;
    const defaultPermissions = Misskey.permissions.filter((p) => !p.startsWith("read:admin") && !p.startsWith("write:admin"));
    const adminPermissions = Misskey.permissions.filter((p) => p.startsWith("read:admin") || p.startsWith("write:admin"));
    const dialog = useTemplateRef("dialog");
    const name = ref(props.initialName);
    const permissionSwitches = ref({});
    const permissionSwitchesForAdmin = ref({});
    if (props.initialPermissions) {
      for (const kind of props.initialPermissions) {
        permissionSwitches.value[kind] = true;
      }
    } else {
      for (const kind of defaultPermissions) {
        permissionSwitches.value[kind] = false;
      }
      if (iAmAdmin) {
        for (const kind of adminPermissions) {
          permissionSwitchesForAdmin.value[kind] = false;
        }
      }
    }
    function ok() {
      emit("done", {
        name: name.value,
        permissions: [...Object.keys(permissionSwitches.value).filter((p) => permissionSwitches.value[p]), ...iAmAdmin ? Object.keys(permissionSwitchesForAdmin.value).filter((p) => permissionSwitchesForAdmin.value[p]) : []]
      });
      dialog.value?.close();
    }
    function disableAll() {
      for (const p in permissionSwitches.value) {
        permissionSwitches.value[p] = false;
      }
      if (iAmAdmin) {
        for (const p in permissionSwitchesForAdmin.value) {
          permissionSwitchesForAdmin.value[p] = false;
        }
      }
    }
    function enableAll() {
      for (const p in permissionSwitches.value) {
        permissionSwitches.value[p] = true;
      }
      if (iAmAdmin) {
        for (const p in permissionSwitchesForAdmin.value) {
          permissionSwitchesForAdmin.value[p] = true;
        }
      }
    }
    return (_ctx, _cache) => {
      return _openBlock(), _createBlock(MkModalWindow, {
        ref_key: "dialog",
        ref: dialog,
        width: 400,
        height: 450,
        withOkButton: true,
        okButtonDisabled: false,
        canClose: false,
        onClose: _cache[0] || (_cache[0] = ($event) => _unref(dialog)?.close()),
        onClosed: _cache[1] || (_cache[1] = ($event) => emit("closed")),
        onOk: _cache[2] || (_cache[2] = ($event) => ok())
      }, {
        header: _withCtx(() => [_createTextVNode(
          _toDisplayString(__props.title || _unref(i18n).ts.generateAccessToken),
          1
          /* TEXT */
        )]),
        default: _withCtx(() => [_createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-min: 20px; --MI_SPACER-max: 28px;"
        }, [_createElementVNode("div", { class: "_gaps_m" }, [
          __props.information ? (_openBlock(), _createElementBlock("div", { key: 0 }, [_createVNode(MkInfo, { warn: "" }, {
            default: _withCtx(() => [_createTextVNode(
              _toDisplayString(__props.information),
              1
              /* TEXT */
            )]),
            _: 1
          })])) : _createCommentVNode("v-if", true),
          _createElementVNode("div", null, [_createVNode(MkInput, {
            modelValue: name.value,
            "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event) => name.value = $event)
          }, {
            label: _withCtx(() => [_createTextVNode(
              _toDisplayString(_unref(i18n).ts.name),
              1
              /* TEXT */
            )]),
            _: 1
          }, 8, ["modelValue"])]),
          _createElementVNode("div", null, [_createElementVNode(
            "b",
            null,
            _toDisplayString(_unref(i18n).ts.permission),
            1
            /* TEXT */
          )]),
          _createElementVNode("div", { class: "_buttons" }, [_createVNode(MkButton, {
            inline: "",
            onClick: disableAll
          }, {
            default: _withCtx(() => [_createTextVNode(
              _toDisplayString(_unref(i18n).ts.disableAll),
              1
              /* TEXT */
            )]),
            _: 1
          }), _createVNode(MkButton, {
            inline: "",
            onClick: enableAll
          }, {
            default: _withCtx(() => [_createTextVNode(
              _toDisplayString(_unref(i18n).ts.enableAll),
              1
              /* TEXT */
            )]),
            _: 1
          })]),
          _createElementVNode("div", { class: "_gaps_s" }, [(_openBlock(true), _createElementBlock(
            _Fragment,
            null,
            _renderList(Object.keys(permissionSwitches.value), (kind) => {
              return _openBlock(), _createBlock(MkSwitch, {
                key: kind,
                modelValue: permissionSwitches.value[kind],
                "onUpdate:modelValue": ($event) => permissionSwitches.value[kind] = $event
              }, {
                default: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._permissions[kind]),
                  1
                  /* TEXT */
                )]),
                _: 2
              }, 1032, ["modelValue", "onUpdate:modelValue"]);
            }),
            128
            /* KEYED_FRAGMENT */
          ))]),
          _unref(iAmAdmin) ? (_openBlock(), _createElementBlock(
            "div",
            {
              key: 0,
              class: _normalizeClass(_ctx.$style.adminPermissions)
            },
            [_createElementVNode(
              "div",
              { class: _normalizeClass(_ctx.$style.adminPermissionsHeader) },
              [_createElementVNode(
                "b",
                null,
                _toDisplayString(_unref(i18n).ts.adminPermission),
                1
                /* TEXT */
              )],
              2
              /* CLASS */
            ), _createElementVNode("div", { class: "_gaps_s" }, [(_openBlock(true), _createElementBlock(
              _Fragment,
              null,
              _renderList(Object.keys(permissionSwitchesForAdmin.value), (kind) => {
                return _openBlock(), _createBlock(MkSwitch, {
                  key: kind,
                  modelValue: permissionSwitchesForAdmin.value[kind],
                  "onUpdate:modelValue": ($event) => permissionSwitchesForAdmin.value[kind] = $event
                }, {
                  default: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts._permissions[kind]),
                    1
                    /* TEXT */
                  )]),
                  _: 2
                }, 1032, ["modelValue", "onUpdate:modelValue"]);
              }),
              128
              /* KEYED_FRAGMENT */
            ))])],
            2
            /* CLASS */
          )) : _createCommentVNode("v-if", true)
        ])])]),
        _: 1
      }, 8, [
        "width",
        "height",
        "withOkButton",
        "okButtonDisabled",
        "canClose"
      ]);
    };
  }
};
