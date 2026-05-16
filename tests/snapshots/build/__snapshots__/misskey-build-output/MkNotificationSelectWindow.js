import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, renderList as _renderList, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue";
import { ref, useTemplateRef } from "vue";
import { notificationTypes } from "misskey-js";
import MkSwitch from "./MkSwitch.vue";
import MkInfo from "./MkInfo.vue";
import MkButton from "./MkButton.vue";
import MkModalWindow from "@/components/MkModalWindow.vue";
import { i18n } from "@/i18n.js";
export default {
  __name: "MkNotificationSelectWindow",
  props: { excludeTypes: {
    type: Array,
    required: false,
    default: () => []
  } },
  emits: ["done", "closed"],
  setup(__props, { emit: __emit }) {
    const emit = __emit;
    const props = __props;
    const dialog = useTemplateRef("dialog");
    const typesMap = notificationTypes.reduce((p, t) => ({
      ...p,
      [t]: ref(!props.excludeTypes.includes(t))
    }), {});
    function ok() {
      emit("done", { excludeTypes: Object.keys(typesMap).filter((type) => !typesMap[type].value) });
      if (dialog.value) dialog.value.close();
    }
    function disableAll() {
      for (const type of notificationTypes) {
        typesMap[type].value = false;
      }
    }
    function enableAll() {
      for (const type of notificationTypes) {
        typesMap[type].value = true;
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
        onOk: _cache[0] || (_cache[0] = ($event) => ok()),
        onClose: _cache[1] || (_cache[1] = ($event) => _unref(dialog)?.close()),
        onClosed: _cache[2] || (_cache[2] = ($event) => emit("closed"))
      }, {
        header: _withCtx(() => [_createTextVNode(
          _toDisplayString(_unref(i18n).ts.notificationSetting),
          1
          /* TEXT */
        )]),
        default: _withCtx(() => [_createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-min: 20px; --MI_SPACER-max: 28px;"
        }, [_createElementVNode("div", { class: "_gaps_m" }, [
          _createVNode(MkInfo, null, {
            default: _withCtx(() => [_createTextVNode(
              _toDisplayString(_unref(i18n).ts.notificationSettingDesc),
              1
              /* TEXT */
            )]),
            _: 1
          }),
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
          (_openBlock(true), _createElementBlock(
            _Fragment,
            null,
            _renderList(_unref(notificationTypes), (ntype) => {
              return _openBlock(), _createBlock(MkSwitch, {
                key: ntype,
                modelValue: _unref(typesMap)[ntype].value,
                "onUpdate:modelValue": ($event) => _unref(typesMap)[ntype].value = $event
              }, {
                default: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._notification._types[ntype]),
                  1
                  /* TEXT */
                )]),
                _: 2
              }, 1032, ["modelValue", "onUpdate:modelValue"]);
            }),
            128
            /* KEYED_FRAGMENT */
          ))
        ])])]),
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
